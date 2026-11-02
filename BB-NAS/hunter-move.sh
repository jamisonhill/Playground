#!/bin/bash
# Moves Hunter + two related folders from BB-NAS to MH-Production via the rsync daemon.
# Usage: hunter-move.sh dry | run | verify
set -u

# Locate sources by pattern; Hunter's name ends in an invisible Mac-SMB character (U+F028)
SRC="$(find /volume1/BreakingBarriers -mindepth 1 -maxdepth 1 -type d -name 'Hunter*')"
B="$(find '/volume1/BreakingBarriers/Video Project Archive' -maxdepth 2 -type d -name 'BRoll From Hunter*' -not -path '*@eaDir*')"
C="$(find '/volume1/BreakingBarriers/**Import' -maxdepth 2 -type d -name 'HUNTER RECORDED Backups*' -not -path '*@eaDir*')"
DEST='rsync://andy@10.20.70.66/Production/z_MigratedFromBB-NAS/'

# Refuse to run unless each pattern matched exactly one folder
for v in "$SRC" "$B" "$C"; do
  if [ -z "$v" ] || [ "$(printf '%s\n' "$v" | wc -l)" -ne 1 ]; then echo "source lookup failed: [$v]"; exit 1; fi
done

# Trailing slash on SRC copies Hunter's contents; B and C are copied as named folders
COMMON=(-rlt -h --password-file=/root/.rsync66
        --exclude='@eaDir' --exclude='#recycle' --exclude='.DS_Store')

case "${1:-}" in
  dry)    rsync "${COMMON[@]}" -n --stats "$SRC/" "$B" "$C" "$DEST" ;;
  run)    rsync "${COMMON[@]}" --partial --info=progress2 --stats "$SRC/" "$B" "$C" "$DEST" ;;
  verify) rsync "${COMMON[@]}" -c -n -i "$SRC/" "$B" "$C" "$DEST" ;;
  *)      echo "usage: $0 dry|run|verify"; exit 2 ;;
esac
