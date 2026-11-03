# Resume — BB-NAS (Breaking Barriers DS1522+, 10.20.70.42)

**Updated:** 2026-11-03 12:15 · **Phase 5:** QNAP -> BB-NAS Creatives copy RUNNING
**Tree:** see `git log -1`

## State
- QNAP TS-X82 "Production-NAS" is direct-cabled 10G to BB-NAS: QNAP Ethernet 6 (eth5) 192.168.200.2 <->
  BB-NAS LAN 5 (eth4) 192.168.200.1, no gateway. Link 10000 Mb/s. QNAP also on Mac Thunderbolt 169.254.6.249 (unplugged now).
- /root/creatives-move.sh (dry|run|verify) on BB-NAS pulls QNAP "/share/Archive 2" then "/share/Archive 1"
  into /volume1/Creatives/<share>/ over ssh (root key /root/.ssh/id_qnap -> QNAP admin). Log /root/creatives-rsync.log.
- Started 2026-11-03 12:08: 161,648 files / 23.7 TB (A2 11.85T, A1 11.84T), ~305 MB/s, ETA ~Wed 11-04 10:00.
- Mac-side BB-NAS/transfer-monitor.sh runs every 15 min (kills rsync on any new ECC; baseline msgs=128).

## Next action
1. If rsync died but BB-NAS is up with ECC=0: `sudo bash -c "setsid nohup /root/creatives-move.sh run >> /root/creatives-rsync.log 2>&1 < /dev/null &"` (resumes).
2. After the copy: `creatives-move.sh verify` (no output lines = clean); check Creatives permissions
   (rsync ran as root without perms — files may need File Station "apply to sub-folders"/synoacltool inherit).
3. Cleanup: remove /root/.ssh/id_qnap*, QNAP authorized_keys entries (/etc/config/ssh + ~/.ssh), /etc/sudoers.d/claude-deploy;
   ask whether to keep the 10G static IPs. Nothing on the QNAP gets deleted.
4. Still open: data scrub, UPS + safe shutdown, close Synology ticket #4057706.

## Gotchas
- QNAP busybox: no `find -size +NM`, no bc. DSM: no pgrep, no scp/sftp (pipe via `ssh ... sudo tee`).
- QNAP share names differ from folder names (/share/Archive 2 -> CACHEDEV4 "Archive 01"); always use /share/<share>.
- `df -h` rounds up; quote `df -B1` values.
