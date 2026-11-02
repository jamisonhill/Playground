# Resume — BB-NAS (Breaking Barriers DS1522+, 10.20.70.42)

**Paused:** 2026-11-02 · **Reason:** Hunter migration finished; waiting on next transfer tasks
**Phase/Task:** Hunter migration COMPLETE · awaiting the user's next automated transfer request
**Tree:** clean · **Last commit:** see `git log -1`

## State
- Hunter + BRoll From Hunter + HUNTER RECORDED Backups are on MH-Production at
  /volume2/Production/z_MigratedFromBB-NAS/: 60,892 files, 20.62T, size+mtime verified, permissions fixed.
- The sources were deleted from BB-NAS. Volume 1 is now 5.78 TiB used / 29.13 TiB free.
- The checksum verify was canceled (too long, about 2 days). The quick verify plus matching counts and bytes was accepted.
- The replacement 16 GB ECC DIMM has had 0 ECC errors since 10-29 (the 128 entries in /var/log/messages are all from 10-28).
- Claude access is intentionally kept: /etc/sudoers.d/claude-deploy and /root/.rsync66 on BB-NAS.

## Next action
1. Ask the user what to transfer next. Reuse BB-NAS/hunter-move.sh as a template (dry → run →
   quick verify) and BB-NAS/transfer-monitor.sh for the 15-min ECC watch (kills rsync on ECC).
2. When the user says the transfers are done, remove /root/.rsync66 and /etc/sudoers.d/claude-deploy.
3. Still open: the data scrub, a UPS (then set the safe shutdown), and closing Synology ticket #4057706.

## Gotchas
- Transfers saturate both NASes, and the videographer edits on MH-Production. Pause with
  `sudo kill -STOP <pid>` / `-CONT`, or use --bwlimit during work hours.
- Never run the `-c` checksum verify without saying it takes about 2 days for 20T.
- `df -h` rounds up. Quote `df -B1` values to match DSM.
- DSM has no pgrep, so use `ps aux | grep "[r]sync -rlt"`. Folder names may end in U+F028, so match them with globs.
- More detail is in the bb-nas-recovery auto-memory.
