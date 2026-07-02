// OtlpReceiver — Tier B of the ingestion pipeline (SPEC §2).
//
// A minimal OTLP/gRPC collector on 127.0.0.1:4317 so no external collector is
// needed. Claude Code exports here when the user launches it with:
//
//   CLAUDE_CODE_ENABLE_TELEMETRY=1
//   OTEL_METRICS_EXPORTER=otlp  OTEL_LOGS_EXPORTER=otlp
//   OTEL_EXPORTER_OTLP_PROTOCOL=grpc
//   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
//
// What we consume:
//   claude_code.cost.usage (metric)  → authoritative burn rate
//   claude_code.api_error  (log)     → API-error lamp; status 429 → rate lamp
// Everything else is accepted and ignored (a collector must not reject data).

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use opentelemetry_proto::tonic::collector::logs::v1::logs_service_server::{
    LogsService, LogsServiceServer,
};
use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::common::v1::any_value;
use tonic::{Request, Response, Status};

/// How long a lamp stays lit after its triggering event.
const API_ERROR_LAMP_MS: i64 = 30_000;
const RATE_LIMIT_LAMP_MS: i64 = 60_000;
/// The exporter sends every ~60s; consider Tier B "connected" within 3 misses.
const CONNECTED_WINDOW_MS: i64 = 190_000;
/// Burn-rate window over authoritative cost deltas (matches the estimate path).
const COST_WINDOW_MS: i64 = 60_000;
const COST_RETENTION_MS: i64 = 300_000;

#[derive(Default)]
pub struct OtlpState {
    /// Wall-clock of the last export received — drives the "connected" lamp.
    pub last_export_ms: i64,
    /// (arrival ms, USD delta) from claude_code.cost.usage counters.
    cost_samples: VecDeque<(i64, f64)>,
    /// Cumulative-counter bookkeeping: last seen value per series, so each
    /// export contributes only its increment.
    last_cost_by_series: HashMap<String, f64>,
    api_error_until_ms: i64,
    rate_limit_until_ms: i64,
}

impl OtlpState {
    pub fn is_connected(&self, now_ms: i64) -> bool {
        self.last_export_ms > 0 && now_ms - self.last_export_ms < CONNECTED_WINDOW_MS
    }

    pub fn api_error_active(&self, now_ms: i64) -> bool {
        now_ms < self.api_error_until_ms
    }

    pub fn rate_limit_active(&self, now_ms: i64) -> bool {
        now_ms < self.rate_limit_until_ms
    }

    /// Authoritative $/hr from cost.usage over the last minute; None when the
    /// exporter hasn't sent cost data recently (caller falls back to estimate).
    pub fn burn_rate_usd_per_hour(&self, now_ms: i64) -> Option<f64> {
        if !self.is_connected(now_ms) || self.cost_samples.is_empty() {
            return None;
        }
        let window_start = now_ms - COST_WINDOW_MS;
        let cost: f64 = self
            .cost_samples
            .iter()
            .filter(|(t, _)| *t >= window_start)
            .map(|(_, c)| c)
            .sum();
        Some(cost * (3_600_000.0 / COST_WINDOW_MS as f64))
    }

