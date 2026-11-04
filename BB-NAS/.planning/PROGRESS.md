# Progress — BB-NAS

## Phase 1 — Recovery from full volume + RW cache hang [COMPLETE]
## Phase 2 — RAM root cause (ECC) + DIMM swap [COMPLETE]
## Phase 3 — Hunter migration to MH-Production [COMPLETE]
## Phase 4 — QNAP Archive 1 + 2 -> /volume1/Creatives over direct 10G [COMPLETE]

## Phase 5 — Hardening + QNAP repurpose
- [ ] Data scrub (Storage Manager > Storage Pool 1) — do before the QNAP is wiped ← PAUSED HERE
- [ ] Install UPS, then set Control Panel > Hardware & Power > UPS to a safe shutdown
- [ ] User checks QNAP Public/Multimedia/Web/apps + the ~17 GB volume before wiping
- [ ] Decide a second copy of Creatives (Hyper Backup / reuse QNAP drives) — BB-NAS is now the only copy
- [ ] Optional: add drives to BB-NAS (2 free bays, hot-swap OK; UPS first; >=18 TB for full SHR gain)
- [ ] Close Synology ticket #4057706
