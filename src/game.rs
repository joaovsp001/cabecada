//! Match simulation: two big-head players, one ball, two goals.
//! All numbers live here and are listed in MECHANICS.md / mechanics.json.
//! Units are virtual pixels on a 1280x720 field and seconds.

pub const W: f32 = 1280.0;
pub const H: f32 = 720.0;
pub const GROUND: f32 = 560.0;

pub const GOAL_DEPTH: f32 = 104.0; // goal line distance from the side wall
pub const BAR_Y: f32 = 308.0; // top of the crossbar
pub const BAR_H: f32 = 14.0; // crossbar thickness
pub const BAR_LEN: f32 = 114.0; // crossbar length from the side wall

pub const HEAD_R: f32 = 56.0;
pub const BODY_OFF: f32 = 52.0; // body circle centre below head centre
pub const BODY_R: f32 = 32.0;
pub const FEET: f32 = BODY_OFF + BODY_R; // head centre to sole (84.0)
pub const FOOT_R: f32 = 20.0;

pub const BALL_R: f32 = 30.0;

pub const GRAVITY: f32 = 3000.0;
pub const BALL_GRAVITY: f32 = 1480.0;
pub const MOVE_SPEED: f32 = 470.0;
pub const GROUND_ACCEL: f32 = 5800.0;
pub const AIR_ACCEL: f32 = 3400.0;
pub const JUMP_SPEED: f32 = 1245.0;

pub const BALL_AIR_DRAG: f32 = 0.12; // per second
pub const BALL_GROUND_E: f32 = 0.74;
pub const BALL_WALL_E: f32 = 0.80;
pub const BALL_BAR_E: f32 = 0.75;
pub const BALL_ROLL_FRICTION: f32 = 0.9; // per second while rolling
pub const BALL_MAX_SPEED: f32 = 1950.0;
pub const HEAD_E: f32 = 0.62;
pub const HEAD_BOOST: f32 = 110.0;

pub const KICK_TIME: f32 = 0.20;
pub const KICK_COOLDOWN: f32 = 0.33;
pub const KICK_REACH: f32 = 22.0;
pub const LOW_KICK_SPEED: f32 = 1120.0;
pub const LOW_KICK_DIR: (f32, f32) = (0.89, -0.45);
pub const HIGH_KICK_SPEED: f32 = 1220.0;
pub const HIGH_KICK_DIR: (f32, f32) = (0.52, -0.85);
pub const KICK_CARRY: f32 = 0.25; // share of the player's speed added to a kick
pub const KICK_STUN: f32 = 0.9; // kicking the opponent knocks them down
pub const KICK_KNOCK: f32 = 340.0;

pub const POWER_CHARGE_TIME: f32 = 20.0; // seconds to fill the power bar
pub const POWER_PER_TOUCH: f32 = 0.05;
pub const POWER_READY_TIME: f32 = 6.0; // how long the power waits for a kick
pub const FIRE_SPEED: f32 = 1750.0;
pub const FIRE_TIME: f32 = 1.1;
pub const FIRE_STUN: f32 = 1.6;
pub const FIRE_KNOCK: f32 = 620.0;

// the power each player brings (chosen before the match)
pub const PK_FIRE: u8 = 0; // next kick: a fire shot straight at the goal
pub const PK_BIG: u8 = 1; // the ball grows for a while
pub const PK_ICE: u8 = 2; // instant freeze: encases the opponent in ice (even mid-air)
pub const PK_INVIS: u8 = 3; // the ball disappears for the opponent
pub const PK_BOMB: u8 = 4; // next kick: the ball becomes a bomb that blows the opponent away
pub const PK_CLONE: u8 = 5; // next kick: two fake balls fly off beside the real one
pub const POWER_KINDS: u8 = 6;
pub const POWER_NAMES: [&str; 6] = ["Fogo", "Bola gigante", "Gelo", "Invisível", "Bomba", "Clones"];

/// One playable character: its name and how it plays (1.0 = normal). The differences
/// are small on purpose, like the attributes of the original (speed, jump, kick, power).
pub struct CharStats {
    pub name: &'static str,
    pub speed: f32,
    pub jump: f32,
    pub kick: f32,
    pub charge: f32, // how fast the power bar fills
}

/// The characters, in the order of the pictures `p1_head.png` .. `p6_head.png`.
pub const CHARS: [CharStats; 6] = [
    CharStats { name: "Rafa", speed: 1.00, jump: 1.00, kick: 1.00, charge: 1.00 },
    CharStats { name: "Zeca", speed: 1.08, jump: 0.97, kick: 0.97, charge: 1.00 },
    CharStats { name: "Dudu", speed: 0.97, jump: 1.08, kick: 1.00, charge: 0.97 },
    CharStats { name: "Lia", speed: 1.05, jump: 1.04, kick: 0.94, charge: 1.00 },
    CharStats { name: "Ruivo", speed: 0.94, jump: 0.97, kick: 1.10, charge: 1.00 },
    CharStats { name: "Rei", speed: 1.00, jump: 0.98, kick: 0.98, charge: 1.12 },
];

pub fn char_stats(look: u8) -> &'static CharStats {
    &CHARS[(look as usize) % CHARS.len()]
}

pub const BIG_TIME: f32 = 7.0;
pub const BIG_SCALE: f32 = 1.85;
pub const FREEZE_TIME: f32 = 3.2;
pub const FREEZE_MASH_REDUCE: f32 = 0.22;
pub const FREEZE_MASH_CD: f32 = 0.08;
pub const FREEZE_MIN_TIME: f32 = 1.0;
pub const INVIS_TIME: f32 = 4.0;
pub const BOMB_FUSE: f32 = 2.2;
pub const BOMB_RADIUS: f32 = 260.0;
pub const BOMB_STUN: f32 = 1.8;
pub const BOMB_KNOCK: f32 = 760.0;
pub const CLONE_TIME: f32 = 4.5;