    fn record_cost(&mut self, series_key: String, cumulative_usd: f64, now_ms: i64) {
        let previous = self
            .last_cost_by_series
            .insert(series_key, cumulative_usd)
            .unwrap_or(0.0);
        // Counter reset (Claude Code restarted): the new value IS the delta.
        let delta = if cumulative_usd >= previous {
            cumulative_usd - previous
        } else {
            cumulative_usd
        };
        if delta > 0.0 {
            self.cost_samples.push_back((now_ms, delta));
        }
        while self
            .cost_samples
            .front()
            .is_some_and(|(t, _)| now_ms - t > COST_RETENTION_MS)
        {
            self.cost_samples.pop_front();
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Extract a string-ish attribute value ("429" from either a string or an int).
fn attr_to_string(value: &opentelemetry_proto::tonic::common::v1::AnyValue) -> String {
    match &value.value {
        Some(any_value::Value::StringValue(s)) => s.clone(),
        Some(any_value::Value::IntValue(i)) => i.to_string(),
        Some(any_value::Value::DoubleValue(d)) => d.to_string(),
        Some(any_value::Value::BoolValue(b)) => b.to_string(),
        _ => String::new(),
    }
}

struct MetricsReceiver {
    state: Arc<Mutex<OtlpState>>,
}

#[tonic::async_trait]
impl MetricsService for MetricsReceiver {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let now = now_ms();
        // The mutex is only ever held briefly by the pusher; poisoning would
        // mean that thread panicked, at which point the app is going down anyway.
        let mut state = self.state.lock().expect("otlp state mutex poisoned");
        state.last_export_ms = now;

        for resource_metrics in &request.get_ref().resource_metrics {
            for scope_metrics in &resource_metrics.scope_metrics {
                for metric in &scope_metrics.metrics {
                    if metric.name != "claude_code.cost.usage" {
                        continue;
                    }
                    // cost.usage is a Sum (cumulative counter) of doubles.
                    use opentelemetry_proto::tonic::metrics::v1::metric::Data;
                    let Some(Data::Sum(sum)) = &metric.data else {
                        continue;
                    };
                    for point in &sum.data_points {
                        use opentelemetry_proto::tonic::metrics::v1::number_data_point::Value;
                        let value = match point.value {
                            Some(Value::AsDouble(d)) => d,
                            Some(Value::AsInt(i)) => i as f64,
                            None => continue,
                        };
                        // Series key = the attribute set (session, model, …) so
                        // concurrent sessions each get their own counter.
                        let mut key: Vec<String> = point
                            .attributes
                            .iter()
                            .map(|kv| {
                                let v = kv.value.as_ref().map(attr_to_string).unwrap_or_default();
                                format!("{}={}", kv.key, v)
                            })
                            .collect();
                        key.sort();
                        state.record_cost(key.join(","), value, now);
                    }
                }
            }
        }
        Ok(Response::new(ExportMetricsServiceResponse::default()))
    }
}

struct LogsReceiver {
    state: Arc<Mutex<OtlpState>>,
}

#[tonic::async_trait]
impl LogsService for LogsReceiver {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let now = now_ms();
        let mut state = self.state.lock().expect("otlp state mutex poisoned");
        state.last_export_ms = now;

        for resource_logs in &request.get_ref().resource_logs {
            for scope_logs in &resource_logs.scope_logs {
                for record in &scope_logs.log_records {
                    // The event name can live in event_name, the body, or an
                    // "event.name" attribute depending on exporter version —
                    // check all three rather than betting on one.
                    let mut is_api_error = record.event_name.contains("api_error");
                    if let Some(body) = &record.body {
                        is_api_error |= attr_to_string(body).contains("api_error");
                    }
                    let mut status_code = String::new();
                    for kv in &record.attributes {
                        let value = kv.value.as_ref().map(attr_to_string).unwrap_or_default();
                        match kv.key.as_str() {
                            "event.name" => is_api_error |= value.contains("api_error"),
                            "status_code" => status_code = value,
                            _ => {}
                        }
                    }
                    if is_api_error {
                        state.api_error_until_ms = now + API_ERROR_LAMP_MS;
                        if status_code == "429" {
                            state.rate_limit_until_ms = now + RATE_LIMIT_LAMP_MS;
                        }
                    }
                }
            }
        }
        Ok(Response::new(ExportLogsServiceResponse::default()))
    }
}

/// Serve OTLP/gRPC on the given address until the process exits.
/// Exposed separately from `spawn_server` so tests can bind an ephemeral port.
pub async fn serve(state: Arc<Mutex<OtlpState>>, addr: std::net::SocketAddr) -> Result<(), tonic::transport::Error> {
    tonic::transport::Server::builder()
        .add_service(MetricsServiceServer::new(MetricsReceiver { state: state.clone() }))
        .add_service(LogsServiceServer::new(LogsReceiver { state }))
        .serve(addr)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue};
    use opentelemetry_proto::tonic::metrics::v1::{
        metric::Data, number_data_point, Metric, MetricsData, NumberDataPoint, ResourceMetrics,
        ScopeMetrics, Sum,
    };

    fn cost_export(session: &str, cumulative_usd: f64) -> ExportMetricsServiceRequest {
        let point = NumberDataPoint {
            attributes: vec![KeyValue {
                key: "session.id".into(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(session.into())),
                }),
                ..Default::default()
            }],
            value: Some(number_data_point::Value::AsDouble(cumulative_usd)),
            ..Default::default()
        };
        let metric = Metric {
            name: "claude_code.cost.usage".into(),
            data: Some(Data::Sum(Sum {
                data_points: vec![point],
                ..Default::default()
            })),
            ..Default::default()
        };
        let data = MetricsData {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![metric],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        ExportMetricsServiceRequest {
            resource_metrics: data.resource_metrics,
        }
    }

    /// End-to-end: run the real gRPC server on an ephemeral port and send two
    /// cumulative cost exports; the state should hold the delta, not the sum.
    #[tokio::test]
    async fn grpc_roundtrip_computes_cost_deltas() {
        let state = Arc::new(Mutex::new(OtlpState::default()));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener); // free the port for the server (tiny race, fine in tests)

        let server_state = state.clone();
        tokio::spawn(async move {
            let _ = serve(server_state, addr).await;
        });
        // Give the server a moment to bind before connecting.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let mut client = MetricsServiceClient::connect(format!("http://{addr}"))
            .await
            .expect("client should connect to in-process server");

        client.export(cost_export("s1", 0.50)).await.unwrap();
        client.export(cost_export("s1", 0.75)).await.unwrap(); // +0.25 delta

        let state = state.lock().unwrap();
        assert!(state.last_export_ms > 0);
        let total: f64 = state.cost_samples.iter().map(|(_, c)| c).sum();
        assert!((total - 0.75).abs() < 1e-9, "expected 0.50 + 0.25 deltas, got {total}");
        assert!(state.burn_rate_usd_per_hour(state.last_export_ms).unwrap() > 0.0);
    }
}
