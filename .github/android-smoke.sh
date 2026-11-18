#!/usr/bin/env bash
# Runs on the GitHub Android emulator: installs the APK, opens the game, taps
# "Contra o computador", takes screenshots and saves the log (logcat).
# Results go to the "smoke" folder, which the workflow uploads.
set -u
OUT=smoke
mkdir -p "$OUT"
APK=$(ls apk/*.apk | head -n 1)
PKG=local.cabecada.futebol
echo "APK: $APK"

adb shell getprop ro.build.version.release | tee "$OUT/android_version.txt"
adb shell getprop ro.product.cpu.abilist | tee "$OUT/abilist.txt"
adb shell wm size | tee "$OUT/wm_size.txt"
adb shell wm density | tee -a "$OUT/wm_size.txt"

adb install -r "$APK" 2>&1 | tee "$OUT/install.txt"
adb logcat -c
adb shell am start -W -n "$PKG/.MainActivity" 2>&1 | tee "$OUT/start.txt"

sleep 15
adb exec-out screencap -p > "$OUT/01_menu.png"
echo "running after 15 s (pid): '$(adb shell pidof $PKG | tr -d '\r')'" | tee "$OUT/alive.txt"

# The game draws a 1280x720 picture, scaled to fit and centred: turn a point of that
# picture into screen pixels and tap it.
SIZE=$(adb shell wm size | sed -n 's/.*: *\([0-9]*\)x\([0-9]*\).*/\1 \2/p' | tail -n 1)
read -r A B <<< "$SIZE"
if [ "$A" -gt "$B" ]; then W=$A; H=$B; else W=$B; H=$A; fi
tap() {
  local xy
  xy=$(awk -v vx="$1" -v vy="$2" -v w="$W" -v h="$H" 'BEGIN {
    s = w / 1280; if (h / 720 < s) s = h / 720;
    ox = (w - 1280 * s) / 2; oy = (h - 720 * s) / 2;
    printf "%d %d", ox + vx * s, oy + vy * s }')
  echo "tap $1,$2 -> $xy"
  adb shell input tap $xy
}

tap 640 313      # "Contra o computador"
sleep 6
adb exec-out screencap -p > "$OUT/02_match_start.png"
sleep 8
adb exec-out screencap -p > "$OUT/03_match_playing.png"
echo "running after the match started (pid): '$(adb shell pidof $PKG | tr -d '\r')'" | tee -a "$OUT/alive.txt"

adb logcat -d -v threadtime > "$OUT/logcat.txt"
echo "===== relevant log lines ====="
grep -iE "cabecada|AndroidRuntime|FATAL|DEBUG +:|libc +:|quad|miniquad|panicked|SAPP|Process .* has died|ActivityManager: Start proc|linker" "$OUT/logcat.txt" | tail -n 400
exit 0
