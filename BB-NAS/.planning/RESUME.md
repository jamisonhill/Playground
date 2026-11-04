# Resume — BB-NAS (Breaking Barriers DS1522+, 10.20.70.42)

**Paused:** 2026-11-04 · **Reason:** QNAP -> Creatives copy COMPLETE, verified, permissions fixed, access removed
**Phase:** 5 complete · Phase 4 hardening items remain

## State
- /volume1/Creatives/Archive 2 (57,567 files, 11.85 TB) + Archive 1 (104,081 files, 11.84 TB) copied 11-03 12:08 -> 11-04 13:01
  from QNAP TS-X82 over a direct 10G link; counts + bytes match the QNAP; rsync size+mtime verify = 0 diffs.
- All items inherit the share ACL (Creatives, hlewis, hunter, administrators: full). QNAP .streams/.@upload_cache junk deleted.
- Cleanup done: QNAP authorized_keys emptied, /root/.ssh + scripts/logs removed on BB-NAS, /etc/sudoers.d/claude-deploy removed
  (Claude has NO sudo on bb-nas now). The claude-deploy DSM user + ssh alias `bb-nas` still exist (non-admin).
- 10G direct link still configured: BB-NAS eth4 192.168.200.1 <-> QNAP eth5 192.168.200.2 (user decides keep/revert).
- Volume 1: about 30.0 TB used / 8.3 TB free.

## Next action
1. claude-deploy DSM user: KEEP (user, 11-04). 10G static IPs: left as is.
2. Still open: data scrub, UPS + safe shutdown, close Synology ticket #4057706.
