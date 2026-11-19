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
    Level { reaction: 0.26, mistake: 0.35, aim: 46.0 },
    Level { reaction: 0.14, mistake: 0.15, aim: 20.0 },
    Level { reaction: 0.06, mistake: 0.04, aim: 8.0 },
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

    // While frozen in ice, Normal/Hard AI slowly mashes buttons to crack out faster.
    if p.frozen > 0.0 {
        if sim.phase == Phase::Play && level > 0 {
            let mash_chance = if level >= 2 { 0.70 } else { 0.55 };
            if mem.rand() < mash_chance {
                inp.kick = true;
            }
        }
        mem.held = inp;
        return inp;
    }

    // where the ball will be soon (simple ballistic guess)
    let t = 0.20;
    let br = b.r();
    let mut px = b.x + b.vx * t;
    px = px.clamp(br, W - br);

    // distance from own goal, positive toward the opponent
    let ball_fwd = s * (px - p.x);
    let ball_toward_goal = s * b.vx < -320.0;
    let ball_behind = ball_fwd < -BALL_R * 0.45;

    let err = (mem.rand() - 0.5) * 2.0 * lv.aim;
    let mut target = if ball_behind || (ball_toward_goal && s * (px - own_goal) < 540.0) {
        // get back between the ball and our goal
        px - s * (HEAD_R + BALL_R + 12.0)
    } else {
        // stand just behind the ball so touches push it forward
        px - s * (HEAD_R * 0.85)
    };
    // don't stray too far from goal when the opponent has the ball near us
    if s * (o.x - own_goal) < 340.0 && s * (px - own_goal) < 400.0 {
        target = target.clamp(HEAD_R + 4.0, W - HEAD_R - 4.0);
    }
    target += err;
    let dx = target - p.x;
    if dx > 10.0 {
        inp.right = true;
    } else if dx < -10.0 {
        inp.left = true;
    }

    let dist = ((b.x - p.x).powi(2) + (b.y - p.y).powi(2)).sqrt();
    let ahead = s * (b.x - p.x);
    let skip = mem.rand() < lv.mistake;

    if sim.phase == Phase::Play && !skip {
        // ball near the feet and in front: kick
        let kick_reach = HEAD_R + br + FOOT_R + KICK_REACH;
        if ahead > -br * 0.25 && ahead < kick_reach && b.y > p.y - HEAD_R * 0.25 && dist < kick_reach + 32.0 {
            if b.y < p.y + BODY_OFF * 0.65 && mem.rand() < 0.5 {
                inp.lob = true;
            } else {
                inp.kick = true;
            }
        }
        // ball dropping above us: head it (scaled to 258 px jump rise)
        if (b.x - p.x).abs() < HEAD_R + br + 12.0 && b.y < p.y - HEAD_R * 0.9 && b.y > p.y - 340.0 && b.vy > -140.0 {
            inp.jump = true;
        }
        // ball flying at our goal at head height: block it
        if ball_toward_goal && (b.x - p.x).abs() < 210.0 && b.y < p.y - HEAD_R * 0.35 && b.y > BAR_Y - br {
            inp.jump = true;
        }
        // opponent close in front: a kick knocks them down (skip if already frozen)
        let oa = s * (o.x - p.x);
        if oa > 0.0 && oa < HEAD_R * 2.0 + 18.0 && (o.y - p.y).abs() < FEET && o.frozen <= 0.0 && mem.rand() < 0.25 {
            inp.kick = true;
        }
        if p.power >= 1.0 {
            let use_now = match p.pkind {
                PK_ICE => o.frozen <= 0.0 && mem.rand() < 0.45,
                PK_BIG | PK_INVIS => dist < 380.0,
                _ => dist < 320.0 && ahead > -br,
            };
            if use_now {
                inp.power = true;
            }
        }
    }
    mem.held = inp;
    inp
}
