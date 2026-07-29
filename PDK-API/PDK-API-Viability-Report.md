# ProDataKey (PDK) Open API — Viability Report

**Prepared:** July 29, 2026
**Question:** How deep does the rabbit hole go? Can we build a custom GUI, or plugins that bridge PDK to Microsoft 365 and Planning Center Online?

---

## 1. Bottom Line Up Front

**Yes, it is a genuinely open system — and it is deeper than you probably expect.**

PDK does not expose a thin, read-only "reporting" API the way many access-control vendors do. It exposes the *same* REST and Streaming APIs that power pdk.io itself. Their own documentation is explicit about this: anything you can do in the pdk.io web interface, you can do from your own code.

That has three consequences worth stating plainly:

| Ambition | Verdict | Effort |
|---|---|---|
| Read-only dashboards / reports / kiosk displays | **Fully viable** | Low — a weekend |
| M365 (Entra ID) → PDK identity sync | **Fully viable** | Moderate — days, not weeks |
| Planning Center → PDK automation (volunteers, room bookings, check-in) | **Fully viable** | Moderate — the PDK half is easy, the PCO half needs design |
| Full replacement GUI for pdk.io | **Technically viable, strategically questionable** | High — months, and you inherit permanent maintenance |
| Branded mobile app with Bluetooth unlock | **Viable via official iOS/Android SDKs** | High — app store lifecycle, but SDK does the hard part |

**The single real gate is not technical — it is administrative.** You must be approved as an API partner before you get a `client_id` and `client_secret`. PDK grants this to both dealers *and* end customers, but it is a request-and-review process, not a self-service signup. Everything below is contingent on clearing that step.

---

## 2. The Access Gate — Read This First

There is no self-service developer portal signup. To get credentials you must either:

1. Fill out the form at `prodatakey.com/become-a-partner/api`, **or**
2. Email a brief description of your intended application to `integrations@prodatakey.com`

PDK's integrations team reviews the request and issues a `client_id` and `client_secret`. Their support documentation states that PDK allows "select Partners, both dealers and customers" access to the APIs — so as an end customer running your own system, you are eligible, but you are not automatically entitled.

**Unknowns you should resolve with PDK directly before committing engineering time:**

- Is there a fee for API access, or is it bundled with the system you already own?
- Does your dealer need to co-sign or enable anything on your organization?
- Are there NDA or contractual terms attached to the credentials?
- Any restrictions on building a *replacement* UI (as opposed to a complementary integration)?

That last question matters most for the "full new GUI" ambition. Nothing in the public docs prohibits it, but it is worth an explicit conversation rather than an assumption.

**Suggested framing when you email them:** describe it as an internal integration for your own organization's system — connecting staff identity management and facility scheduling to door access. That is a well-trodden, uncontroversial use case, and it is also the honest description of the highest-value work.

---

## 3. Architecture — The Mental Model

PDK 2.0 uses a cloud-first hierarchy. Understanding this is the difference between an afternoon of confusion and an afternoon of progress.

```
Organization  (your church, as a tenant in PDK's dealer/customer model)
    └── System  (the cloud source of truth — holders, credentials, groups, rules)
            └── Cloud Node  (a physical location / panel — the on-site gateway)
                    └── Connection  (a controller on the local network)
                            └── Device  (a door, gate, reader, relay, elevator floor)
```

**The critical thing to understand about 2.0:** the *System* lives in the cloud and is the source of truth. Cloud Nodes sync down from it. This is a change from the 1.0 architecture, where the on-premise panel was authoritative.

Practically, this means:

- You can configure people, credentials, groups, and rules **even when the hardware is offline**
- You can pre-configure a system **before hardware is installed**
- Hardware can be swapped without migrating data
- One System can span **multiple physical campuses** (multiple Cloud Nodes) with shared people and rules — directly relevant if Mercy Hill has more than one location

**If you find older sample code or blog posts, check which version they target.** The 1.0 → 2.0 migration renamed things significantly: `panel_id` in a subdomain became `system_id` in a path, `panel_token` became `system_token`, `persons` became `holders`, and integer IDs became UUID strings.

---

## 4. Authentication

Two OAuth 2.0 flows are supported.

### Client Credentials (recommended for your use cases)