pub const COUNTDOWN: f32 = 3.0;
pub const GOAL_PAUSE: f32 = 2.2;
pub const KICKOFF_READY: f32 = 1.0;
pub const BALL_DROP_Y: f32 = 175.0;

pub const MATCH_LENGTHS: [f32; 3] = [60.0, 90.0, 120.0];

// event bits (for sounds and effects)
pub const EV_KICK: u32 = 1;
pub const EV_HEAD: u32 = 2;
pub const EV_BOUNCE: u32 = 4;
pub const EV_POST: u32 = 8;
pub const EV_GOAL: u32 = 16;
pub const EV_WHISTLE: u32 = 32;
pub const EV_POWER: u32 = 64;
pub const EV_STUN: u32 = 128;
pub const EV_BEEP: u32 = 256;
pub const EV_FIRE: u32 = 512;
pub const EV_BOOM: u32 = 1024;
#[allow(dead_code)]
pub const EV_FREEZE: u32 = 2048;
pub const EV_ICE_CAST: u32 = 1 << 12;
pub const EV_ICE_CRACK: u32 = 1 << 13;
pub const EV_THAW: u32 = 1 << 14;
pub const EV_BIG_CAST: u32 = 1 << 15;
pub const EV_INVIS_CAST: u32 = 1 << 16;
pub const EV_CLONE_CAST: u32 = 1 << 17;
pub const EV_NET_HIT: u32 = 1 << 18;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub kick: bool,
    pub lob: bool,
    pub power: bool,
}

