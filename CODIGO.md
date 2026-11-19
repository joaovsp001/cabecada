# Onde fica o codigo (codigo-fonte, antes de compilar)

Pasta do projeto: a pasta `app` deste repositorio.

Tudo esta em texto puro, na linguagem Rust. Da para abrir qualquer arquivo com o Bloco de Notas ou o VS Code.

| Arquivo | O que tem dentro |
|---|---|
| `src/main.rs` | O comeco do jogo: telas de rede local, desenho do campo, jogadores, bola, placar, pausa e botoes de toque (`touch_layout` diz onde ficam) |
| `src/ui.rs` | As telas de menu: menu principal, multijogador, preparacao da partida (personagem, poder, estadio, tempo), configuracoes, teste dos botoes e as salas online |
| `src/settings.rs` | As configuracoes que ficam salvas no aparelho (volumes, botoes, ultimas escolhas) |
| `src/relay.rs` | O jogo pela internet: conversa com um servidor MQTT publico e gratuito usando o codigo da sala |
| `src/music.rs` | A musica do menu (feita por codigo, ou `assets/music/menu.ogg` se existir) |
| `src/game.rs` | As regras e a fisica: andar, pular, chutes, bola, gols, tempo, os 6 poderes e os 6 personagens com atributos (`CHARS`); os numeros ficam no topo do arquivo |
| `src/ai.rs` | O adversario do computador (3 niveis) |
| `src/net.rs` | O jogo em rede sem servidor: criar partida, achar partidas no Wi-Fi e entrar; pela internet usa o `relay.rs` |
| `src/audio.rs` | Os sons, criados pelo proprio codigo |
| `src/assets.rs` | Carrega as imagens da pasta `assets/` para dentro do jogo |
| `src/selftest.rs` | O teste automatico que joga sozinho e confere se tudo funciona |
| `src/win.rs` | Um detalhe do Windows usado so nos testes automaticos |
| `src/platform.rs` | O que muda no celular: log do Android, fechar o app pelo botao voltar, saber quando o app foi para segundo plano |
| `assets/` | As imagens (estadio, cabecas, chuteiras, bola, botoes) |
| `assets/ui/` e `assets/music/` | Imagens e musica opcionais: e so colocar o arquivo com o nome certo (lista em `PARA_GEMINI_2.md`) que o jogo usa sozinho |
| `android/res/` | O icone do app no celular (em todos os tamanhos do Android) |
| `android/MainActivityInject.java` e `quad.toml` | Codigo Java extra do app Android: tela cheia de verdade e busca de partidas na rede Wi-Fi |
| `.github/workflows/` | A compilacao do APK na nuvem do GitHub e o teste dele num emulador de Android |
| `PARA_GEMINI_2.md` | As imagens novas que faltam (botoes da partida, icones, painel) |
| `PARA_GEMINI.md` | A lista da rodada anterior (imagens e ideias para o jogo) |
| `Cargo.toml` | A "receita" do projeto: nome, bibliotecas usadas e configuracao para Android |
| `build.rs` | Um ajuste para compilar neste computador e a busca das imagens e musicas opcionais |

## Como compilar
- Windows: na pasta `app`, rode `cargo build --release`. O jogo fica em `target\release\cabecada.exe`.
- Android, pela nuvem (mais facil): no GitHub, aba **Actions** > **Android APK** > **Run workflow**. Em uns 5 minutos o APK aparece em **Artifacts** (`cabecada-apk`). Depois disso o **Android emulator test** roda sozinho e guarda fotos e o log do jogo num emulador.
- Android, neste computador: com o `cargo-quad-apk` instalado, rode `cargo quad-apk build --release`. O arquivo `.apk` fica em `target\android-artifacts\release\apk`.
- Cuidado no `Cargo.toml`: o `package_name` do Android tem que terminar em `.cabecada`, senao o app fecha assim que abre.

A pasta `target` e so o resultado da compilacao; ela pode ser apagada e e criada de novo ao compilar.

## Como conferir depois de mudar algo
- `target\release\cabecada.exe --self-test saida.json`: o jogo joga sozinho; todas as etapas do arquivo `saida.json` devem dar `"passed": true`.
- `target\release\cabecada.exe --shots pasta`: salva uma foto de cada tela (menu, preparacao, configuracoes, partida, gol, pausa, cada poder...) para voce ver como ficou. Com `--size 1600x720` mostra um celular comprido; com `--only menu,settings` so algumas telas.
- `target\release\cabecada.exe --online-test`: cria uma sala online de verdade e entra nela pelo mesmo PC (precisa de internet) e mostra o atraso.
- `cargo test`: testes pequenos (arquivo de configuracoes, codigos de sala).
