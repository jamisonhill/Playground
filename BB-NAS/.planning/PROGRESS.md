# Progress — BB-NAS

## Phase 1 — Recovery from full volume + RW cache hang [COMPLETE]
## Phase 2 — RAM root cause (ECC) + DIMM swap [COMPLETE]
## Phase 3 — Hunter migration to MH-Production [COMPLETE]

## Phase 4 — Follow-up transfers + hardening
- [x] BB-NAS -> MH-Production transfers done (user, 2026-11-03)
- [x] Removed /root/.rsync66 + hunter-move.sh/logs from BB-NAS (2026-11-03)
- [x] Phase 5 setup: 10G direct link (192.168.200.1/.2), keys, creatives-move.sh, dry run clean
- [x] Phase 5 copy: 161,648 files / 23.7 TB -> /volume1/Creatives (11-03 12:08 -> 11-04 13:01), verified, ACLs fixed
- [x] Removed QNAP keys, BB-NAS /root/.ssh + scripts, /etc/sudoers.d/claude-deploy (2026-11-04)
- [ ] Decide: claude-deploy DSM user on BB-NAS; 10G static IPs  ← PAUSED HERE
- [ ] Data scrub (Storage Manager > Storage Pool 1)
- [ ] Install UPS, then set Control Panel > Hardware & Power > UPS to a safe shutdown
- [ ] Close Synology ticket #4057706