App-to-system access with no human in the loop. Your application is granted access when an admin adds a permission in PDK using **the email address assigned to your application**. This behaves like a service account — it survives staff turnover, which matters for church IT where volunteers and roles rotate.

### Authorization Code

Per-user login and attribution. Use this only if you need to distinguish *which* human took an action inside your app, or if you are building something multiple people in your org log into with their own PDK identities.

### The Token Dance

Every API session follows the same three steps:

```
1. POST https://accounts.pdk.io/oauth2/token
   Authorization: Basic base64(client_id:client_secret)
   → returns an id_token (JWT, valid ~300 seconds)

2. POST https://accounts.pdk.io/api/systems/{system_id}/token
   Authorization: Bearer {id_token}
   → returns a system_token

3. Call system endpoints
   GET https://systems.pdk.io/{system_id}/holders
   Authorization: Bearer {system_token}
```

**Practical notes:**

- Tokens live about five minutes. Cache and reuse them — do not re-authenticate before every call.
- The authorization code flow can request `openid+offline_access` to get a **refresh token that never expires but is single-use** — each refresh returns a new one. If you build this, persist the new token atomically or you will lock yourself out.
- Two host families: `accounts.pdk.io` for identity/org/webhook administration, `systems.pdk.io` for everything about an actual access-control system.

---

## 5. The Full Endpoint Catalog — What Each Resource Actually Is

This is the map of the rabbit hole. Every resource below is full CRUD unless noted.

### Identity & People

**Holders** — A person who can be granted access. The central object of the whole system.
Fields include `id`, `firstName`, `lastName`, `email`, `photoUrl`, `partition`, `enabled`, `activeDate`, `expireDate`, `pin`, `duressPin`, `customFields`, `groups`, `credentials`.

Beyond basic CRUD it supports: photo upload (base64), **CSV bulk import with a preview/dry-run step and conflict resolution**, bulk delete, filtering by group, and **RSOP** (Resultant Set of Policy) queries that answer "given all rules and group memberships, what can this person actually open, and when?"

> The `activeDate` / `expireDate` fields plus `enabled` are the entire foundation of automated onboarding and offboarding. The RSOP endpoint is the foundation of any credible audit report.

**Credentials** — The thing a holder presents at a door. Three types:
- `card` — physical: prox card, fob, sticker, wristband, QR code
- `touch` — Bluetooth digital credential
- `token` — mobile app digital credential

Fields: `id`, `holderId`, `credentialNumber`, `facilityCode`, `description`, `types[]`.

Also exposes: facility code lookup, **digital credential invitations** (72-hour expiry — this is how you programmatically issue someone a phone credential), and self-help reset restriction management.

> The invite endpoint is the single most useful thing here. It means "new staff member appears in Entra ID → they receive a mobile credential invitation automatically" is a real, supported workflow.

**Groups** — Containers for applying shared rules to many holders at once. Simple object (`id`, `name` up to 70 chars, `partition`) but rich relationship endpoints: add/remove a single holder, bulk-assign holders to a group, bulk-assign groups to a holder.

> This is your synchronization target. Map an Entra ID security group or a PCO list to a PDK group and the whole sync problem becomes tractable.

**Custom Fields** — Arbitrary structured metadata attached to holders. Six types: string (with regex validation), number (min/max, decimal places), boolean, enum, array, and object (nested). Fields can be required, and visibility set to hidden / readonly / visible. Scoped per partition.

> This is where you store the join keys: Entra ID object ID, PCO person ID, employee number, volunteer team. **Design these before you write any sync code.** Getting the identity correlation right is the whole ballgame in a sync integration.

### Access Policy

**Rules** — The policy engine. Four categories, five `type` values (`access`, `antiPassback`, `autoOpen`, `event`, `systemEvent`).

Key fields: `allow` (grant/deny boolean), `authenticationPolicy` (`cardOnly`, `pinOnly`, `cardOrPin`, `cardAndPin`), `devices[]`, `startTime`/`stopTime` in 24-hour HH:MM.

Scheduling supports recurring weekday patterns **or** a single specific date (YYYY-MM-DD).

Endpoints are scoped: `/holders/{id}/rules`, `/groups/{id}/rules`, `/auto-open`, `/system-events`.

