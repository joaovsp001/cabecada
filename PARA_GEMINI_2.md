# Para o Gemini: imagens novas (rodada 2)

Este arquivo é a lista **atual**. O `PARA_GEMINI.md` é da rodada anterior (muita coisa de lá já foi feita).

O que você precisa fazer aqui: **criar as imagens da seção 3 e colocar na pasta `assets/ui/`**. Não precisa mudar código:
o jogo acha os arquivos sozinho pelo nome (o `build.rs` embute tudo o que estiver em `assets/ui/` ao compilar).
Enquanto uma imagem não existe, o jogo desenha uma versão simples por código, então dá para fazer aos poucos.

A arte tem que ser **original**. Só a posição e o tamanho dos botões imitam o Head Ball 2; não copie imagens dele.

## 1. O que mudou no jogo nesta rodada (para você se situar)

- **Menu novo**: JOGAR, CAMPEONATO, 2 JOGADORES, MULTIJOGADOR e uma engrenagem (configurações) no canto.
- **Preparação da partida**: depois de escolher o modo, abre uma tela onde se escolhe tudo antes de jogar:
  personagem (com nome e atributos), poder, adversário, estádio, tempo e nível do computador (`src/ui.rs`, `setup_screen`).
- **Personagens com atributos**: Rafa, Zeca, Dudu, Lia, Ruivo e Rei (`CHARS` em `src/game.rs`), na ordem das imagens `heads`/`boots`.
  Cada um tem velocidade, pulo, chute e carga do poder um pouco diferentes.
- **Configurações** (salvas no aparelho): volume dos efeitos, volume da música, tamanho dos botões (70% a 150%),
  visibilidade dos botões, botões na tela (automático, sempre, nunca) e um teste dos botões (`settings_screen`, `button_test`).
- **Botões da partida no lugar do Head Ball 2**: uma fileira só embaixo, setas à esquerda e chute, chute alto, pulo e poder
  à direita, medidos na tela do original (`touch_layout` em `src/main.rs`). Eles ficam presos às bordas reais da tela,
  então funcionam em celular comprido e em tablet.
- **Multijogador** com duas opções: mesma rede Wi-Fi (como antes) e **pela internet** com código de sala de 6 números
  (`src/relay.rs`, usa um servidor MQTT público e gratuito).
- **Música de menu** feita por código (`src/music.rs`).

## 2. Como testar (faça antes de enviar)

Na pasta `app`:

```bash
cargo build --release
target\release\cabecada.exe --self-test saida.json
target\release\cabecada.exe --shots fotos
target\release\cabecada.exe --shots fotos_largo --size 1600x720 --only menu,button_test,play_touch,play_local2_touch
```

1. Em `saida.json` as 10 etapas devem estar com `"passed": true`.
2. Em `fotos` confira as imagens novas. As fotos que mostram cada uma:
   - botões da partida (`ctrl_*`): `*_button_test.png`, `*_play_touch.png`, `*_play_local2_touch.png`
   - `icon_settings`: `*_menu.png`; `icon_wifi` e `icon_online`: `*_multi.png`
   - `icon_prev`, `icon_next`, `power_random`: `*_setup_cpu.png` (e as outras `*_setup_*.png`)
   - `panel`: `*_settings.png`, `*_setup_*.png`, `*_multi.png`, `*_online_host.png`, `*_online_join.png`
3. `fotos_largo` mostra um celular comprido (20:9): os botões devem continuar alinhados na parte de baixo.

Ao terminar: commit na `main` **com `[skip ci]` no fim da mensagem** e push. O Claude confere tudo e compila o APK depois
(sem o `[skip ci]` o GitHub já compila sozinho, o que também funciona, mas sem a conferência).

## 3. Imagens para criar (todas PNG com fundo transparente, em `assets/ui/`)

### 3.1 Botões da partida (os mais importantes)

| Arquivo | Tamanho | Uso |
|---|---|---|
| `ctrl_left.png` | 290x200 | Andar para a esquerda (seta) |
| `ctrl_right.png` | 290x200 | Andar para a direita. Opcional: sem ele o jogo espelha o `ctrl_left.png` |
| `ctrl_kick.png` | 290x200 | Chute (chuteira chutando para a frente) |
| `ctrl_lob.png` | 290x200 | Chute alto (chuteira chutando a bola para cima) |
| `ctrl_jump.png` | 290x200 | Pulo (seta grossa para cima, ou chuteira com asinhas) |

