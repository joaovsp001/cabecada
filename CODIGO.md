# Onde fica o codigo (codigo-fonte, antes de compilar)

Pasta do projeto: a pasta `app` deste repositorio.

Tudo esta em texto puro, na linguagem Rust. Da para abrir qualquer arquivo com o Bloco de Notas ou o VS Code.

| Arquivo | O que tem dentro |
|---|---|
| `src/main.rs` | O comeco do jogo: menus, telas de rede, desenho do campo, jogadores, bola, placar e botoes de toque |
| `src/game.rs` | As regras e a fisica: andar, pular, chutes, bola, gols, tempo, e os 6 poderes (todos os numeros ficam no topo do arquivo) |
| `src/ai.rs` | O adversario do computador (3 niveis) |
| `src/net.rs` | O jogo em rede local sem servidor: criar partida, achar partidas na rede e entrar |
| `src/audio.rs` | Os sons, criados pelo proprio codigo |
| `src/assets.rs` | Carrega as imagens da pasta `assets/` para dentro do jogo |
| `src/selftest.rs` | O teste automatico que joga sozinho e confere se tudo funciona |
| `src/win.rs` | Um detalhe do Windows usado so nos testes automaticos |
| `src/platform.rs` | O que muda no celular: log do Android, fechar o app pelo botao voltar, saber quando o app foi para segundo plano |
| `assets/` | As imagens (estadio, cabecas, chuteiras, bola, botoes) |
| `android/res/` | O icone do app no celular (em todos os tamanhos do Android) |
| `android/MainActivityInject.java` e `quad.toml` | Codigo Java extra do app Android: tela cheia de verdade e busca de partidas na rede Wi-Fi |
| `.github/workflows/` | A compilacao do APK na nuvem do GitHub e o teste dele num emulador de Android |
| `PARA_GEMINI.md` | O que ainda falta fazer (imagens para refazer ou criar, e funcoes para completar o jogo) |
| `Cargo.toml` | A "receita" do projeto: nome, bibliotecas usadas e configuracao para Android |
| `build.rs` | Um ajuste para compilar neste computador |

## Como compilar
- Windows: na pasta `app`, rode `cargo build --release`. O jogo fica em `target\release\cabecada.exe`.
- Android, pela nuvem (mais facil): no GitHub, aba **Actions** > **Android APK** > **Run workflow**. Em uns 5 minutos o APK aparece em **Artifacts** (`cabecada-apk`). Depois disso o **Android emulator test** roda sozinho e guarda fotos e o log do jogo num emulador.
- Android, neste computador: com o `cargo-quad-apk` instalado, rode `cargo quad-apk build --release`. O arquivo `.apk` fica em `target\android-artifacts\release\apk`.
- Cuidado no `Cargo.toml`: o `package_name` do Android tem que terminar em `.cabecada`, senao o app fecha assim que abre.

A pasta `target` e so o resultado da compilacao; ela pode ser apagada e e criada de novo ao compilar.

## Como conferir depois de mudar algo
- `target\release\cabecada.exe --self-test saida.json`: o jogo joga sozinho; todas as etapas do arquivo `saida.json` devem dar `"passed": true`.
- `target\release\cabecada.exe --shots pasta`: salva uma foto de cada tela (menu, partida, gol, pausa, cada poder...) para voce ver como ficou.