> **The single-date rule is the killer feature for a church.** It means "this specific Tuesday evening, this specific side door unlocks from 6:00 to 9:00 PM because a PCO Calendar reservation exists" is directly expressible. Auto-open rules handle the recurring Sunday morning door schedule.

**Permissions** — Who can administer the *PDK platform* (distinct from holders, who merely walk through doors). Four roles: `reporter`, `manager`, `admin`, `integrator`. Endpoints live under `/api/organizations/{org_id}/permissions`.

**Partitions** — Logical subdivisions restricting which holders, groups, and rules a given manager or reporter can touch. Admins and integrators see across all partitions. PDK's own examples: tenants in a multi-tenant building, departments in a company, floors of a building.

> For a multi-campus church, partitions could delegate day-to-day management to a campus lead without exposing the whole organization. Custom fields are also partition-scoped, which is worth knowing before you design your schema.

### Hardware

**Systems** — The cloud source of truth. `GET /{system_id}`, `PATCH /{system_id}`, plus `/search` (cross-entity search) and `/status`. Also the endpoint that mints system tokens.

**Cloud Nodes** — The on-site gateway hardware ("panels" in older docs). One per physical location, typically. Fields cover identity (name, serial), access behavior (emergency card numbers, offline policy, kill timer before reverting to e-card-only), configuration (timezone, locale, allowed card formats), and live status (sync status, connection status).

**Connections** — Controllers on the local network beneath a cloud node. Five types: PDK controllers (Ethernet or WiMAC, 1–24 doors by model), Aperio hubs (up to 64 wireless locks), USB on-board, wireless coordinator, wireless gateway. Full network config exposed: `ipaddress`, `ipv6Address`, `dhcp`, `netmask`, `gateway`, `dns`, `macAddress`, `port`, `protocol`, `manufacturer`. Includes a **wireless diagnostics** endpoint.

**Devices** — The actual doors, gates, readers, and relays. Seven types: `primaryReader`, `additionalReader`, `ioDevice`, `premiumIoDevice`, `elevatorReader`, `additionalElevatorReader`, `floorRelay`.

Control actions available via API:

| Action | Effect |
|---|---|
| Open | Unlock / activate |
| Close | Lock / deactivate |
| Delay Open / Delay Close | Scheduled activation |
| Try Open | Momentary unlock-then-relock cycle |
| **Virtual Read** | Simulate a specific holder presenting a credential |
| Toggle | Reverse current state |
| Do Not Disturb | Suppress notifications |
| Clear Alarm | Clear a forced-door alarm |
| Floor Group Activation | Trigger elevator floor access |

There is also a bulk device **states** endpoint per cloud node — poll one call to render every door's live status.

> **Virtual Read deserves a callout.** It lets your software say "treat this as though Jane badged at the north door" — which means you can build remote-unlock that still honors that person's rules, permissions, and audit trail, rather than a blunt override. That is a much better foundation for a "let someone in from your phone" feature.

**Card Formats** — Two kinds. *Standard* formats are predefined bit schemas (bit counts, cardholder ID bit offsets, even/odd parity, facility code bits). *Custom* formats let you supply **a JavaScript module** to decode proprietary reader output. PDK-provided formats (`"standard": true`) cannot be deleted.

**Floor Groups** — Elevator control. A floor group binds an `elevatorReader` device to an array of `floorRelay` devices, so presenting a credential authorizes specific floors. Not applicable to most church campuses, but present if you ever need it.

**Maps** — Visual device placement. Two types: *local* maps (you supply an image as a data URL; devices get X/Y pixel coordinates, adjustable icon size and opacity) and *global* maps (third-party map service with lat/long). Maps can link to one another.

> If you build a custom GUI, this is your floor-plan view — and PDK already stores the placement data, so a custom UI and pdk.io stay in sync rather than diverging.

### Reporting

**Reports** — Three types: **holder** reports, **credential** reports, and **event** reports.

Each supports the same lifecycle:

```
POST /reports/{type}/preview                    → up to 50 rows, synchronous
POST /reports/{type}/generate                   → asynchronous job
GET  /reports/{type}/requests/{id}              → job metadata / status
GET  /reports/{type}/requests/{id}/view         → the results
GET  /reports/{type}/requests                   → list past reports
DELETE /reports/{type}/requests/{id}            → clean up
```

