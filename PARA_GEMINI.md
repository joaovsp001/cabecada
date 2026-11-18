# Para o Gemini: o que fazer no Cabecada

Este arquivo diz o que já está pronto, o que ficou fraco ou faltando nas **imagens** e o que ainda falta no **código**.
Pode mexer nas duas coisas. Leia primeiro a seção 2 (como testar) antes de alterar qualquer coisa.

## 1. Contexto rápido

- Jogo de futebol de cabeção 1 contra 1, inspirado no Head Ball 2. É feito em **Rust + macroquad**, roda no Windows e no Android.
- O jogo desenha numa tela virtual de **1280 x 720**. Tudo (menus, campo, botões) usa essas coordenadas.
- Onde está cada coisa: veja `CODIGO.md`. As regras e os números da física estão no topo de `src/game.rs` e explicados em `MECHANICS.md`.
- A arte é toda original. **Não copie imagens nem sons do Head Ball 2.**

Já funciona:
- Menu, contra o computador (3 níveis), 2 jogadores no mesmo aparelho, partida em rede local (UDP, sem servidor).
- 6 poderes: Fogo, Bola gigante, Gelo, Invisível, Bomba e Clones.
- Campeonato de 3 fases contra o computador (Quartas, Semifinal e Final), com tela de campeão.
- Gol de ouro quando o jogo termina empatado (o próximo gol vence).
- Abertura "VS" antes da partida, poeira ao pular e pousar, risco ao chutar, torcida ao fundo, sons sintetizados.
- Texto com acentos (usa uma fonte bonita do Windows se achar; veja a seção 3.2).

## 2. Como testar (obrigatório antes de enviar mudanças)

Na pasta `app`:

```bash
cargo build --release
```

1. **Autoteste.** O jogo joga sozinho e confere cada parte:
   `target\release\cabecada.exe --self-test saida.json`
   Abra `saida.json`: todas as etapas devem estar com `"passed": true` (`move`, `jump`, `act`, `goal`, `power`, `powers`, `cpu`, `overtime`, `hud`, `lan`).
2. **Fotos de todas as telas.** Use isto para **ver** o resultado das suas imagens:
   `target\release\cabecada.exe --shots pasta_das_fotos`
   Gera um PNG para cada tela: menu, salas de rede, abertura VS, partida, 2 jogadores, botões de toque, gol, gol de ouro, fim de jogo, eliminado, campeão, pausa e cada poder. A lista está em `scene_setup` e `amain` no `src/main.rs`; para ver uma tela nova, adicione ali.

Regras:
- Se mudar o que vai pela rede, atualize `to_bytes` e `from_bytes` (`src/game.rs`) e o teste `lan_test` (`src/selftest.rs`).
- Não remova o `--self-test` nem o `--shots`.
- Acentos: o texto aceita acentos quando há fonte; sem fonte o jogo remove os acentos sozinho (`fix_text`).

## 3. Imagens

### 3.0 Como trocar ou adicionar
- Todas ficam em `assets/` (PNG). Para **trocar**, mantenha o mesmo nome e a mesma proporção; o jogo redimensiona.
- Para **adicionar**: coloque o PNG em `assets/`, acrescente o campo em `struct Assets` e a linha em `load()` no `src/assets.rs` (`include_bytes!`), e desenhe com `tex_draw` ou `tex_center` no `src/main.rs`.
- Fundo transparente em tudo, menos `stadium_bg.png` e `menu_bg.png`.
- Os personagens e as chuteiras devem **olhar para a direita**: o jogo vira a imagem (flip) quando o jogador ataca para a esquerda.

Imagens atuais:

