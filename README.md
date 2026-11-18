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
- Android: with cargo-quad-apk (`cargo quad-apk build --release`); the Android settings and network permission are in Cargo.toml.

Options for checks: `--self-test file.json` (the game plays itself and checks each part), `--shots folder` (a picture of every screen and power), `--screenshot file.png` (one match picture).

Text uses `assets/fonts/game.ttf` if present, otherwise a bold font that ships with Windows; with neither, the built-in pixel font is used (without accents).