Filters by type:
- **Holder:** firstName, lastName, email, groups, enabled, credentials, partition
- **Credential:** credentialNumber, type, facilityCode, created/lastScan timestamps, status
- **Event:** occurrence date, event type, result, doors, holders, cards, connections, groups

Reports download as **ZIP containing CSV, HTML, or JSON**. Filter sets can be saved as reusable **templates**. A reporting-plan endpoint tells you your storage capacity and event/report retention window.

> Retention is worth checking early. If PDK only retains events for N days, and you want multi-year attendance or audit history, you need to be streaming events into your own store from day one. You cannot retroactively recover what aged out.

---

## 6. The Real-Time Layer

This is where the platform gets genuinely interesting, and where most of the creative applications live.

### Webhooks

Configured at `https://accounts.pdk.io/api/organizations/{org_id}/subscriptions`.

**Subscription properties:**
- `url` — must be HTTPS
- `scope` — `this` (org only), `recursive` (org and children), or specific cloud node IDs
- `events[]` — which event types to send
- Authentication — `None` or `Basic`
- SSL verification — on by default, disableable for self-signed certs
- **`secrecyLevel`** — `0` = full data, `1` = exclude PINs and credential numbers, `2` = also exclude names

**Security:** if the subscription includes a secret, every delivery carries an `X_PDK_SIGNATURE` header containing a **SHA-1 HMAC of the request body**. Validate it. Always. A webhook endpoint that can unlock doors and does not verify signatures is a door with a sign on it.

> `secrecyLevel: 2` is the responsible default for anything that logs or forwards events into a less-controlled system — a dashboard on a lobby TV, a Teams channel notification, a third-party analytics tool.

### Event Catalog

Roughly 80 event types across three scopes.

**Organization events (10)**
`organization.created` · `organization.updated` · `organization.deleted` · `permission.created` · `permission.updated` · `permission.deleted` · `permission.accepted` · `subscription.created` · `subscription.updated` · `subscription.deleted`

**System events (24)**
`holder.created` · `holder.updated` · `holder.deleted` · `credential.created` · `credential.updated` · `credential.deleted` · `group.created` · `group.updated` · `group.deleted` · `rule.created` · `rule.updated` · `rule.deleted` · `rule.antipassback.violation` · `device.created` · `device.updated` · `device.deleted` · `connection.created` · `connection.updated` · `connection.deleted` · `cloudnode.created` · `cloudnode.updated` · `cloudnode.deleted` · `partition.created` · `partition.updated` · `partition.deleted` · `floorgroup.created` · `floorgroup.updated` · `floorgroup.deleted` · `floorgroup.activated` · `cardformat.created` · `cardformat.deleted`

**Cloud node events (~46) — the operational firehose**

*Access requests:*
`device.request.allowed` · `device.request.denied` · `device.request.unknown` · `device.request.duress` · `device.request.found` · `device.request.filtered` · `device.request.multiallowed` · `device.request.ecard.allowed` · `device.request.ecard.denied`

*Door state and alarms:*
`device.input.dps.opened` · `device.input.dps.closed` · `device.input.rex.on` · `device.input.rex.off` · `device.input.relay.on` · `device.input.relay.off` · `device.input.virtualread` · `device.alarm.forced` · `device.alarm.forced.cleared` · `device.alarm.propped.on` · `device.alarm.propped.off` · `device.alarm.propped.alloff` · `device.alarm.circuitbreaker.on` · `device.alarm.circuitbreaker.off`

*Door control state:*
`device.autoopen.on` · `device.autoopen.off` · `device.autoopen.override.on` · `device.autoopen.override.off` · `device.forceopen.on` · `device.forceopen.off` · `device.forceclose.on` · `device.forceclose.off`

*Infrastructure health:*
`cloudnode.connected` · `cloudnode.disconnected` · `controller.alarm.comloss.on` · `controller.alarm.comloss.off` · `controller.input.lp.on` · `controller.input.lp.off` (low power) · `controller.input.pb.connected` · `controller.input.pb.disconnected` · `controller.input.pb.status` (battery) · `controller.input.pi.on` · `controller.input.pi.off` · `controller.input.pi.status` · `controller.input.po.present` · `controller.input.po.absent` (power) · `controller.input.time.unknown.enter` · `controller.input.time.unknown.exit`