impl Input {
    pub fn bits(&self) -> u8 {
        (self.left as u8)
            | (self.right as u8) << 1
            | (self.jump as u8) << 2
            | (self.kick as u8) << 3
            | (self.lob as u8) << 4
            | (self.power as u8) << 5
    }
    pub fn from_bits(b: u8) -> Input {
        Input {
            left: b & 1 != 0,
            right: b & 2 != 0,
            jump: b & 4 != 0,
            kick: b & 8 != 0,
            lob: b & 16 != 0,
            power: b & 32 != 0,
        }
    }
    pub fn or(self, o: Input) -> Input {
        Input {
            left: self.left || o.left,
            right: self.right || o.right,
            jump: self.jump || o.jump,
            kick: self.kick || o.kick,
            lob: self.lob || o.lob,
            power: self.power || o.power,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub side: f32, // +1 plays toward the right goal, -1 toward the left
    pub on_ground: bool,
    pub kick_t: f32, // time since the kick started, <0 when not kicking
    pub kick_kind: u8, // 1 low, 2 high
    pub kick_hit: bool,
    pub kick_cd: f32,
    pub stun: f32,
    pub power: f32, // 0..1
    pub power_ready: f32,
    pub walk: f32, // walk animation phase
    pub pkind: u8, // which power this player brings (PK_*)
    pub frozen: f32, // frozen in ice (separate from stun; mash buttons to break out)
    pub look: u8, // which character (CHARS): its face and its attributes
}

impl Player {
    pub fn new(side: f32) -> Player {
        let mut p = Player { side, kick_t: -1.0, ..Default::default() };
        p.reset();
        p
    }
    pub fn reset(&mut self) {
        self.x = if self.side > 0.0 { 330.0 } else { W - 330.0 };
        self.y = GROUND - FEET;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = true;
        self.kick_t = -1.0;
        self.kick_hit = false;
        self.kick_cd = 0.0;
        self.stun = 0.0;
        self.frozen = 0.0;
        self.power_ready = 0.0;
    }
    pub fn body(&self) -> (f32, f32) {
        (self.x, self.y + BODY_OFF)
    }
    /// Where the kicking foot is now.
    pub fn foot(&self) -> (f32, f32) {
        let rest = (self.x + self.side * 22.0, self.y + FEET - FOOT_R);
        if self.kick_t < 0.0 {
            return rest;
        }
        let ext = if self.kick_kind == 2 {
            (self.x + self.side * 66.0, self.y + 22.0)
        } else {
            (self.x + self.side * 74.0, self.y + 62.0)
        };
        let k = (self.kick_t / KICK_TIME).clamp(0.0, 1.0);
        let f = (k * std::f32::consts::PI).sin();
        (rest.0 + (ext.0 - rest.0) * f, rest.1 + (ext.1 - rest.1) * f)
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub angle: f32,
    pub fire: f32,
    pub fire_owner: u8,
    pub big: f32,
    pub ice: f32,
    pub ice_owner: u8,
    pub invis: f32,
    pub invis_owner: u8,
    pub bomb: f32,
    pub bomb_owner: u8,
    pub clone_t: f32,
    pub clones: [[f32; 4]; 2], // x, y, vx, vy of the fake balls
}

impl Ball {
    pub fn r(&self) -> f32 {
        if self.big > 0.0 {
            BALL_R * BIG_SCALE
        } else {
            BALL_R
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Countdown,
    Play,
    Goal,
    Ready,
    Over,
}

impl Phase {
    fn to_u8(self) -> u8 {
        match self {
            Phase::Countdown => 0,
            Phase::Play => 1,
            Phase::Goal => 2,
            Phase::Ready => 3,
            Phase::Over => 4,
        }
    }
    fn from_u8(v: u8) -> Phase {
        match v {
            0 => Phase::Countdown,
            1 => Phase::Play,
            2 => Phase::Goal,
            3 => Phase::Ready,
            _ => Phase::Over,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sim {
    pub p: [Player; 2],
    pub ball: Ball,
    pub score: [u8; 2],
    pub time_left: f32,
    pub match_len: f32,
    pub phase: Phase,
    pub phase_t: f32,
    pub last_scorer: u8,
    pub events: u32,
    pub touches: [u32; 2],
    pub overtime: bool, // tied at the whistle: the next goal wins (golden goal)
    pub stadium: u8,    // which stadium picture (sent to the other device too)
    prev: [Input; 2],
    rng: u32,
    freeze_mash_cd: [f32; 2],
    freeze_age: [f32; 2],
}

fn len(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

impl Sim {
    pub fn new(match_len: f32) -> Sim {
        let mut s = Sim {
            p: [Player::new(1.0), Player::new(-1.0)],
            ball: Ball::default(),
            score: [0, 0],
            time_left: match_len,
            match_len,
            phase: Phase::Countdown,
            phase_t: COUNTDOWN,
            last_scorer: 0,
            events: 0,
            touches: [0, 0],
            overtime: false,
            stadium: 0,
            prev: [Input::default(); 2],
            rng: 0x9e3779b9,
            freeze_mash_cd: [0.0; 2],
            freeze_age: [0.0; 2],
        };
        s.reset_ball();
        s
    }

    /// The clock ran out (or a goal was scored after it did): a tie goes to golden-goal
    /// overtime, anything else ends the match.
    fn end_of_time(&mut self) {
        if self.score[0] == self.score[1] && !self.overtime {
            self.overtime = true;
            for p in self.p.iter_mut() {
                p.reset();
            }
            self.freeze_mash_cd = [0.0; 2];
            self.freeze_age = [0.0; 2];
            self.reset_ball();
            self.phase = Phase::Ready;
            self.phase_t = KICKOFF_READY * 2.0;
        } else {
            self.phase = Phase::Over;
        }
        self.events |= EV_WHISTLE;
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng % 10000) as f32 / 10000.0
    }

    fn reset_ball(&mut self) {
        let r = self.rand();
        self.ball = Ball { x: W / 2.0, y: BALL_DROP_Y, vx: (r - 0.5) * 60.0, vy: 0.0, ..Default::default() };
    }

    pub fn take_events(&mut self) -> u32 {
        std::mem::take(&mut self.events)
    }

    pub fn winner(&self) -> Option<usize> {
        if self.score[0] > self.score[1] {
            Some(0)
        } else if self.score[1] > self.score[0] {
            Some(1)
        } else {
            None
        }
    }

    pub fn step(&mut self, dt: f32, input: [Input; 2]) {
        let prev = self.prev;
        self.prev = input;
        match self.phase {
            Phase::Countdown | Phase::Ready => {
                let before = self.phase_t.ceil();
                self.phase_t -= dt;
                if self.phase == Phase::Countdown && self.phase_t.ceil() < before && self.phase_t > 0.0 {
                    self.events |= EV_BEEP;
                }
                // players may move while waiting, the ball hangs
                self.step_players(dt, input, prev, false);
                if self.phase_t <= 0.0 {
                    self.phase = Phase::Play;
                    self.events |= EV_WHISTLE;
                }
            }
            Phase::Play => {
                if !self.overtime {
                    self.time_left -= dt;
                }
                self.step_players(dt, input, prev, true);
                self.step_ball(dt);
                self.check_goal();
                if !self.overtime && self.time_left <= 0.0 && self.phase == Phase::Play {
                    self.time_left = 0.0;
                    self.end_of_time();
                }
            }
            Phase::Goal => {
                self.phase_t -= dt;
                self.step_players(dt, [Input::default(); 2], prev, false);
                self.step_ball(dt);
                if self.phase_t <= 0.0 {
                    if self.overtime || self.time_left <= 0.0 {
                        self.end_of_time();
                    } else {
                        for p in self.p.iter_mut() {
                            p.reset();
                        }
                        self.freeze_mash_cd = [0.0; 2];
                        self.freeze_age = [0.0; 2];
                        self.reset_ball();
                        self.phase = Phase::Ready;
                        self.phase_t = KICKOFF_READY;
                    }
                }
            }
            Phase::Over => {
                self.step_players(dt, [Input::default(); 2], prev, false);
                self.step_ball(dt);
            }
        }
    }

    fn step_players(&mut self, dt: f32, input: [Input; 2], prev: [Input; 2], can_kick: bool) {
        let mut freeze_cast = [false; 2];
        for i in 0..2 {
            let inp = input[i];
            let pin = prev[i];
            self.freeze_mash_cd[i] = (self.freeze_mash_cd[i] - dt).max(0.0);

            let p = &mut self.p[i];
            if self.phase == Phase::Play {
                p.power = (p.power + dt / POWER_CHARGE_TIME * char_stats(p.look).charge).min(1.0);
            }
            p.kick_cd = (p.kick_cd - dt).max(0.0);
            if p.power_ready > 0.0 {
                p.power_ready -= dt;
                if p.power_ready <= 0.0 {
                    p.power_ready = 0.0;
                }
            }
            if p.frozen > 0.0 {
                self.freeze_age[i] += dt;
                p.frozen -= dt;
                let mash = (inp.left && !pin.left)
                    || (inp.right && !pin.right)
                    || (inp.jump && !pin.jump)
                    || (inp.kick && !pin.kick)
                    || (inp.lob && !pin.lob);
                if mash && self.freeze_mash_cd[i] <= 0.0 && p.frozen > 0.0 {
                    let floor = (FREEZE_MIN_TIME - self.freeze_age[i]).max(0.0);
                    let next = (p.frozen - FREEZE_MASH_REDUCE).max(floor);
                    if next < p.frozen {
                        p.frozen = next;
                        self.freeze_mash_cd[i] = FREEZE_MASH_CD;
                        self.events |= EV_ICE_CRACK;
                    }
                }
                if p.frozen <= 0.0 {
                    p.frozen = 0.0;
                    self.freeze_age[i] = 0.0;
                    self.freeze_mash_cd[i] = 0.0;
                    self.events |= EV_THAW;
                }
            } else {
                self.freeze_age[i] = 0.0;
            }
            let frozen = p.frozen > 0.0;
            let stunned = p.stun > 0.0;
            if stunned {
                p.stun = (p.stun - dt).max(0.0);
            }
            if frozen {
                // Suspended solidly in ice (even in mid-air): no gravity, movement, jump, kick or power.
                p.vx = 0.0;
                p.vy = 0.0;
                p.kick_t = -1.0;
                continue;
            }
            let mut want = 0.0;
            if !stunned {
                if inp.left {
                    want -= 1.0;
                }
                if inp.right {
                    want += 1.0;
                }
            }
            let target = want * MOVE_SPEED * char_stats(p.look).speed;
            let acc = if p.on_ground { GROUND_ACCEL } else { AIR_ACCEL } * dt;
            if stunned && p.on_ground {
                p.vx *= 1.0 - (6.0 * dt).min(1.0);
            } else if !stunned {
                if (target - p.vx).abs() <= acc {
                    p.vx = target;
                } else {
                    p.vx += acc * (target - p.vx).signum();
                }
            }
            if !stunned && inp.jump && p.on_ground {
                p.vy = -JUMP_SPEED * char_stats(p.look).jump;
                p.on_ground = false;
            }
            if !stunned && can_kick && p.kick_cd <= 0.0 && p.kick_t < 0.0 {
                let kick = inp.kick && !pin.kick;
                let lob = inp.lob && !pin.lob;
                if kick || lob {
                    p.kick_t = 0.0;
                    p.kick_kind = if lob { 2 } else { 1 };
                    p.kick_hit = false;
                    p.kick_cd = KICK_COOLDOWN;
                }
            }
            if !stunned && inp.power && !pin.power && p.power >= 1.0 && self.phase == Phase::Play {
                p.power = 0.0;
                self.events |= EV_POWER;
                match p.pkind {
                    PK_BIG => {
                        self.ball.big = BIG_TIME;
                        self.events |= EV_BIG_CAST;
                    }
                    PK_INVIS => {
                        self.ball.invis = INVIS_TIME;
                        self.ball.invis_owner = i as u8;
                        self.events |= EV_INVIS_CAST;
                    }
                    PK_ICE => {
                        freeze_cast[1 - i] = true;
                        self.events |= EV_ICE_CAST;
                    }
                    _ => p.power_ready = POWER_READY_TIME, // the next kick carries it
                }
            }
            if p.kick_t >= 0.0 {
                p.kick_t += dt;
                if p.kick_t > KICK_TIME {
                    p.kick_t = -1.0;
                }
            }
            p.vy += GRAVITY * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.on_ground = false;
            if p.y + FEET >= GROUND {
                p.y = GROUND - FEET;
                p.vy = 0.0;
                p.on_ground = true;
            }
            if p.on_ground && p.vx.abs() > 10.0 {
                p.walk += p.vx.abs() * dt * 0.05;
            }
            let minx = HEAD_R + 4.0;
            if p.x < minx {
                p.x = minx;
                p.vx = p.vx.max(0.0);
            }
            if p.x > W - minx {
                p.x = W - minx;
                p.vx = p.vx.min(0.0);
            }
            // crossbars
            for bx in [0.0, W - BAR_LEN] {
                for (cx, cy, r) in [(p.x, p.y, HEAD_R), (p.x, p.y + BODY_OFF, BODY_R)] {
                    if let Some((nx, ny, d)) = circle_rect(cx, cy, r, bx, BAR_Y, BAR_LEN, BAR_H) {
                        p.x += nx * d;
                        p.y += ny * d;
                        let vn = p.vx * nx + p.vy * ny;
                        if vn < 0.0 {
                            p.vx -= vn * nx;
                            p.vy -= vn * ny;
                        }
                        if ny < -0.6 {
                            p.on_ground = true;
                        }
                    }
                }
            }
        }
        for i in 0..2 {
            if freeze_cast[i] {
                self.p[i].frozen = FREEZE_TIME;
                self.p[i].vx = 0.0;
                self.p[i].vy = 0.0;
                self.p[i].kick_t = -1.0;
                self.freeze_age[i] = 0.0;
                self.freeze_mash_cd[i] = 0.0;
            }
        }
        // players against each other
        self.collide_players();
        // kicks
        if can_kick {
            for i in 0..2 {
                self.resolve_kick(i);
            }
        }
    }

    fn collide_players(&mut self) {
        let pairs = [(0.0, 0.0, HEAD_R), (BODY_OFF, BODY_OFF, BODY_R)];
        let (a, b) = self.p.split_at_mut(1);
        let a = &mut a[0];
        let b = &mut b[0];
        let af = a.frozen > 0.0;
        let bf = b.frozen > 0.0;
        if af && bf {
            return;
        }
        for (oa, _ob, ra) in pairs {
            for (ob2, _x, rb) in pairs {
                let dx = b.x - a.x;
                let dy = (b.y + ob2) - (a.y + oa);
                let d = len(dx, dy);
                let min = ra + rb;
                if d < min && d > 0.001 {
                    let nx = dx / d;
                    let ny = dy / d;
                    let pen = min - d;
                    let (wa, wb) = match (af, bf) {
                        (true, false) => (0.0, 1.0),
                        (false, true) => (1.0, 0.0),
                        _ => (0.5, 0.5),
                    };
                    a.x -= nx * pen * wa;
                    a.y -= ny * pen * wa;
                    b.x += nx * pen * wb;
                    b.y += ny * pen * wb;
                    let rv = (b.vx - a.vx) * nx + (b.vy - a.vy) * ny;
                    if rv < 0.0 {
                        a.vx += rv * nx * wa;
                        a.vy += rv * ny * wa;
                        b.vx -= rv * nx * wb;
                        b.vy -= rv * ny * wb;
                    }
                    if ny > 0.6 && !af {
                        // a is on top of b
                        a.on_ground = true;
                        a.vy = a.vy.min(0.0);
                    } else if ny < -0.6 && !bf {
                        b.on_ground = true;
                        b.vy = b.vy.min(0.0);
                    }
                }
            }
        }
    }

    fn resolve_kick(&mut self, i: usize) {
        let p = self.p[i];
        if p.kick_t < 0.0 || p.kick_hit {
            return;
        }
        let (fx, fy) = p.foot();
        // the ball
        let d = len(self.ball.x - fx, self.ball.y - fy);
        if d < self.ball.r() + FOOT_R + KICK_REACH {
            let s = p.side;
            let ks = char_stats(p.look).kick;
            if p.power_ready > 0.0 && p.pkind != PK_FIRE {
                let (dx, dy, sp) = if p.kick_kind == 2 {
                    (HIGH_KICK_DIR.0, HIGH_KICK_DIR.1, HIGH_KICK_SPEED * ks)
                } else {
                    (LOW_KICK_DIR.0, LOW_KICK_DIR.1, LOW_KICK_SPEED * ks)
                };
                let b = &mut self.ball;
                b.vx = s * dx * sp + p.vx * KICK_CARRY;
                b.vy = dy * sp + p.vy.min(0.0) * KICK_CARRY;
                b.fire = 0.0;
                match p.pkind {
                    PK_BOMB => {
                        b.bomb = BOMB_FUSE;
                        b.bomb_owner = i as u8;
                        self.events |= EV_FIRE;
                    }
                    PK_CLONE | _ => {
                        // clones: two fake balls split above and below the real one
                        b.clone_t = CLONE_TIME;
                        let (vx, vy) = (b.vx, b.vy);
                        b.clones = [[b.x, b.y, vx * 0.85, vy - 420.0], [b.x, b.y, vx * 1.1, vy + 320.0]];
                        self.events |= EV_CLONE_CAST;
                    }
                }
                self.p[i].power_ready = 0.0;
            } else if p.power_ready > 0.0 {
                // power shot: straight at the far goal mouth
                let gx = if s > 0.0 { W - GOAL_DEPTH * 0.5 } else { GOAL_DEPTH * 0.5 };
                let gy = (BAR_Y + BAR_H + GROUND) * 0.5;
                let dx = gx - self.ball.x;
                let dy = gy - self.ball.y;
                let l = len(dx, dy).max(1.0);
                self.ball.vx = dx / l * FIRE_SPEED;
                self.ball.vy = dy / l * FIRE_SPEED;
                self.ball.fire = FIRE_TIME;
                self.ball.fire_owner = i as u8;
                self.p[i].power_ready = 0.0;
                self.events |= EV_FIRE;
            } else {
                let (dx, dy, sp) = if p.kick_kind == 2 {
                    (HIGH_KICK_DIR.0, HIGH_KICK_DIR.1, HIGH_KICK_SPEED * ks)
                } else {
                    (LOW_KICK_DIR.0, LOW_KICK_DIR.1, LOW_KICK_SPEED * ks)
                };
                self.ball.vx = s * dx * sp + p.vx * KICK_CARRY;
                self.ball.vy = dy * sp + p.vy.min(0.0) * KICK_CARRY;
                self.ball.fire = 0.0;
            }
            self.p[i].kick_hit = true;
            self.touch(i);
            self.events |= EV_KICK;
        }
        // the opponent's body
        let o = 1 - i;
        let ob = self.p[o].body();
        let oh = (self.p[o].x, self.p[o].y);
        let hit_body = len(ob.0 - fx, ob.1 - fy) < BODY_R + FOOT_R + 4.0;
        let hit_head = len(oh.0 - fx, oh.1 - fy) < HEAD_R + FOOT_R;
        if (hit_body || hit_head) && self.p[o].stun <= 0.0 && self.p[o].frozen <= 0.0 {
            self.p[o].stun = KICK_STUN;
            self.p[o].vx = p.side * KICK_KNOCK;
            self.p[o].vy = -300.0;
            self.p[i].kick_hit = true;
            self.events |= EV_STUN;
        }
    }

    fn touch(&mut self, i: usize) {
        self.touches[i] += 1;
        let p = &mut self.p[i];
        if p.power_ready <= 0.0 {
            p.power = (p.power + POWER_PER_TOUCH).min(1.0);
        }
    }

    fn step_powers(&mut self, dt: f32) {
        let cr = self.ball.r();
        let b = &mut self.ball;
        b.big = (b.big - dt).max(0.0);
        b.invis = (b.invis - dt).max(0.0);
        b.ice = (b.ice - dt).max(0.0);
        if b.clone_t > 0.0 {
            b.clone_t -= dt;
            for c in b.clones.iter_mut() {
                c[3] += BALL_GRAVITY * dt;
                c[0] += c[2] * dt;
                c[1] += c[3] * dt;
                if c[1] + cr > GROUND {
                    c[1] = GROUND - cr;
                    c[3] = -c[3].abs() * BALL_GROUND_E;
                }
                if c[1] - cr < 0.0 {
                    c[1] = cr;
                    c[3] = c[3].abs();
                }
                if c[0] - cr < 0.0 || c[0] + cr > W {
                    c[0] = c[0].clamp(cr, W - cr);
                    c[2] = -c[2] * BALL_WALL_E;
                }
            }
        }
        if b.bomb > 0.0 {
            b.bomb -= dt;
            if b.bomb <= 0.0 {
                b.bomb = 0.0;
                b.vy -= 560.0;
                let (bx, by) = (b.x, b.y);
                let o = 1 - b.bomb_owner as usize;
                let q = &mut self.p[o];
                let dx = q.x - bx;
                let dy = q.y - by;
                if len(dx, dy) < BOMB_RADIUS {
                    if q.frozen > 0.0 {
                        q.frozen = 0.0;
                        self.freeze_age[o] = 0.0;
                        self.freeze_mash_cd[o] = 0.0;
                        self.events |= EV_THAW;
                    }
                    q.stun = q.stun.max(BOMB_STUN);
                    q.vx = if dx >= 0.0 { BOMB_KNOCK } else { -BOMB_KNOCK };
                    q.vy = -660.0;
                    self.events |= EV_STUN;
                }
                self.events |= EV_BOOM;
            }
        }
    }

    fn step_ball(&mut self, dt: f32) {
        self.step_powers(dt);
        let br = self.ball.r();
        let b = &mut self.ball;
        if b.fire > 0.0 {
            b.fire -= dt;
        } else {
            b.vy += BALL_GRAVITY * dt;
            let drag = 1.0 - BALL_AIR_DRAG * dt;
            b.vx *= drag;
            b.vy *= drag;
        }
        let sp = len(b.vx, b.vy);
        if sp > BALL_MAX_SPEED {
            b.vx *= BALL_MAX_SPEED / sp;
            b.vy *= BALL_MAX_SPEED / sp;
        }
        b.x += b.vx * dt;
        b.y += b.vy * dt;
        b.angle += b.vx * dt / br;
        let mut ev = 0;
        if b.y + br > GROUND {
            b.y = GROUND - br;
            if b.vy > 150.0 {
                ev |= EV_BOUNCE;
            }
            b.vy = -b.vy * BALL_GROUND_E;
            if b.vy.abs() < 70.0 {
                b.vy = 0.0;
            }
            b.vx *= 1.0 - BALL_ROLL_FRICTION * dt;
            b.fire = 0.0;
        }
        if b.y - br < 0.0 {
            b.y = br;
            b.vy = b.vy.abs() * BALL_WALL_E;
            b.fire = 0.0;
        }
        if b.x - br < 0.0 {
            b.x = br;
            let in_net = b.y > BAR_Y + BAR_H;
            if in_net && b.vx < -80.0 {
                ev |= EV_NET_HIT;
            }
            b.vx = b.vx.abs() * if in_net { 0.48 } else { BALL_WALL_E };
        }
        if b.x + br > W {
            b.x = W - br;
            let in_net = b.y > BAR_Y + BAR_H;
            if in_net && b.vx > 80.0 {
                ev |= EV_NET_HIT;
            }
            b.vx = -b.vx.abs() * if in_net { 0.48 } else { BALL_WALL_E };
        }
        for bx in [0.0, W - BAR_LEN] {
            if let Some((nx, ny, d)) = circle_rect(b.x, b.y, br, bx, BAR_Y, BAR_LEN, BAR_H) {
                b.x += nx * d;
                b.y += ny * d;
                let vn = b.vx * nx + b.vy * ny;
                if vn < 0.0 {
                    b.vx -= (1.0 + BALL_BAR_E) * vn * nx;
                    b.vy -= (1.0 + BALL_BAR_E) * vn * ny;
                    if vn < -120.0 {
                        ev |= EV_POST;
                    }
                }
                b.fire = 0.0;
            }
        }
        self.events |= ev;
        // ball against players
        for i in 0..2 {
            let fire_through = self.ball.fire > 0.0 && self.ball.fire_owner as usize != i;
            let p = self.p[i];
            let mut touched = false;
            for (cx, cy, r) in [(p.x, p.y, HEAD_R), (p.x, p.y + BODY_OFF, BODY_R)] {
                let mut bb = self.ball;
                let b = &mut bb;
                let dx = b.x - cx;
                let dy = b.y - cy;
                let d = len(dx, dy);
                let min = r + br;
                if d < min && d > 0.001 {
                    if fire_through {
                        // a power shot knocks the player away and keeps going
                        if self.p[i].stun <= 0.0 && self.p[i].frozen <= 0.0 {
                            self.p[i].stun = FIRE_STUN;
                            self.p[i].vx = b.vx.signum() * FIRE_KNOCK;
                            self.p[i].vy = -480.0;
                            self.events |= EV_STUN;
                        }
                        continue;
                    }
                    let nx = dx / d;
                    let ny = dy / d;
                    b.x = cx + nx * min;
                    b.y = cy + ny * min;
                    let rvx = b.vx - p.vx;
                    let rvy = b.vy - p.vy;
                    let vn = rvx * nx + rvy * ny;
                    if vn < 0.0 {
                        let nvx = rvx - (1.0 + HEAD_E) * vn * nx + nx * HEAD_BOOST;
                        let nvy = rvy - (1.0 + HEAD_E) * vn * ny + ny * HEAD_BOOST;
                        b.vx = nvx + p.vx;
                        b.vy = nvy + p.vy.min(0.0);
                        if vn < -200.0 {
                            self.events |= EV_HEAD;
                        }
                        touched = true;
                    }
                    if !(b.fire > 0.0 && b.fire_owner as usize == i) {
                        b.fire = 0.0;
                    }
                }
                self.ball = bb;
            }
            if touched {
                self.touch(i);
            }
        }
    }

    fn check_goal(&mut self) {
        let b = self.ball;
        let br = b.r();
        // Normal ball must fully cross GOAL_DEPTH; giant ball (2*r > GOAL_DEPTH) scores
        // once its centre is deep inside the goal with room before the back wall.
        let line = (GOAL_DEPTH - br).max(br + 12.0);
        let scorer = if b.x < line && b.y > BAR_Y + BAR_H {
            Some(1u8)
        } else if b.x > W - line && b.y > BAR_Y + BAR_H {
            Some(0u8)
        } else {
            None
        };
        if let Some(s) = scorer {
            self.score[s as usize] += 1;
            self.last_scorer = s;
            self.phase = Phase::Goal;
            self.phase_t = GOAL_PAUSE;
            self.ball.fire = 0.0;
            self.ball.bomb = 0.0;
            self.ball.ice = 0.0;
            self.ball.clone_t = 0.0;
            for p in self.p.iter_mut() {
                p.power_ready = 0.0;
                p.kick_t = -1.0;
            }
            self.events |= EV_GOAL | EV_NET_HIT;
        }
    }

    // ---------- network snapshot ----------

    pub fn to_bytes(&self, out: &mut Vec<u8>) {
        let f = |out: &mut Vec<u8>, v: f32| out.extend_from_slice(&v.to_le_bytes());
        for p in &self.p {
            f(out, p.x);
            f(out, p.y);
            f(out, p.vx);
            f(out, p.vy);
            f(out, p.kick_t);
            f(out, p.stun);
            f(out, p.power);
            f(out, p.power_ready);
            f(out, p.walk);
            out.push(p.kick_kind);
            out.push(p.on_ground as u8);
            out.push(p.pkind);
            out.push(p.look);
            f(out, p.frozen);
        }
        let b = &self.ball;
        for v in [b.x, b.y, b.vx, b.vy, b.angle, b.fire] {
            f(out, v);
        }
        out.push(b.fire_owner);
        out.push(self.score[0]);
        out.push(self.score[1]);
        f(out, self.time_left);
        f(out, self.match_len);
        out.push(self.phase.to_u8());
        f(out, self.phase_t);
        out.push(self.last_scorer);
        for v in [b.big, b.ice, b.invis, b.bomb, b.clone_t] {
            f(out, v);
        }
        out.push(b.ice_owner);
        out.push(b.invis_owner);
        out.push(b.bomb_owner);
        for c in b.clones {
            for v in c {
                f(out, v);
            }
        }
        out.push(self.overtime as u8);
        out.extend_from_slice(&self.events.to_le_bytes());
        out.push(self.stadium);
    }

    pub fn from_bytes(data: &[u8]) -> Option<Sim> {
        let mut r = Reader { d: data, i: 0 };
        let mut s = Sim::new(90.0);
        for p in s.p.iter_mut() {
            p.x = r.f()?;
            p.y = r.f()?;
            p.vx = r.f()?;
            p.vy = r.f()?;
            p.kick_t = r.f()?;
            p.stun = r.f()?;
            p.power = r.f()?;
            p.power_ready = r.f()?;
            p.walk = r.f()?;
            p.kick_kind = r.u()?;
            p.on_ground = r.u()? != 0;
            p.pkind = r.u()?;
            p.look = r.u()?;
            p.frozen = r.f()?;
        }
        s.ball.x = r.f()?;
        s.ball.y = r.f()?;
        s.ball.vx = r.f()?;
        s.ball.vy = r.f()?;
        s.ball.angle = r.f()?;
        s.ball.fire = r.f()?;
        s.ball.fire_owner = r.u()?;
        s.score[0] = r.u()?;
        s.score[1] = r.u()?;
        s.time_left = r.f()?;
        s.match_len = r.f()?;
        s.phase = Phase::from_u8(r.u()?);
        s.phase_t = r.f()?;
        s.last_scorer = r.u()?;
        s.ball.big = r.f()?;
        s.ball.ice = r.f()?;
        s.ball.invis = r.f()?;
        s.ball.bomb = r.f()?;
        s.ball.clone_t = r.f()?;
        s.ball.ice_owner = r.u()?;
        s.ball.invis_owner = r.u()?;
        s.ball.bomb_owner = r.u()?;
        for c in s.ball.clones.iter_mut() {
            for v in c.iter_mut() {
                *v = r.f()?;
            }
        }
        s.overtime = r.u()? != 0;
        let e = r.take(4)?;
        s.events = u32::from_le_bytes([e[0], e[1], e[2], e[3]]);
        s.stadium = r.u()? % 3;
        Some(s)
    }
}

struct Reader<'a> {
    d: &'a [u8],
    i: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.i + n > self.d.len() {
            return None;
        }
        let s = &self.d[self.i..self.i + n];
        self.i += n;
        Some(s)
    }
    fn f(&mut self) -> Option<f32> {
        let b = self.take(4)?;
        Some(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
}

/// Circle against an axis-aligned box: the push-out normal and depth.
pub fn circle_rect(cx: f32, cy: f32, r: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> Option<(f32, f32, f32)> {
    let qx = cx.clamp(rx, rx + rw);
    let qy = cy.clamp(ry, ry + rh);
    let dx = cx - qx;
    let dy = cy - qy;
    let d = len(dx, dy);
    if d >= r {
        return None;
    }
    if d > 0.0001 {
        return Some((dx / d, dy / d, r - d));
    }
    // centre inside the box: push out the shortest way
    let left = cx - rx;
    let right = rx + rw - cx;
    let top = cy - ry;
    let bottom = ry + rh - cy;
    let m = left.min(right).min(top).min(bottom);
    if m == top {
        Some((0.0, -1.0, top + r))
    } else if m == bottom {
        Some((0.0, 1.0, bottom + r))
    } else if m == left {
        Some((-1.0, 0.0, left + r))
    } else {
        Some((1.0, 0.0, right + r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    #[test]
    fn wire_roundtrip_preserves_190_bytes_and_events() {
        let mut s = Sim::new(90.0);
        s.score = [3, 2];
        s.p[0].x = 512.5;
        s.p[1].frozen = 2.45;
        s.events = EV_ICE_CAST | EV_ICE_CRACK | EV_THAW | EV_BIG_CAST | EV_INVIS_CAST | EV_CLONE_CAST | EV_NET_HIT;
        let mut buf = Vec::new();
        s.to_bytes(&mut buf);
        assert_eq!(buf.len(), 190, "wire snapshot layout must stay 190 bytes");
        let got = Sim::from_bytes(&buf).expect("from_bytes failed");
        assert_eq!(got.score, [3, 2]);
        assert!((got.p[0].x - 512.5).abs() < 1e-4);
        assert!((got.p[1].frozen - 2.45).abs() < 1e-4);
        assert_eq!(got.events, s.events);
    }

    #[test]
    fn jump_clears_crossbar_and_lands_quickly() {
        let mut s = Sim::new(90.0);
        s.phase = Phase::Play;
        let y0 = s.p[0].y;
        let mut min_y = y0;
        let mut landed_at = None;
        for step in 0..140 {
            let mut inp = [Input::default(); 2];
            inp[0].jump = step < 3;
            s.step(DT, inp);
            min_y = min_y.min(s.p[0].y);
            if step > 10 && s.p[0].on_ground && landed_at.is_none() {
                landed_at = Some(step as f32 * DT);
            }
        }
        let rise = y0 - min_y;
        let sole_apex = min_y + FEET;
        assert!(rise > 250.0, "jump rise {} too low", rise);
        assert!(sole_apex < BAR_Y, "sole apex {} did not clear BAR_Y {}", sole_apex, BAR_Y);
        assert!(landed_at.unwrap_or(9.9) < 0.95, "jump airtime too long: {:?}", landed_at);
    }

    #[test]
    fn instant_freeze_midair_and_mash_breakout() {
        let mut s = Sim::new(90.0);
        s.phase = Phase::Play;
        s.p[0].pkind = PK_ICE;
        s.p[0].power = 1.0;
        // Put opponent in mid-air before freezing
        s.p[1].y = GROUND - FEET - 120.0;
        s.p[1].vy = -200.0;
        s.p[1].on_ground = false;

        let mut inp = [Input::default(); 2];
        inp[0].power = true;
        s.step(DT, inp);
        let ev = s.take_events();
        assert!(ev & EV_ICE_CAST != 0, "expected EV_ICE_CAST");
        assert!((s.p[1].frozen - FREEZE_TIME).abs() < 1e-3);
        assert_eq!(s.p[1].stun, 0.0, "freeze must stay separate from stun");
        assert_eq!(s.p[1].vx, 0.0);
        assert_eq!(s.p[1].vy, 0.0);
        let frozen_y = s.p[1].y;

        // Hold still for 0.3s: must remain suspended at the same Y
        for _ in 0..36 {
            s.step(DT, [Input::default(); 2]);
        }
        assert!((s.p[1].y - frozen_y).abs() < 1e-4, "mid-air frozen player fell!");
        assert!(s.p[1].frozen > 2.5);

        // Rapid button mashing breaks out early, emitting EV_ICE_CRACK and EV_THAW
        let mut cracks = 0;
        let mut thawed = false;
        let mut elapsed_steps = 36;
        for step in 0..240 {
            let mut m = [Input::default(); 2];
            // Press kick every 12 steps (0.10 s > 0.08 s cooldown)
            m[1].kick = step % 12 == 0;
            s.step(DT, m);
            elapsed_steps += 1;
            let e = s.take_events();
            if e & EV_ICE_CRACK != 0 {
                cracks += 1;
            }
            if e & EV_THAW != 0 {
                thawed = true;
                break;
            }
        }
        let total_secs = elapsed_steps as f32 * DT;
        assert!(cracks >= 5, "expected multiple crack events, got {}", cracks);
        assert!(thawed, "expected EV_THAW");
        assert!(total_secs >= FREEZE_MIN_TIME - 0.02, "broke out before min floor: {} s", total_secs);
        assert!(total_secs < 2.2, "button mashing did not shorten freeze: {} s", total_secs);

        // After thaw, gravity resumes and player falls toward the ground
        for _ in 0..10 {
            s.step(DT, [Input::default(); 2]);
        }
        assert!(s.p[1].y > frozen_y + 1.0, "gravity did not resume after thaw");
    }

    #[test]
    fn giant_ball_scores_and_power_events_fire() {
        let mut s = Sim::new(90.0);
        s.phase = Phase::Play;
        s.p[0].pkind = PK_BIG;
        s.p[0].power = 1.0;
        let mut inp = [Input::default(); 2];
        inp[0].power = true;
        s.step(DT, inp);
        assert!(s.take_events() & EV_BIG_CAST != 0);
        assert!((s.ball.r() - BALL_R * BIG_SCALE).abs() < 1e-3);

        // Kick giant ball into right goal
        s.ball.x = W - GOAL_DEPTH - 20.0;
        s.ball.y = GROUND - s.ball.r();
        s.ball.vx = 900.0;
        s.ball.vy = 0.0;
        let mut scored_ev = 0;
        for _ in 0..60 {
            s.step(DT, [Input::default(); 2]);
            scored_ev |= s.take_events();
            if s.score[0] > 0 {
                break;
            }
        }
        assert_eq!(s.score[0], 1, "giant ball must be able to score");
        assert!(scored_ev & EV_GOAL != 0);
        assert!(scored_ev & EV_NET_HIT != 0);

        // Invisible + Clone events
        let mut s2 = Sim::new(90.0);
        s2.phase = Phase::Play;
        s2.p[0].pkind = PK_INVIS;
        s2.p[0].power = 1.0;
        s2.step(DT, inp);
        assert!(s2.take_events() & EV_INVIS_CAST != 0);

        let mut s3 = Sim::new(90.0);
        s3.phase = Phase::Play;
        s3.p[0].pkind = PK_CLONE;
        s3.p[0].power = 1.0;
        s3.p[0].x = 400.0;
        s3.ball = Ball { x: 400.0 + HEAD_R + BALL_R + 4.0, y: GROUND - BALL_R, ..Default::default() };
        s3.step(DT, inp);
        s3.take_events();
        let mut kick = [Input::default(); 2];
        kick[0].kick = true;
        let mut clone_ev = 0;
        for _ in 0..6 {
            s3.step(DT, kick);
            clone_ev |= s3.take_events();
            kick[0].kick = false;
        }
        assert!(clone_ev & EV_CLONE_CAST != 0, "expected EV_CLONE_CAST on clone kick");
        assert!(s3.ball.clone_t > 0.0);
    }
}