- O jogo desenha cada um em **133 x 92** numa tela de 720 de altura (um pouco mais largo que alto, como no original),
  vezes o tamanho escolhido nas configurações. Mantenha a proporção 290x200.
- Estilo sugerido: retângulo com cantos bem arredondados (raio de uns 45 px na imagem), fundo azul-escuro meio
  transparente (uns 55% de opacidade), borda clara de 6 a 8 px e um **ícone branco grande** no centro, com uma
  sombra leve. Os cinco no mesmo estilo. Sem texto e sem números.
- O jogo ainda aplica a transparência escolhida pelo jogador por cima, então não precisa ser muito transparente.
- No modo 2 jogadores o jogador 2 usa os botões de chute, chute alto e pulo **espelhados**; desenhe-os de um jeito
  que fique bom espelhado (a chuteira do chute aponta para a direita).
- O botão de **poder** continua sendo os `btn_power_*.png` que já existem (quadrados, um para cada poder); não precisa refazer.

### 3.2 Menus

| Arquivo | Tamanho | Uso |
|---|---|---|
| `icon_settings.png` | 128x128 | Botão de configurações no canto de cima do menu, desenhado em 76x76. É o **botão inteiro**: círculo (azul, com borda clara) e engrenagem branca |
| `icon_prev.png` | 128x128 | Botão redondo com seta para a esquerda (trocar personagem, adversário e estádio), desenhado em 68x68 e em 48x48. Botão inteiro |
| `icon_next.png` | 128x128 | O mesmo com a seta para a direita |
| `icon_wifi.png` | 256x256 | Ícone do painel "Na mesma rede Wi-Fi" (sinal de Wi-Fi, ou dois celulares lado a lado), desenhado em 110x110 |
| `icon_online.png` | 256x256 | Ícone do painel "Pela internet" (globo, ou dois celulares ligados por uma nuvem), desenhado em 110x110 |
| `power_random.png` | 160x160 | Opção "Aleatório" na escolha de poder, **no mesmo estilo dos `btn_power_*.png`** (quadrado verde arredondado), com um "?" grande ou um dado |
| `panel.png` | 192x192 | Fundo de todos os painéis (configurações, preparação, multijogador, salas online) |

Sobre o `panel.png`: ele é esticado em 9 partes. Os **32 px de cada canto** ficam como estão e o resto estica, então os
cantos arredondados e a borda precisam caber nesses 32 px. Sugestão: azul-marinho escuro com uns 80% de opacidade,
borda fina clara (2 a 3 px) e um brilho leve no topo. Tem que deixar o texto branco bem legível por cima.

### 3.3 Extras opcionais (rodada 3)

| Arquivo | Tamanho | Uso |
|---|---|---|
| `icon_pause.png` | 128x128 | Botão de pausa redondo no canto de cima, à esquerda, na partida (desenhado em 64x64). Botão inteiro, no estilo do `icon_settings.png` |
| `band.png` | 1600x200 | A faixa escura embaixo do campo, onde ficam os botões (esticada na largura da tela). Terra/grama escura, sem desenhos no meio, que não chame atenção |

### 3.4 Música (opcional)

- `assets/music/menu.ogg` (ou `.wav`): música do menu, de 30 a 90 segundos, que faça **loop sem corte**, alegre, estilo
  jogo de futebol. Só use se for **criada por você** ou com licença livre (CC0). Se não der, deixe: o jogo já cria uma
  música por código.

## 4. Código (só se quiser melhorar; não é obrigatório)

- Posições e tamanhos dos botões da partida: `touch_layout` em `src/main.rs` (frações da largura medidas no original:
  setas em 18,4% e 34,2%, chute em 56,1%, chute alto em 67,6%, pulo em 78,9%, poder em 90,5%). Não mude sem motivo.
- Desenho dos botões: `draw_buttons_for` em `src/main.rs`. Telas do menu: `src/ui.rs`.
- Se mudar o que vai pela rede (`to_bytes`/`from_bytes` em `src/game.rs`), rode o `--self-test` (etapa `lan`) e também
  `target\release\cabecada.exe --online-test` (precisa de internet: cria uma sala de verdade e entra nela).
- Não apague o `--self-test`, o `--shots` nem o `--online-test`.
