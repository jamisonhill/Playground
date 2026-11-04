# Resume — BB-NAS (Breaking Barriers DS1522+, 10.20.70.42)

**Paused:** 2026-11-04 · **Reason:** Creatives copy done; next is hardening + QNAP repurpose decisions
**Phase/Task:** Phase 5 — hardening (data scrub first)
**Tree:** clean · **Last commit:** see `git log -1`

## State
- /volume1/Creatives holds QNAP Archive 2 + Archive 1: 161,648 files / 23.7 TB; counts + bytes match the QNAP,
  verify 0 diffs, every item inherits the share ACL (Creatives, hlewis, hunter, administrators: full).
- The QNAP keys are removed and Claude has NO sudo on bb-nas. The claude-deploy user + `ssh bb-nas` are kept (user choice).
- The 10G direct link is still configured: BB-NAS eth4 192.168.200.1 <-> QNAP TS-X82 eth5 192.168.200.2.
- Volume 1: about 30.0 TB used / 8.3 TB free; 2 of 5 bays empty. No UPS yet.
- BB-NAS is now the ONLY copy of Creatives once the user wipes the QNAP.

## Next action
1. The user runs the data scrub (Storage Manager > Storage Pool 1) before wiping the QNAP.
2. The user checks the QNAP Public/Multimedia/Web/apps for anything to keep, then wipes or repurposes it.
3. Recommend a second copy (Hyper Backup / QNAP drives as backup target); UPS before any pool expansion.

## Gotchas
- Without sudo, Claude can't run synoacltool, read /var/log/messages or dmesg; ask the user to re-add
  /etc/sudoers.d/claude-deploy if root work is needed again.
- rsync onto an ACL share leaves items in Linux mode 000; fix with `synoacltool -enforce-inherit` per item.
- New drives are hot-swappable; SHR expansion takes days, so install the UPS first.
