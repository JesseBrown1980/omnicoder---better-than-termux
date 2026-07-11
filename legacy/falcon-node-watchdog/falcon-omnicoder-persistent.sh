#!/data/data/com.termux/files/usr/bin/bash
# falcon-omnicoder-persistent.sh  (acer-pushed via direct file manager 2026-06-14)
# Newer persistent respawn-watchdog for the Falcon omnicoder so it runs PERMANENTLY
# and can receive messages (packets pushed into _auto_inbox) ANY time.
# Fixes the node v26 "ResetStdio errno 9 (EBADF)" crash by fully detaching stdio:
#   setsid + stdin</dev/null + append-only log  (no dying-shell fd).
# RUN ONCE in Termux (operator step; adb cannot exec Termux node):
#   nohup bash /sdcard/Asolaria/omnicoder/falcon-omnicoder-persistent.sh >/dev/null 2>&1 &
# or drop into ~/.termux/boot/ for auto-start. Then acer remote-controls via file pushes.
DIR=/sdcard/Asolaria/omnicoder
NODE=/data/data/com.termux/files/usr/bin/node
SERVER="$DIR/omnicoder-server-v2.mjs"
LOG="$DIR/omnicoder-persistent.log"
PIDF="$DIR/omnicoder.pid"
echo "[watchdog] start $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$LOG"
while true; do
  P="$(cat "$PIDF" 2>/dev/null)"
  if [ -n "$P" ] && kill -0 "$P" 2>/dev/null; then sleep 20; continue; fi
  echo "[watchdog] (re)launch omnicoder $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$LOG"
  setsid "$NODE" "$SERVER" </dev/null >> "$LOG" 2>&1 &
  echo $! > "$PIDF"
  sleep 20
done