> `device.request.duress` fires when someone enters their duress PIN — a person being forced to open a door. If you build nothing else real-time, build an alert on that one. Route it to a Teams channel and an SMS, immediately, with no batching.
>
> `device.alarm.propped.on` is the mundane workhorse: the door someone wedged open with a folding chair after Wednesday night. A Teams notification after N minutes solves a recurring, real problem.
>
> The `controller.input.po.absent` / `pb.status` / `comloss` family gives you free infrastructure monitoring. Feed it into whatever alerting you already run.

### Streaming API

Socket.IO over WebSocket. Connect to:

```
wss://systems.pdk.io?systemId={systemId}&token={systemToken}
```

Send a ping roughly every 25 seconds to keep the connection alive, then subscribe to topics via the `subscribeIsolated` event. Roughly 30+ topics, largely mirroring the cloud node webhook events, plus **aggregate events** — pre-composed, human-readable event descriptions with context already joined in.

**Note:** command-sending was *removed* from the Streaming API in 2.0. Streaming is now read-only; use REST for control. Do not follow 1.0-era examples here.

**Webhooks vs. Streaming — how to choose:**

| Use Webhooks when | Use Streaming when |
|---|---|
| You have a stable HTTPS endpoint | You are building a live UI |
| Events feed a durable store or a queue | You need sub-second latency |
| Delivery must survive your app restarting | A dropped moment is acceptable |
| Backend automation, sync, alerting | Dashboards, kiosks, live door maps |

A full custom GUI would use both: webhooks for the durable event log, streaming for the live view.

---

## 7. Mobile SDKs

Official **iOS** and **Android** SDKs. Documented capabilities:

- Users accept **Bluetooth credentials and activate them inside your app** — not PDK's app, yours
- Software buttons to open doors and gates
- Event-driven alerts

PDK provides a demo app as a starting point. Contact `integrations@prodatakey.com` for the SDK itself; it is not on a public package registry.

Notably, reporting from the field suggests PDK provides the mobile credential type to integrators **at no additional per-credential cost** — unusual and favorable in this industry, where per-credential mobile fees are the norm. Confirm this applies to your agreement.

---

## 8. Constraints and Limits

### Rate Limits

| Host | Writes/sec | Total requests/sec |
|---|---|---|
| `accounts.pdk.io` | 10 | 35 |
| `systems.pdk.io` | 20 | 70 |

Reporting endpoints have daily caps **per system**:
- Preview: 500/day
- Generate: 150/day

Every response includes `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` (Unix seconds). Exceeding a limit returns **429**. Use exponential backoff.

> These limits are generous for a single-organization integration and would only bite during a large initial import. Note the report caps are daily and per-system — a dashboard that regenerates a report on every page load will exhaust 150 generates fast. Cache aggressively, or use `preview` for interactive views and reserve `generate` for scheduled exports.

### Pagination

`?page=` (zero-based) and `?per_page=`. Defaults and maximums vary per endpoint. Responses carry a `Link` header (first/prev/next/last) and `X-Total-Count`.

### Tooling Maturity

- **Postman collection** — officially published. Start here; it is the fastest way to validate credentials and explore.
- **Code tutorial** — official walkthroughs in JavaScript (native `fetch`) and C#.
- **`prodatakey/node-pdk`** — official JS client on GitHub, but self-described as beta, pre-1.0, targeting Node ≥ 8 and the older auth/panel API surface. **Treat it as a reference implementation, not a dependency.** For 2.0 work, write your own thin client; the API is plain REST + JSON and the auth flow is three requests.

> There is no official TypeScript SDK, no Python SDK, no OpenAPI spec published publicly. You will hand-roll your client. Given how simple the surface is, that is a minor cost — but it does mean no generated types, so budget time for writing your own interfaces if you go TypeScript.

---

## 9. What You Could Actually Build

Ordered by value-to-effort, not by ambition.

### Tier 1 — Read-Only Visibility (a weekend)

