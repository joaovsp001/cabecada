# Cabecada

Note: this build does not contain any of the original game's files; its art, sounds and name are its own (see CREDITS.md).

A 1 v 1 big-head football game. Pick a mode in the menu, then choose everything for the match on one setup screen (character, power, opponent, stadium, time, computer level) and play:

- **Jogar** - against the computer, three difficulty levels.
- **Campeonato** - a three-round cup against the computer (Quartas, Semifinal, Final), each round harder; win all three to be champion.
- **2 jogadores** - both players on one screen (each gets their own row of buttons).
- **Multijogador**
  - *Na mesma rede Wi-Fi*: one device hosts, the other joins over the same Wi-Fi or the host's phone hotspot. No internet and no server.
  - *Pela internet*: one player creates a room and sends the 6-digit code; the other types it in. The packets go through a free public MQTT broker (`src/relay.rs`), so it works between different networks with no server of our own. Whoever joins sees the match with some delay (about 0.2 s each way from Brazil; the brokers are in Europe and the US); the host plays without delay.
- **Personagens**: Rafa, Zeca, Dudu, Lia, Ruivo and Rei, each a little different in speed, jump, kick and power charge (`CHARS` in `src/game.rs`).
- **Poderes**: Fogo, Bola gigante, Gelo, Invisível, Bomba and Clones (or Aleatório). See MECHANICS.md.
- **Gol de ouro**: a tied match goes to overtime and the next goal wins.
- **Configurações** (the gear on the menu, also in the pause box): sound effects and music volume, size and visibility of the on-screen buttons, when to show them, and a button test. Saved on the device (`settings.txt` in the app's own folder; `%APPDATA%\Cabecada` on Windows).

Where the source code is and what each file does: see CODIGO.md (in Portuguese). What is still missing (art and code) is listed for the next person in PARA_GEMINI.md.

## Controls
The match is framed like Head Ball 2: the field fills the whole screen width (on any phone shape) and ends at about 80% of the height; below it a dark band holds the buttons. Touch (phones, or after the first touch): the two arrows near the left corner, kick, high kick and jump near the right corner, the power button in the top-right corner (it fills up as the power charges) and pause in the top-left corner. Sizes and distances were measured on screenshots of the original (`touch_layout` in `src/main.rs`). In Configurações > Mover botões each button can be dragged anywhere (saved); size and visibility are there too. New button pictures can be dropped into `assets/ui/` (see PARA_GEMINI_2.md).
Keyboard: A/D or arrows to move, W / Up / Space to jump, F / J / K to kick, G / L high kick, R / O power, P or Esc to pause, T to show or hide the on-screen buttons. With two players on one keyboard: player 1 A D W F G R, player 2 arrows K L O.

## Building
- Windows: `cargo build --release`.
- Android, in the cloud (easiest): on GitHub open the **Actions** tab, pick **Android APK**, click **Run workflow**, and when it ends (about 5 minutes) download `cabecada-apk` from the run's **Artifacts**. Install the `.apk` on the phone (allow "install unknown apps"). The workflow is `.github/workflows/android.yml`; it uses the `notfl3/cargo-apk` image with the latest stable Rust and a version 3 `Cargo.lock` (the cargo inside cargo-quad-apk does not read version 4).
- After each APK build, **Android emulator test** (`.github/workflows/android-test.yml` and `.github/android-smoke.sh`) installs it on Android 11 and Android 14 emulators, opens the game, starts a match, tries the back and home buttons and keeps screenshots and the log in the run's artifacts (`android-smoke-api30`, `android-smoke-api34`).
- Android, on your own computer: with cargo-quad-apk (`cargo quad-apk build --release`), the Android SDK and NDK r25.
- Android settings live in `Cargo.toml` (`[package.metadata.android]`). Things that matter:
  - `package_name` must end in `.cabecada` (the crate name): cargo-quad-apk loads the library named after its last part. With another ending the app closes as soon as it opens.
  - No `orientation` key (cargo-quad-apk rejects it); landscape comes from `activity_attributes`, which also sets `android:exported` (required by Android 12+).
  - The launcher icon is in `android/res` (made from `assets/icon_512.png` and `assets/icon_adaptive_fg.png`).
  - `quad.toml` adds the Java code in `android/MainActivityInject.java` to the app: real full screen (system bars hidden) and a Wi-Fi multicast lock so phones find matches on the local network.
- On the phone the back button pauses a match, leaves the lobbies and, on the menu, asks before closing the game; a match pauses by itself when the app goes to the background.
- Signing: with the repository secrets `ANDROID_KEYSTORE_BASE64` (the key file in base64) and `ANDROID_KEYSTORE_PASSWORD`, the workflow signs every APK with the same key, so a new APK installs over the old one; the version code is the run number, so it always goes up. Without the secrets the APK keeps a throw-away debug signature (then uninstall before installing a new one). Keep a private backup of the key: if it is lost, installed copies can no longer be updated.

Options for checks:
- `--self-test file.json`: the game plays itself and checks each part (10 steps, all must pass).
- `--shots folder`: a picture of every screen and power. Add `--only menu,settings` for some of them and `--size 1600x720` for another screen shape (wide phones, tablets).
- `--online-test file.json`: creates an online room through the internet broker and joins it from the same PC, and reports the delay. Needs the internet, so it is not part of the self-test.
- `--screenshot file.png`: one match picture.
- `cargo test`: small unit tests (settings file, room codes).

Text uses `assets/fonts/game.ttf` if present, otherwise a bold font that ships with Windows; with neither, the built-in pixel font is used (without accents).
