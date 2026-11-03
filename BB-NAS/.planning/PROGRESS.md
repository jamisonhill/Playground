# Progress — BB-NAS

## Phase 1 — Recovery from full volume + RW cache hang [COMPLETE]
## Phase 2 — RAM root cause (ECC) + DIMM swap [COMPLETE]
## Phase 3 — Hunter migration to MH-Production [COMPLETE]

## Phase 4 — Follow-up transfers + hardening
- [x] BB-NAS -> MH-Production transfers done (user, 2026-11-03)
- [x] Removed /root/.rsync66 + hunter-move.sh/logs from BB-NAS (2026-11-03)
- [x] Phase 5 setup: 10G direct link (192.168.200.1/.2), keys, creatives-move.sh, dry run clean
- [ ] Phase 5 copy: QNAP Archive 2 + Archive 1 -> /volume1/Creatives (started 11-03 12:08, ETA 11-04 ~10:00) ← RUNNING
- [ ] Verify + permissions on Creatives; remove QNAP keys
- [ ] Remove /etc/sudoers.d/claude-deploy after the Creatives transfer
- [ ] Data scrub (Storage Manager > Storage Pool 1)
- [ ] Install UPS, then set Control Panel > Hardware & Power > UPS to a safe shutdown
- [ ] Close Synology ticket #4057706
