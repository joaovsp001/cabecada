//! Computer opponent. It decides every `reaction` seconds and keeps the
//! decision in between, so easier levels react later and make more mistakes.

use crate::game::*;

#[derive(Clone, Copy, Default)]
pub struct AiMem {
    pub timer: f32,
    pub held: Input,
    pub rng: u32,
    pub seen: Ball, // where the ball was last seen (the invisible power hides it)
}

pub struct Level {
    pub reaction: f32,
    pub mistake: f32, // chance to skip a kick or jump
    pub aim: f32,     // how well it stays behind the ball (px of error)
}

pub const LEVELS: [Level; 3] = [
    Level { reaction: 0.26, mistake: 0.35, aim: 40.0 },
    Level { reaction: 0.14, mistake: 0.15, aim: 18.0 },
    Level { reaction: 0.06, mistake: 0.04, aim: 6.0 },
];

impl AiMem {
    fn rand(&mut self) -> f32 {
        if self.rng == 0 {
            self.rng = 0x1234567;
        }
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng % 10000) as f32 / 10000.0
    }
}

pub fn think(sim: &Sim, me: usize, level: usize, mem: &mut AiMem, dt: f32) -> Input {
    let lv = &LEVELS[level.min(2)];
    mem.timer -= dt;
    if mem.timer > 0.0 {
        // keep moving, but buttons that need a fresh press are released
        let mut h = mem.held;
        h.kick = false;
        h.lob = false;
        h.power = false;
        return h;
    }
    mem.timer = lv.reaction;
    let p = sim.p[me];
    let o = sim.p[1 - me];
    let b = if sim.ball.invis > 0.0 && sim.ball.invis_owner as usize != me {
        mem.seen // it can only guess from the last place it saw the ball
    } else {
        mem.seen = sim.ball;
        sim.ball
    };
    let s = p.side;
    let own_goal = if s > 0.0 { 0.0 } else { W };
    let mut inp = Input::default();

    if sim.phase != Phase::Play && sim.phase != Phase::Ready && sim.phase != Phase::Countdown {
        mem.held = inp;
        return inp;
    }

    // where the ball will be soon (simple ballistic guess)
    let t = 0.22;
    let mut px = b.x + b.vx * t;
    px = px.clamp(BALL_R, W - BALL_R);

    // distance from own goal, positive toward the opponent
    let ball_fwd = s * (px - p.x);
    let ball_toward_goal = s * b.vx < -250.0;
    let ball_behind = ball_fwd < -10.0;

    let err = (mem.rand() - 0.5) * 2.0 * lv.aim;
    let mut target = if ball_behind || (ball_toward_goal && s * (px - own_goal) < 520.0) {
        // get back between the ball and our goal
        px - s * (HEAD_R + 30.0)
    } else {
        // stand just behind the ball so touches push it forward
        px - s * (HEAD_R * 0.8)
    };
    // don't stray too far from goal when the opponent has the ball near us
    if s * (o.x - own_goal) < 300.0 && s * (px - own_goal) < 350.0 {
        target = target.min(W).max(0.0);
    }
    target += err;
    let dx = target - p.x;
    if dx > 8.0 {
        inp.right = true;
    } else if dx < -8.0 {
        inp.left = true;
    }

    let dist = ((b.x - p.x).powi(2) + (b.y - p.y).powi(2)).sqrt();
    let ahead = s * (b.x - p.x);
    let skip = mem.rand() < lv.mistake;

    if sim.phase == Phase::Play && !skip {
        // ball near the feet and in front: kick
        if ahead > -5.0 && ahead < 95.0 && b.y > p.y - 10.0 && dist < 120.0 {
            if b.y < p.y + 20.0 && mem.rand() < 0.5 {
                inp.lob = true;
            } else {
                inp.kick = true;
            }
        }
        // ball dropping above us: head it
        if (b.x - p.x).abs() < 70.0 && b.y < p.y - 40.0 && b.y > p.y - 260.0 && b.vy > -100.0 {
            inp.jump = true;
        }
        // ball flying at our goal at head height: block it
        if ball_toward_goal && (b.x - p.x).abs() < 160.0 && b.y < p.y - 20.0 {
            inp.jump = true;
        }
        // opponent close in front: a kick knocks them down
        let oa = s * (o.x - p.x);
        if oa > 0.0 && oa < 90.0 && mem.rand() < 0.25 {
            inp.kick = true;
        }
        if p.power >= 1.0 && dist < 260.0 && ahead > 0.0 {
            inp.power = true;
        }
    }
    mem.held = inp;
    inp
}
