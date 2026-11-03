#!/bin/bash
# Copies the QNAP (TS-X82) "Archive 2" then "Archive 1" shares into BB-NAS /volume1/Creatives,
# pulling over the direct 10G cable (QNAP 192.168.200.2 <-> BB-NAS 192.168.200.1).
# Runs ON BB-NAS as root.  Usage: creatives-move.sh dry | run | verify
set -u

QNAP='admin@192.168.200.2'
# SSH over the 10G link; aes128-gcm is the cheapest cipher for both CPUs
SSH_CMD='ssh -i /root/.ssh/id_qnap -o BatchMode=yes -o ServerAliveInterval=30 -c aes128-gcm@openssh.com'
DEST_ROOT='/volume1/Creatives'
SHARES=('Archive 2' 'Archive 1')   # newest content first

# -r recurse, -l keep symlinks, -t keep modification times (no owners/perms: BB-NAS ACLs apply)
# -s (protect-args) sends paths with spaces to the QNAP safely
# Excludes: QNAP recycle bin, snapshots, thumbnail caches, Qsync db, Mac Finder junk
COMMON=(-rlts -h -e "$SSH_CMD"
        --exclude='@Recycle' --exclude='@Recently-Snapshot' --exclude='.@__thumb'
        --exclude='.syncing_db' --exclude='.DS_Store' --exclude='@eaDir')

# Safety check: the destination share must exist (created in DSM by the user)
if [ ! -d "$DEST_ROOT" ]; then echo "missing $DEST_ROOT"; exit 1; fi

mode="${1:-}"
for share in "${SHARES[@]}"; do
  src="$QNAP:/share/$share/"          # trailing slash = copy the share's contents
  dst="$DEST_ROOT/$share/"
  mkdir -p "$dst"
  echo "=== $(date '+%F %T') $mode: $share"
  case "$mode" in
    dry)    rsync "${COMMON[@]}" -n --stats "$src" "$dst" ;;
    run)    rsync "${COMMON[@]}" --partial --info=progress2 --stats "$src" "$dst" ;;
    verify) rsync "${COMMON[@]}" -n -i "$src" "$dst" ;;   # quick size+mtime check; lists anything that differs
    *)      echo "usage: $0 dry|run|verify"; exit 2 ;;
  esac
  rc=$?
  # Stop instead of moving to the next share if rsync failed (network drop, NAS reboot, etc.)
  if [ $rc -ne 0 ]; then echo "rsync exit $rc on $share - stopping"; exit $rc; fi
done
echo "=== $(date '+%F %T') $mode finished"