- **Live door-status board** for the office or security desk. Poll the bulk device-states endpoint, or subscribe to the streaming API. Show every door, locked/unlocked/propped/offline, on one screen.
- **Who's in the building right now** — derive from `device.request.allowed` events since the last "all doors locked" boundary.
- **Weekly access audit email** — event reports filtered to after-hours and denied attempts, delivered Monday morning.
- **Infrastructure health monitor** — subscribe to the controller power/battery/comloss events, alert into Teams. This alone may justify the whole effort; access control panels fail silently until the day they matter.

### Tier 2 — Microsoft 365 / Entra ID Integration (days)

This is the highest-value work and the most defensible.

**Onboarding.** A new staff member is created in Entra ID → your service creates a PDK holder, stamps their Entra object ID into a custom field, assigns groups derived from their Entra group membership, and sends a mobile credential invitation to their work email. Their badge works before their first day.

**Offboarding — the one that actually matters.** An account is disabled in Entra ID → the corresponding PDK holder is set `enabled: false` within minutes, and every credential is revoked. Today, in most churches, this is a manual checklist item that gets missed. That gap is the single largest physical-security risk most church IT departments carry, and it is entirely automatable with the endpoints above.

**Continuous reconciliation.** A scheduled job that compares Entra ID against PDK holders and reports drift: people in PDK who no longer exist in Entra, people whose group memberships diverge, credentials with no active holder. Run it nightly, email the exceptions.

**Implementation shape:** Microsoft Graph on one side (`/users`, `/groups`, and Graph change notifications for near-real-time), PDK REST on the other, correlated through a PDK custom field holding the Entra object ID. Run it as an Azure Function on a timer, or event-driven off Graph subscriptions. Both sides authenticate as service principals — no user sessions to babysit.

**Bidirectional alerting.** PDK webhook → Teams via an incoming webhook or Graph. Duress events, forced doors, after-hours access, panel offline. Use `secrecyLevel: 2` so names do not land in a Teams channel.

### Tier 3 — Planning Center Integration (days to weeks)

Planning Center runs a single unified API across People, Services, Groups, Check-Ins, Calendar, and Giving, supports both personal access tokens and OAuth, and offers webhooks. That makes it a workable counterpart, with more design ambiguity than the M365 side.

**Highest-value ideas, roughly in order:**

**Event-driven door schedules.** A PCO Calendar reservation for a room implies people need to reach that room. Generate PDK single-date rules from confirmed reservations — the side entrance unlocks 30 minutes before a Tuesday night small group and relocks after. Reservation cancelled, rule removed. This eliminates a recurring category of "someone has to go let them in."

**Volunteer and team-based access.** Map PCO Services teams or People lists to PDK groups. The worship team gets the sound booth and green room. Facilities volunteers get mechanical spaces. Membership changes in PCO propagate to door access automatically — and, more importantly, *removals* propagate too.

**Background-check gating.** If your PCO People workflow tracks background check status and expiration, key PDK access to it. Children's ministry doors open only for holders whose check is current. When a check expires in PCO, access to those specific doors is revoked automatically. Use the holder `expireDate` field, or targeted group removal.

**Children's ministry security.** This is where the two systems have real synergy. Subscribe to `device.request.*` on children's-wing doors and correlate against active PCO Check-Ins. An access-allowed event at a children's door during a service, by someone not on the checked-in volunteer roster, is worth flagging in real time.

**Attendance context.** Stream door events into your own store and correlate with PCO Check-Ins for a fuller picture of building usage — which entrances are actually used, when the building is genuinely empty, whether facility scheduling matches reality.

**A caution on the PCO side.** PCO's webhook coverage is stronger for People and Giving than for Calendar; you may need scheduled polling for reservations. Also, PCO is the messier data source — duplicate person records, inconsistent email addresses, people who exist in PCO but not Entra. **Do not let PCO create PDK holders unattended.** Have it propose, and have a human approve, at least until you trust the matching. Removals are safer to automate than creations; the failure mode of an over-eager removal is an inconvenience, while the failure mode of an over-eager creation is an unauthorized credential.

### Tier 4 — A Full Replacement GUI (months)

Technically supported. Every object pdk.io renders is reachable. You would build against holders, credentials, groups, rules, devices, maps, reports, plus streaming for live state.

**Be honest about the trade:**

