# Cabecada - mechanics sheet

Units: virtual pixels on a 1280 x 720 field, seconds. Values live in `src/game.rs`.

## What came from the original program
Evidence `ev_81d7c9451d939569b7b50eae7291f7a9f848db260d1f393ab5ca08f1b819f407` (strings in the game's native library, libHeadBall2.so):
- Controls in a match: move left / right, jump (`jump_button`), low kick (`low_kick_button`, kick1) and high kick (`high_kick_button`, kick2), plus a super-power button.
- Kicking the opponent is a thing (`sfx_kicking_opponent`), so a kick that hits the other player knocks them down here.
- Player stats are jump / size / shoot / speed / power; computer players are tuned by move / kick / scale / jump factors and mistake / false-decision frequencies (here: three difficulty levels with reaction time and mistake chance).
- Gravity is read from `gravityx` / `gravityy` keys (looks like server-provided settings), so the physics numbers are not in the program as named constants. Every number below is therefore **estimated** (open unknown `unk_efd79da2cc750bca`) and tuned to Head Ball 2 side-view arcade proportions.

## Movement (estimated)
| value | number |
|---|---|
| run speed | 470 px/s |
| ground acceleration | 5800 px/s² |
| air acceleration | 3400 px/s² |
| player gravity | 3000 px/s² |
| jump take-off speed | 1245 px/s (about 258 px high, 0.83 s in the air; sole clears the crossbar) |
| head radius | 56 px; body circle 32 px, 52 px below the head (sole 84 px below head centre, 140 px total hitbox height) |
| foot circle & kick reach | 20 px foot radius + 22 px reach (extends 74 px forward low, 66 px high) |
| players can stand on each other's head and on the crossbars | yes |

## Ball (estimated)
| value | number |
|---|---|
| radius | 30 px (60 px diameter) |
| gravity | 1480 px/s² |
| air drag | 12 % per second |
| bounce on grass / walls / crossbar / back net | 0.74 / 0.80 / 0.75 / 0.48 |
| bounce off a head | 0.62 + 110 px/s push |
| top speed | 1950 px/s |

## Kicks (estimated)
| value | number |
|---|---|
| kick swing | 0.20 s, cooldown 0.33 s |
| low kick | 1120 px/s, forward 0.89 / up 0.45 |
| high kick | 1220 px/s, forward 0.52 / up 0.85 |
| share of own speed added | 25 % |
| kicking the opponent | knocks them down 0.9 s (340 px/s horizontal, -300 px/s pop-up) |

## Super powers (estimated)
The original ships sound effects for these powers (file names in the game package, evidence `ev_873c7191ee3537003c166561572c3aa057d3deb44a33452216b6402f987330f0`: fireball, big ball, freeze, invisible ball, bomb ball, clone ball). How each one behaves there isn't readable, so the effects below are this game's own versions.

- The bar fills in 20 s of play, +5 % for each touch of the ball (no gain while a power waits for its kick).
- Each player brings one power, chosen in the menu ("Aleatorio" picks one at random). The computer and the second local player get a random one; in network play each device picks its own.
- Full bar + power button:
  - **Fogo**: the next kick within 6 s is a shot at 1750 px/s straight at the far goal, ignoring gravity for 1.1 s; it knocks down the defender it hits (1.6 s, 620 px/s knockback) and keeps going.
  - **Bola gigante**: the ball grows to 1.85x its size (55.5 px radius, 111 px diameter) for 7 s, at once.
  - **Gelo**: instant cast — immediately encases the opponent in solid ice for 3.2 s wherever they are (even suspended in mid-air with velocity zeroed and gravity disabled, separate from stun so dizzy stars do not appear). **Button-mash breakout**: tapping any movement/jump/kick button while frozen (0.08 s cooldown) shaves 0.22 s off the freeze timer and cracks the ice down to a minimum 1.0 s freeze floor before shattering cleanly on thaw.
  - **Invisivel**: the ball disappears for the opponent for 4 s (the owner still sees a faint ball; it fades back in over the last 0.5 s; the computer only knows where it last saw it).
  - **Bomba**: the next kick arms a 2.2 s fuse; when it goes off, an opponent within 260 px is thrown 760 px/s away (shattering ice if frozen) and knocked down 1.8 s; the ball jumps up.
  - **Clones**: the next kick also sends two fake balls (one higher, one lower, matching the current ball radius) that bounce around for 4.5 s and can't score or be touched.
- A goal clears every active power effect except the big ball.

## Match (estimated)
- 1 v 1, goals at both ends (goal line 104 px from the wall, crossbar 114 px long with top at 308 px and 14 px thickness, ground at 560 px, bottom control deck below 576 px).
- Match length 90 s (60 / 120 selectable); 3 s countdown; 2.2 s goal celebration; ball dropped at the centre (y = 175 px) after each goal.
- Most goals when time runs out wins. A tie goes to **golden-goal overtime** (this game's own rule, not read from the original): players and ball go back to the centre, the clock stops at 0:00 and the next goal wins.
- **Campeonato** (also this game's own design): three matches against the computer (Quartas = easy, Semifinal = normal, Final = hard). Lose one and you are out; win all three and you are the champion.

## Computer player
Three levels: reaction 0.26 / 0.14 / 0.06 s, mistake chance 35 / 15 / 4 %. It stands behind the ball to push it forward, runs back when the ball is behind it or flying at its goal, jumps to head dropping balls, kicks balls at its feet, sometimes kicks the opponent, uses powers strategically when full, and mashes out of ice on Normal and Hard levels.

## Local-network play (instead of the online servers)
- One device hosts (runs the match), the other joins. No internet, no server: UDP on port 47777, discovery on 47778 by broadcast; the joiner can also type the host's address.
- Host sends the 190-byte match state snapshot (including all 32-bit `EV_*` event flags) 60 times a second; the joiner sends its buttons every frame. 5 s without packets = disconnected.
