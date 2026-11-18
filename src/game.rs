//! Match simulation: two big-head players, one ball, two goals.
//! All numbers live here and are listed in MECHANICS.md / mechanics.json.
//! Units are virtual pixels on a 1280x720 field and seconds.

pub const W: f32 = 1280.0;
pub const H: f32 = 720.0;
pub const GROUND: f32 = 620.0;

pub const GOAL_DEPTH: f32 = 86.0; // goal line distance from the side wall
pub const BAR_Y: f32 = 388.0; // top of the crossbar
pub const BAR_H: f32 = 12.0; // crossbar thickness
pub const BAR_LEN: f32 = 100.0; // crossbar length from the side wall

pub const HEAD_R: f32 = 44.0;
pub const BODY_OFF: f32 = 38.0; // body circle centre below head centre
pub const BODY_R: f32 = 24.0;
pub const FEET: f32 = BODY_OFF + BODY_R; // head centre to sole
pub const FOOT_R: f32 = 15.0;

pub const BALL_R: f32 = 21.0;

pub const GRAVITY: f32 = 1900.0;
pub const BALL_GRAVITY: f32 = 1250.0;
pub const MOVE_SPEED: f32 = 340.0;
pub const GROUND_ACCEL: f32 = 4200.0;
pub const AIR_ACCEL: f32 = 2600.0;
pub const JUMP_SPEED: f32 = 830.0;

pub const BALL_AIR_DRAG: f32 = 0.12; // per second
pub const BALL_GROUND_E: f32 = 0.70;
pub const BALL_WALL_E: f32 = 0.80;
pub const BALL_BAR_E: f32 = 0.75;
pub const BALL_ROLL_FRICTION: f32 = 0.9; // per second while rolling
pub const BALL_MAX_SPEED: f32 = 1750.0;
pub const HEAD_E: f32 = 0.62;
pub const HEAD_BOOST: f32 = 90.0;

pub const KICK_TIME: f32 = 0.20;
pub const KICK_COOLDOWN: f32 = 0.33;
pub const KICK_REACH: f32 = 16.0;
pub const LOW_KICK_SPEED: f32 = 980.0;
pub const LOW_KICK_DIR: (f32, f32) = (0.85, -0.53);
pub const HIGH_KICK_SPEED: f32 = 1080.0;
pub const HIGH_KICK_DIR: (f32, f32) = (0.52, -0.85);
pub const KICK_CARRY: f32 = 0.25; // share of the player's speed added to a kick
pub const KICK_STUN: f32 = 0.9; // kicking the opponent knocks them down
pub const KICK_KNOCK: f32 = 260.0;

pub const POWER_CHARGE_TIME: f32 = 20.0; // seconds to fill the power bar
pub const POWER_PER_TOUCH: f32 = 0.05;
pub const POWER_READY_TIME: f32 = 6.0; // how long the power waits for a kick
pub const FIRE_SPEED: f32 = 1550.0;
pub const FIRE_TIME: f32 = 1.1;
pub const FIRE_STUN: f32 = 1.6;
pub const FIRE_KNOCK: f32 = 520.0;