| Arquivo | Tamanho | Como o jogo usa |
|---|---|---|
| `stadium_bg.png` | 1280x720 | Fundo da partida. Pés no chão em y = 620; gol em x = 86 e x = 1194; travessão em y = 388 |
| `menu_bg.png` | 1280x720 | Fundo dos menus e da tela de campeão (o pódio fica no centro, embaixo) |
| `logo_cabecada.png` | 640x320 | Logo do menu, desenhada em 470x235 |
| `p1_head.png`, `p2_head.png` | 256x256 | Cabeça dos jogadores, desenhada em 112x112 na partida (168 no menu, 124 na abertura VS, 200 no pódio) |
| `p1_boot.png`, `p2_boot.png` | 192x128 | Chuteira que chuta (66x44) e chuteira de trás (56x38, mais escura) |
| `ball.png` | 128x128 | Bola, desenhada com 2,15 vezes o raio (45 px) |
| `ball_fire.png` | 192x192 | Bola em chamas (poder Fogo), 3,15 vezes o raio |
| `goal_left.png` | 220x340 | Gol esquerdo; o código espelha a mesma imagem para o gol direito |
| `scoreboard.png` | 480x190 | Placar no topo (desenhado em 380x104); os números ficam nas caixas azul e vermelha e o tempo na caixa branca |
| `btn_left.png` | 160x160 | Seta; o código espelha para a direita |
| `btn_kick.png`, `btn_lob.png`, `btn_jump.png` | 160x160 | Botões de toque: chute, chute alto e pulo |
| `btn_power.png` | 160x160 | Botão de poder (verde, bola de fogo) |

### 3.1 Refazer ou melhorar (ficou fraco ou é provisório)

1. **Ícones dos poderes.** `btn_power.png` mostra sempre a bola de fogo, mesmo quando o jogador escolheu outro poder. Criar **6 ícones** 160x160, no mesmo estilo (botão quadrado verde): `btn_power_fire`, `btn_power_big`, `btn_power_ice`, `btn_power_invis`, `btn_power_bomb`, `btn_power_clone`. Código: `draw_touch_buttons` (`Btn::Power`), escolhendo pelo `pkind` do jogador (`PK_FIRE`..`PK_CLONE` em `game.rs`).
2. **Bloco de gelo.** Quando um jogador congela, o código desenha um retângulo azul transparente (`draw_player`, trecho `frozen`). Criar `ice_block.png` (cerca de 160x200, azul claro com brilho e rachaduras) e, se der, 4 quadros de gelo quebrando para quando ele descongela.
3. **Bola de gelo e bola bomba.** Hoje são o `ball.png` pintado de azul e de vermelho piscando, e o pavio é uma linha desenhada por código (`draw_ball`). Criar `ball_ice.png` e `ball_bomb.png` (128x128, a bomba com pavio aceso) e, para a explosão, 8 quadros `boom_00..07.png` (256x256). A explosão acontece quando `EV_BOOM` dispara.
4. **Expressões das cabeças.** Cada personagem tem uma expressão só. O jogo gira e pinta a cabeça para fingir tontura. Criar variações 256x256 de cada cabeça: atordoado (olhos em espiral), comemorando (gol) e triste (derrota). Código: `draw_player`.
5. **Botões do menu.** Os botões dos menus são retângulos arredondados feitos por código (`button` em `main.rs`) e não combinam com os botões brilhantes de jogo. Criar botões em 9 fatias (cantos de 32 px, por exemplo 256x96) nas cores verde, azul, laranja, roxo, dourado, vermelho e cinza.
6. **Efeitos.** A poeira do pulo e do pouso, o risco do chute e os confetes são círculos desenhados por código (`dust`, `burst`, `champion_screen`). O jogo original tem sprites de efeito de pulo e de chute. Criar uma folha `fx_dust.png` (6 quadros de 128x128), `fx_kick.png` (6 quadros de 128x128) e `confetti.png` (peças 32x32 variadas).
7. **Gola cortada.** Embaixo da cabeça aparece um pedaço de gola da camisa que fica estranho entre a cabeça e a chuteira. Redesenhar `p1_head` e `p2_head` sem esse pedaço **ou**, melhor, criar um personagem completo (cabeça, corpo e perna, com quadros de andar, pular e chutar). Para isso o código já calcula onde está o pé (`Player::foot`, em `game.rs`).
8. **Placar.** A caixa branca do meio é pequena para "0:00" e para "OURO". Se der, aumentar a caixa do meio.

