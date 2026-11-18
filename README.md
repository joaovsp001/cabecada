# Cabecada

Note: this build does not contain any of the original game's files; its art, sounds and name are its own (see CREDITS.md).

A 1 v 1 big-head football game that works fully offline:

- **Contra o computador** - three difficulty levels.
- **2 jogadores neste aparelho** - both players on one screen (each gets their own buttons).
- **Criar partida / Entrar em partida (rede local)** - one player's device hosts, the other joins over the same Wi-Fi or the host's phone hotspot. No internet and no server.
- **Campeonato** - a three-round cup against the computer (Quartas, Semifinal, Final), each round harder; win all three to be champion.
- **Poderes**: Fogo, Bola gigante, Gelo, Invisível, Bomba and Clones (pick one in the menu, or Aleatório). See MECHANICS.md.
- **Gol de ouro**: a tied match goes to overtime and the next goal wins.

Where the source code is and what each file does: see CODIGO.md (in Portuguese). What is still missing (art and code) is listed for the next person in PARA_GEMINI.md.

## Controls
Touch (phones, or after the first touch): arrows bottom-left; CHUTE (kick), ALTO (high kick), PULO (jump), PODER (power) bottom-right.
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

Options for checks: `--self-test file.json` (the game plays itself and checks each part), `--shots folder` (a picture of every screen and power), `--screenshot file.png` (one match picture).

Text uses `assets/fonts/game.ttf` if present, otherwise a bold font that ships with Windows; with neither, the built-in pixel font is used (without accents).