pub const COUNTDOWN: f32 = 3.0;
pub const GOAL_PAUSE: f32 = 2.2;
pub const KICKOFF_READY: f32 = 1.0;
pub const BALL_DROP_Y: f32 = 170.0;

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
        self.power_ready = 0.0;
    }
    pub fn body(&self) -> (f32, f32) {
        (self.x, self.y + BODY_OFF)
    }
    /// Where the kicking foot is now.
    pub fn foot(&self) -> (f32, f32) {
        let rest = (self.x + self.side * 16.0, self.y + FEET - FOOT_R);
        if self.kick_t < 0.0 {
            return rest;
        }
        let ext = if self.kick_kind == 2 {
            (self.x + self.side * 46.0, self.y + 16.0)
        } else {
            (self.x + self.side * 54.0, self.y + 46.0)
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
    prev: [Input; 2],
    rng: u32,
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
            prev: [Input::default(); 2],
            rng: 0x9e3779b9,
        };
        s.reset_ball();
        s
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
                self.time_left -= dt;
                self.step_players(dt, input, prev, true);
                self.step_ball(dt);
                self.check_goal();
                if self.time_left <= 0.0 && self.phase == Phase::Play {
                    self.time_left = 0.0;
                    self.phase = Phase::Over;
                    self.events |= EV_WHISTLE;
                }
            }
            Phase::Goal => {
                self.phase_t -= dt;
                self.step_players(dt, [Input::default(); 2], prev, false);
                self.step_ball(dt);
                if self.phase_t <= 0.0 {
                    if self.time_left <= 0.0 {
                        self.phase = Phase::Over;
                        self.events |= EV_WHISTLE;
                    } else {
                        for p in self.p.iter_mut() {
                            p.reset();
                        }
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
        for i in 0..2 {
            let inp = input[i];
            let pin = prev[i];
            let p = &mut self.p[i];
            if self.phase == Phase::Play {
                p.power = (p.power + dt / POWER_CHARGE_TIME).min(1.0);
            }
            p.kick_cd = (p.kick_cd - dt).max(0.0);
            if p.power_ready > 0.0 {
                p.power_ready -= dt;
                if p.power_ready <= 0.0 {
                    p.power_ready = 0.0;
                }
            }
            let stunned = p.stun > 0.0;
            if stunned {
                p.stun -= dt;
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
            let target = want * MOVE_SPEED;
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
                p.vy = -JUMP_SPEED;
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
                p.power_ready = POWER_READY_TIME;
                self.events |= EV_POWER;
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
        for (oa, _ob, ra) in pairs {
            for (ob2, _x, rb) in pairs {
                let dx = b.x - a.x;
                let dy = (b.y + ob2) - (a.y + oa);
                let d = len(dx, dy);
                let min = ra + rb;
                if d < min && d > 0.001 {
                    let nx = dx / d;
                    let ny = dy / d;
                    let push = (min - d) * 0.5;
                    a.x -= nx * push;
                    a.y -= ny * push;
                    b.x += nx * push;
                    b.y += ny * push;
                    let rv = (b.vx - a.vx) * nx + (b.vy - a.vy) * ny;
                    if rv < 0.0 {
                        a.vx += rv * nx * 0.5;
                        a.vy += rv * ny * 0.5;
                        b.vx -= rv * nx * 0.5;
                        b.vy -= rv * ny * 0.5;
                    }
                    if ny > 0.6 {
                        // a is on top of b
                        a.on_ground = true;
                        a.vy = a.vy.min(0.0);
                    } else if ny < -0.6 {
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
        if d < BALL_R + FOOT_R + KICK_REACH {
            let s = p.side;
            if p.power_ready > 0.0 {
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
                    (HIGH_KICK_DIR.0, HIGH_KICK_DIR.1, HIGH_KICK_SPEED)
                } else {
                    (LOW_KICK_DIR.0, LOW_KICK_DIR.1, LOW_KICK_SPEED)
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
        if (hit_body || hit_head) && self.p[o].stun <= 0.0 {
            self.p[o].stun = KICK_STUN;
            self.p[o].vx = p.side * KICK_KNOCK;
            self.p[o].vy = -200.0;
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

    fn step_ball(&mut self, dt: f32) {
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
        b.angle += b.vx * dt / BALL_R;
        let mut ev = 0;
        if b.y + BALL_R > GROUND {
            b.y = GROUND - BALL_R;
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
        if b.y - BALL_R < 0.0 {
            b.y = BALL_R;
            b.vy = b.vy.abs() * BALL_WALL_E;
            b.fire = 0.0;
        }
        if b.x - BALL_R < 0.0 {
            b.x = BALL_R;
            b.vx = b.vx.abs() * BALL_WALL_E;
        }
        if b.x + BALL_R > W {
            b.x = W - BALL_R;
            b.vx = -b.vx.abs() * BALL_WALL_E;
        }
        for bx in [0.0, W - BAR_LEN] {
            if let Some((nx, ny, d)) = circle_rect(b.x, b.y, BALL_R, bx, BAR_Y, BAR_LEN, BAR_H) {
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
                let min = r + BALL_R;
                if d < min && d > 0.001 {
                    if fire_through {
                        // a power shot knocks the player away and keeps going
                        if self.p[i].stun <= 0.0 {
                            self.p[i].stun = FIRE_STUN;
                            self.p[i].vx = b.vx.signum() * FIRE_KNOCK;
                            self.p[i].vy = -350.0;
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
        let scorer = if b.x < GOAL_DEPTH - BALL_R && b.y > BAR_Y + BAR_H {
            Some(1u8)
        } else if b.x > W - GOAL_DEPTH + BALL_R && b.y > BAR_Y + BAR_H {
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
            for p in self.p.iter_mut() {
                p.power_ready = 0.0;
                p.kick_t = -1.0;
            }
            self.events |= EV_GOAL;
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
        out.extend_from_slice(&self.events.to_le_bytes());
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
        let e = r.take(4)?;
        s.events = u32::from_le_bytes([e[0], e[1], e[2], e[3]]);
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
