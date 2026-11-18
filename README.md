# Cabecada

Note: this build does not contain any of the original game's files; its art, sounds and name are its own (see CREDITS.md).

A 1 v 1 big-head football game that works fully offline:

- **Contra o computador** - three difficulty levels.
- **2 jogadores neste aparelho** - both players on one screen (each gets their own buttons).
- **Criar partida / Entrar em partida (rede local)** - one player's device hosts, the other joins over the same Wi-Fi or the host's phone hotspot. No internet and no server.
- **Poderes**: Fogo, Bola gigante, Gelo, Invisivel, Bomba and Clones (pick one in the menu, or Aleatorio). See MECHANICS.md.

Where the source code is and what each file does: see CODIGO.md (in Portuguese).

## Controls
Touch: arrows bottom-left; CHUTE (kick), ALTO (high kick), PULO (jump), PODER (power) bottom-right.
Keyboard: A/D or arrows to move, W / Up / Space to jump, F / J / K to kick, G / L high kick, R / O power. With two players on one keyboard: player 1 A D W F G R, player 2 arrows K L O.

## Building
- Windows: `cargo build --release`.
- Android: with cargo-quad-apk (`cargo quad-apk build --release`); the Android settings and network permission are in Cargo.toml.

Options for checks: `--screenshot file.png`, `--self-test file.json`.