*What you would gain:* a UI shaped like your ministry rather than a generic commercial building. Terminology your volunteers understand. Workflows that match how a church actually operates — "unlock for tonight's event" as a single button instead of a rules dialog. Deep PCO and M365 context shown inline. Mobile-first for staff who are never at a desk.

*What you would take on:* permanent maintenance of a security-critical application. Every PDK API change becomes your migration. Every bug becomes a potential lockout or an unauthorized entry. You lose vendor support for anything you touch. And pdk.io does not go away — it remains, so you now have two interfaces that can disagree.

**The recommendation:** do not replace the GUI. Build *purpose-built surfaces* alongside it. A single-purpose "unlock the building for tonight's event" page that a volunteer coordinator can use without training is worth more than a full admin console rebuild, at perhaps five percent of the cost and risk. Keep pdk.io as the system of record for administration; build thin, opinionated tools on top for the specific jobs it does awkwardly.

If a custom GUI still appeals after that, the **Marketplace** path is worth knowing: PDK lists integrations as tiles inside pdk.io. A customer clicks configure, gets redirected to your URL with an encrypted `token` query parameter, your app exchanges that token via `accounts.pdk.io` for their system ID, and the customer is returned to pdk.io afterward. It is the sanctioned way to distribute an integration to other organizations — relevant only if you ever want to share this with other churches, which is a genuinely plausible outcome.

---

## 10. Risks and Open Questions

| Risk | Assessment |
|---|---|
| **API access denied or costed** | The only hard blocker. Resolve first, before any design work. |
| **Event retention window** | If PDK retains events for a short period, long-term audit history requires you to stream and store from day one. Ask them the number. |
| **You now hold door-unlocking credentials** | A `client_secret` that can open every door in the building is a materially different secret from an API key for a reporting tool. It belongs in Azure Key Vault with rotation and access logging, never in a repo, never in a `.env` on a laptop. |
| **Webhook endpoint is an attack surface** | It is an internet-reachable HTTPS endpoint associated with your access control system. Verify `X_PDK_SIGNATURE` on every request. Reject unsigned. Rate-limit it. |
| **Sync bugs have physical consequences** | A logic error in an offboarding sync either leaves a former employee with access or locks out your entire staff on a Sunday morning. Build a dry-run mode. Log every intended change before making it. Cap the blast radius — refuse to execute a batch that would disable more than N holders without a human confirming. |
| **Bus factor** | If you build this and later leave, someone inherits an undocumented service that controls the doors. Document it as though for a stranger, because eventually it will be. |
| **`node-pdk` is beta and dated** | Do not take a dependency. Write ~200 lines of your own client. |
| **1.0 vs 2.0 confusion** | Most findable examples predate 2.0. Verify version on every snippet you copy. |

---

## 11. Recommended Path

**Step 1 — Clear the gate (this week).**
Email `integrations@prodatakey.com`. Describe it as an internal integration connecting your staff directory and facility scheduling to door access. Ask specifically about: cost, whether your dealer must enable anything, contractual terms, and the event retention window. Nothing else can start until this is answered.

**Step 2 — Prove the loop (one evening, once credentialed).**
Import the Postman collection. Authenticate, mint a system token, list holders. That single round trip validates everything and takes minutes.

**Step 3 — Build the cheapest useful thing.**
A webhook receiver that posts duress, forced-door, propped-door, and panel-offline events to a Teams channel. Small, immediately valuable, and it teaches you the event model with no write access and no risk. Nothing it does can lock anyone out.

**Step 4 — Design the identity schema before writing sync code.**
Decide which custom fields hold your correlation keys, which system wins on conflict, and what happens to a holder who exists in PDK but in neither Entra nor PCO. **Write this down before you write code.** Every painful integration failure traces back to this step being skipped.

**Step 5 — Offboarding sync first, in report-only mode.**
Entra ID disabled → PDK holder disabled. Run it for two weeks logging what it *would* do without doing it. Read the log every day. When it has been correct for two weeks straight, turn on enforcement. This is the highest-value automation in the entire project and it should be the first thing that writes.

**Step 6 — Then onboarding, then PCO.**
Onboarding is lower-risk than offboarding is high-value, so it comes second. PCO comes after both, because its data is messier and its wins are more discretionary.

