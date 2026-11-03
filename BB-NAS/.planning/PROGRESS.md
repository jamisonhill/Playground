# Progress — BB-NAS

## Phase 1 — Recovery from full volume + RW cache hang [COMPLETE]
## Phase 2 — RAM root cause (ECC) + DIMM swap [COMPLETE]
## Phase 3 — Hunter migration to MH-Production [COMPLETE]

## Phase 4 — Follow-up transfers + hardening
- [x] BB-NAS -> MH-Production transfers done (user, 2026-11-03)
- [x] Removed /root/.rsync66 + hunter-move.sh/logs from BB-NAS (2026-11-03)
- [ ] Phase 5: QNAP (Thunderbolt 169.254.6.249) -> BB-NAS /volume1/Creatives ← strategy proposed
- [ ] Remove /etc/sudoers.d/claude-deploy after the Creatives transfer
- [ ] Data scrub (Storage Manager > Storage Pool 1)
- [ ] Install UPS, then set Control Panel > Hardware & Power > UPS to a safe shutdown
- [ ] Close Synology ticket #4057706