### 3.2 O que falta criar

- **Mais personagens**: pelo menos 4 além dos 2 atuais, cada um com `pN_head.png` (256x256) e `pN_boot.png` (192x128, chuteira de cor própria). Hoje só existem o azul (jogador 1) e o vermelho (jogador 2). O campeonato precisa de adversários diferentes.
- **Troféu** do campeonato (`trophy.png`, 256x256) para a tela de campeão (`champion_screen`) e um quadro de chaveamento das 3 fases.
- **Mais estádios**: `stadium_bg2.png` e seguintes (1280x720), com outra cor ou clima, mantendo o gramado, as traves e a linha de pés nos mesmos pontos do `stadium_bg.png`.
- **Ícone do app Android**: 512x512 e a versão adaptativa (primeiro plano 432x432).
- **Fonte**: uma fonte TTF livre e arredondada (licença OFL, por exemplo Lilita One ou Fredoka), salva como `assets/fonts/game.ttf`. O jogo procura esse arquivo primeiro; sem ele, no Windows usa fontes que já vêm no sistema (Arial Rounded, Segoe UI Black), e no Android cai na fonte pixelada sem acentos. Para o Android funcionar com a fonte, ela precisa ser embutida (`include_bytes!` e `load_ttf_font_from_bytes`) ou entrar nos assets do APK (veja `load_game_font` em `main.rs`).
- **Sons** (se o usuário tiver ou você conseguir gerar): gritos de chute, pouso na grama, torcida comemorando, música de menu. Hoje todos os sons são sintetizados por código em `src/audio.rs` e são bem simples.

## 4. Código: o que ainda falta para ficar parecido com o original

Estas são ideias tiradas das strings do jogo original (veja `MECHANICS.md`). Ordem sugerida:

1. **Personagens com atributos.** O original tem atributos por jogador (pulo, tamanho, chute, velocidade e poder). Criar uma tela de escolha de personagem e fazer cada atributo multiplicar `MOVE_SPEED`, `JUMP_SPEED`, a força do chute e a velocidade de carga do poder (hoje são constantes globais em `game.rs`). O personagem escolhido precisa ir pela rede (`T_HELLO` em `net.rs` e `to_bytes` / `from_bytes`).
2. **Duelo de pênaltis** (o original tem um modo "PenaltyDuel"): como modo novo ou como desempate no lugar do gol de ouro.
3. **Personagem com corpo e perna animada** (depende da imagem do item 7 da seção 3.1).
4. **Campeonato maior**: 8 times, adversário diferente em cada fase, troféu e salvar o progresso.
5. **Tutorial** (o original tem passos de tutorial, por exemplo antes do chute alto).
6. **Salvar as configurações** (poder, nível, tempo, som, botões na tela) num arquivo.
7. **Tela de escolha de poder** mais visual, com cartões e ícones (depende do item 1 da seção 3.1).
8. **Replay** do melhor lance depois de um gol.
9. **Física**: os números de `game.rs` são **estimados** (no original a gravidade parece vir de um servidor, não de constantes). Ajustar jogando os dois lado a lado.
10. Jogo pela internet (hoje é só rede local, sem servidor).

## 5. Cuidados

- Os botões de toque só aparecem no celular ou depois do primeiro toque; no PC aparece uma linha de dicas de teclado e a tecla `T` liga e desliga os botões.
- A física roda em passo fixo de 1/120 s e o desenho em tempo real; o mesmo código roda no host e no cliente da rede (o cliente só desenha o estado recebido).
- Ao mexer em `game.rs`, rode o `--self-test` de novo.
- Imagens grandes pesam no APK: `menu_bg.png` e `stadium_bg.png` já têm cerca de 1,2 MB cada.
