#!/usr/bin/env bash
# Runs on the GitHub Android emulator: installs the APK, opens the game, plays a bit,
# tries the back and home buttons, and keeps screenshots and the log (logcat).
# Results go to the "smoke" folder, which the workflow uploads.
set -u
OUT=smoke
mkdir -p "$OUT"
APK=$(ls apk/*.apk | head -n 1)
PKG=$(aapt2 dump packagename "$APK" 2>/dev/null || true)
if [ -z "$PKG" ]; then
  BT=$(ls -d "$ANDROID_HOME"/build-tools/* | sort -V | tail -n 1)
  PKG=$("$BT/aapt2" dump packagename "$APK")
fi
echo "APK: $APK  package: $PKG"

adb shell getprop ro.build.version.release | tee "$OUT/android_version.txt"
adb shell getprop ro.product.cpu.abilist | tee "$OUT/abilist.txt"
adb shell wm size | tee "$OUT/wm_size.txt"
adb shell wm density | tee -a "$OUT/wm_size.txt"

adb install -r "$APK" 2>&1 | tee "$OUT/install.txt"
adb logcat -c
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 2>&1 | tail -n 2 | tee "$OUT/start.txt"

alive() { echo "$1: pid '$(adb shell pidof "$PKG" | tr -d '\r')'" | tee -a "$OUT/alive.txt"; }
shot() { adb exec-out screencap -p > "$OUT/$1.png"; }

sleep 15
shot 01_menu
alive "menu, 15 s after start"

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
key() { echo "key $1"; adb shell input keyevent "$1"; }

tap 640 313                # "Contra o computador"
sleep 4
shot 02_match_start
sleep 9
shot 03_match_playing
alive "during the match"

key KEYCODE_BACK           # pause
sleep 2
shot 04_back_pauses
key KEYCODE_BACK           # back again: continue
sleep 2
shot 05_back_again_continues

key KEYCODE_HOME           # leave the app...
sleep 3
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 > /dev/null 2>&1   # ...and come back
sleep 4
shot 06_back_from_home_paused
alive "after home and back"

tap 640 447                # "Sair para o menu"
sleep 3
shot 07_menu_again
key KEYCODE_BACK           # "Sair do jogo?" box
sleep 2
shot 08_quit_box
tap 755 428                # "Continuar"
sleep 2
shot 09_menu_after_cancel
alive "end"

adb logcat -d -v threadtime > "$OUT/logcat.txt"
echo "===== relevant log lines ====="
grep -iE "cabecada|futebol|AndroidRuntime|FATAL|DEBUG +:|libc +:|SAPP|panicked|Process .* has died|Start proc|UnsatisfiedLink|linker" "$OUT/logcat.txt" | tail -n 300
exit 0
