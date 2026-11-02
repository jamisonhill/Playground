#!/bin/bash
# Monitors the BB-NAS -> MH-Production Hunter rsync every 15 minutes.
# - Kills rsync immediately if any NEW ECC / machine-check event appears.
# - Exits (which notifies Claude) when rsync stops, ECC trips, or SSH keeps failing.

LOG="$(dirname "$0")/hunter-monitor.log"
BASELINE_MSG_ECC=128   # all 128 existing hits are from 2026-10-28 (old DIMM)
INTERVAL=900
ssh_failures=0

while true; do
  # One SSH round-trip gathers everything; fields separated by "|"
  status=$(ssh -o ConnectTimeout=15 -o BatchMode=yes bb-nas '
    dm=$(sudo dmesg | grep -ciE "ECC notification|Hardware Error")
    msg=$(sudo grep -ciE "ECC notification|Machine Check" /var/log/messages)
    pids=$(ps aux | grep "[r]sync -rlt" | awk "{print \$2}" | tr "\n" " ")
    up=$(cut -d. -f1 /proc/uptime)
    prog=$(sudo tail -c 300 /root/hunter-rsync.log | tr "\r" "\n" | grep -E "[0-9]" | tail -1 | xargs)
    echo "$dm|$msg|$pids|$up|$prog"' 2>&1)

  # SSH failure: NAS may be rebooting or the network dropped
  if [[ "$status" != *"|"* ]]; then
    ssh_failures=$((ssh_failures + 1))
    echo "$(date '+%F %T') SSH FAILED ($ssh_failures): $status" >> "$LOG"
    if (( ssh_failures >= 3 )); then
      echo "$(date '+%F %T') EXIT: bb-nas unreachable 3 checks in a row" | tee -a "$LOG"
      exit 2
    fi
    sleep "$INTERVAL"; continue
  fi
  ssh_failures=0

  IFS='|' read -r dmesg_ecc msg_ecc rsync_pids uptime_s progress <<< "$status"
  echo "$(date '+%F %T') ecc_dmesg=$dmesg_ecc ecc_msgs=$msg_ecc uptime=${uptime_s}s pids=[${rsync_pids% }] $progress" >> "$LOG"

  # Any new ECC event -> stop the copy right away
  if (( dmesg_ecc > 0 || msg_ecc > BASELINE_MSG_ECC )); then
    ssh bb-nas 'sudo kill $(ps aux | grep -E "[r]sync -rlt|[h]unter-move.sh" | awk "{print \$2}") 2>/dev/null'
    echo "$(date '+%F %T') EXIT: ECC EVENT DETECTED — rsync killed (dmesg=$dmesg_ecc msgs=$msg_ecc)" | tee -a "$LOG"
    exit 3
  fi

  # rsync gone -> finished, crashed, or NAS rebooted; show the log tail so we can tell which
  if [[ -z "${rsync_pids// }" ]]; then
    echo "$(date '+%F %T') EXIT: rsync no longer running (uptime=${uptime_s}s)" | tee -a "$LOG"
    ssh bb-nas 'sudo tail -c 1500 /root/hunter-rsync.log | tr "\r" "\n" | tail -25' | tee -a "$LOG"
    exit 0
  fi

  sleep "$INTERVAL"
done
