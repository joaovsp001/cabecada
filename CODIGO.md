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
| `assets/` | As imagens (estadio, cabecas, chuteiras, bola, botoes) |
| `PARA_GEMINI.md` | O que ainda falta fazer (imagens para refazer ou criar, e funcoes para completar o jogo) |
| `Cargo.toml` | A "receita" do projeto: nome, bibliotecas usadas e configuracao para Android |
| `build.rs` | Um ajuste para compilar neste computador |

## Como compilar
- Windows: na pasta `app`, rode `cargo build --release`. O jogo fica em `target\release\cabecada.exe`.
- Android: com o `cargo-quad-apk` instalado, rode `cargo quad-apk build --release`. O arquivo `.apk` fica em `target\android-artifacts\release\apk`.

A pasta `target` e so o resultado da compilacao; ela pode ser apagada e e criada de novo ao compilar.

## Como conferir depois de mudar algo
- `target\release\cabecada.exe --self-test saida.json`: o jogo joga sozinho; todas as etapas do arquivo `saida.json` devem dar `"passed": true`.
- `target\release\cabecada.exe --shots pasta`: salva uma foto de cada tela (menu, partida, gol, pausa, cada poder...) para voce ver como ficou.
