# Cabecada - mechanics sheet

Units: virtual pixels on a 1280 x 720 field, seconds. Values live in `src/game.rs`.

## What came from the original program
Evidence `ev_81d7c9451d939569b7b50eae7291f7a9f848db260d1f393ab5ca08f1b819f407` (strings in the game's native library, libHeadBall2.so):
- Controls in a match: move left / right, jump (`jump_button`), low kick (`low_kick_button`, kick1) and high kick (`high_kick_button`, kick2), plus a super-power button.
- Kicking the opponent is a thing (`sfx_kicking_opponent`), so a kick that hits the other player knocks them down here.
- Player stats are jump / size / shoot / speed / power; computer players are tuned by move / kick / scale / jump factors and mistake / false-decision frequencies (here: three difficulty levels with reaction time and mistake chance).
- Gravity is read from `gravityx` / `gravityy` keys (looks like server-provided settings), so the physics numbers are not in the program as named constants. Every number below is therefore **estimated** (open unknown `unk_efd79da2cc750bca`).

## Movement (estimated)
| value | number |
|---|---|
| run speed | 340 px/s |
| ground acceleration | 4200 px/s² |
| air acceleration | 2600 px/s² |
| player gravity | 1900 px/s² |
| jump take-off speed | 830 px/s (about 181 px high, 0.87 s in the air) |
| head radius | 44 px; body circle 24 px, 38 px below the head |
| players can stand on each other's head and on the crossbars | yes |

## Ball (estimated)
| value | number |
|---|---|
| radius | 21 px |
| gravity | 1250 px/s² |
| air drag | 12 % per second |
| bounce on grass / walls / crossbar | 0.70 / 0.80 / 0.75 |
| bounce off a head | 0.62 + 90 px/s push |
| top speed | 1750 px/s |

## Kicks (estimated)
| value | number |
|---|---|
| kick swing | 0.20 s, cooldown 0.33 s |
| low kick | 980 px/s, forward 0.85 / up 0.53 |
| high kick | 1080 px/s, forward 0.52 / up 0.85 |
| share of own speed added | 25 % |
| kicking the opponent | knocks them down 0.9 s |

## Super power (estimated)
- The bar fills in 20 s of play, +5 % for each touch of the ball.
- Full bar + power button: the next kick within 6 s is a power shot at 1550 px/s straight at the far goal, ignoring gravity for 1.1 s; it knocks down the defender it hits (1.6 s) and keeps going.

## Match (estimated)
- 1 v 1, goals at both ends (goal line 86 px from the wall, crossbar top at 388 px, ground at 620 px).
- Match length 90 s (60 / 120 selectable); 3 s countdown; 2.2 s goal celebration; ball dropped at the centre after each goal.
- Most goals when time runs out wins; equal is a draw.

## Computer player
Three levels: reaction 0.26 / 0.14 / 0.06 s, mistake chance 35 / 15 / 4 %. It stands behind the ball to push it forward, runs back when the ball is behind it or flying at its goal, jumps to head dropping balls, kicks balls at its feet, sometimes kicks the opponent, uses the power when full and close.

## Local-network play (instead of the online servers)
- One device hosts (runs the match), the other joins. No internet, no server: UDP on port 47777, discovery on 47778 by broadcast; the joiner can also type the host's address.
- Host sends the match state 60 times a second; the joiner sends its buttons every frame. 5 s without packets = disconnected.