**Step 7 — Purpose-built UI surfaces, if still wanted.**
By now you will know the API well enough to judge honestly whether a full GUI is worth it. The prediction: you will have solved eighty percent of the real pain in Tiers 1–3 and find the remaining twenty does not justify a rebuild.

---

## 12. Verdict

The rabbit hole goes **all the way down**. This is a real platform API, not a courtesy endpoint: full CRUD on people, credentials, groups, and access rules; direct hardware control including a virtual-read primitive that preserves the audit trail; ~80 webhook event types; a Socket.IO streaming feed; an asynchronous reporting engine; and official mobile SDKs with Bluetooth credentials. It is well documented, conventionally designed, and generously rate-limited.

The constraint is not depth. It is that the interesting work is *identity synchronization* — and identity synchronization is always harder than the API that exposes it. The PDK half of every project described above is the easy half.

**Build the M365 offboarding sync. Build the duress and propped-door alerting. Build the PCO event-to-door-schedule bridge. Those three deliver nearly all the available value.**

**Do not rebuild the GUI.** Build small, sharp tools that do the two or three things pdk.io does awkwardly, and let PDK keep maintaining the rest.

---

## Sources

- [PDK Developer Portal](https://developer.pdk.io/)
- [Web API 2.0 Introduction](https://developer.pdk.io/web/2.0/introduction/)
- [New 2.0 Architecture](https://developer.pdk.io/web/2.0/new-architecture/)
- [Authentication](https://developer.pdk.io/web/2.0/authentication/)
- [Rate Limits](https://developer.pdk.io/web/2.0/rate-limits/)
- [Pagination](https://developer.pdk.io/web/2.0/pagination/)
- [Marketplace](https://developer.pdk.io/web/2.0/marketplace/)
- [Video Integrations](https://developer.pdk.io/web/2.0/video-integrations/)
- [Code Tutorial](https://developer.pdk.io/web/2.0/code-tutorial/)
- [REST — Holders](https://developer.pdk.io/web/2.0/rest/holders/)
- [REST — Credentials](https://developer.pdk.io/web/2.0/rest/credentials/)
- [REST — Groups](https://developer.pdk.io/web/2.0/rest/groups/)
- [REST — Rules](https://developer.pdk.io/web/2.0/rest/rules/)
- [REST — Devices](https://developer.pdk.io/web/2.0/rest/devices/)
- [REST — Systems](https://developer.pdk.io/web/2.0/rest/systems/)
- [REST — Cloud Nodes](https://developer.pdk.io/web/2.0/rest/cloud-nodes/)
- [REST — Connections](https://developer.pdk.io/web/2.0/rest/connections/)
- [REST — Organizations](https://developer.pdk.io/web/2.0/rest/organizations/)
- [REST — Permissions](https://developer.pdk.io/web/2.0/rest/permissions/)
- [REST — Partitions](https://developer.pdk.io/web/2.0/rest/partitions/)
- [REST — Custom Fields](https://developer.pdk.io/web/2.0/rest/custom-fields/)
- [REST — Card Formats](https://developer.pdk.io/web/2.0/rest/card-formats/)
- [REST — Floor Groups](https://developer.pdk.io/web/2.0/rest/floor-groups/)
- [REST — Maps](https://developer.pdk.io/web/2.0/rest/maps/)
- [REST — Reports](https://developer.pdk.io/web/2.0/rest/reports/)
- [REST — Webhooks](https://developer.pdk.io/web/2.0/rest/webhooks/)
- [Streaming API](https://developer.pdk.io/web/2.0/streaming/)
- [iOS SDK](https://developer.pdk.io/ios/introduction/)
- [Android SDK](https://developer.pdk.io/android/introduction/)
- [PDK Open API — Partner Page](https://www.prodatakey.com/become-a-partner/api)
- [PDK Support — API Integration Basics](https://support.pdk.io/hc/en-us/sections/14607541679255-API-Integration-Basics)
- [prodatakey/node-pdk on GitHub](https://github.com/prodatakey/node-pdk)
- [Planning Center for Developers](https://www.planningcenter.com/developers)
- [Planning Center API — Webhooks](https://api.planningcenteronline.com/docs/overview/webhooks)
