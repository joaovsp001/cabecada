//! `--self-test <file.json>`: plays scripted moments of a real match and
//! measures the game's own state to see that each part of the loop works.

use crate::game::*;
use crate::net;
use crate::{App, Mode};

pub struct SelfTest {
    stage: usize,
    f: u32,
    steps: Vec<(String, bool, String)>,
    x0: f32,
    y0: f32,
    miny: f32,
    bx0: f32,
}

const DT: f32 = 1.0 / 60.0;

impl SelfTest {
    pub fn new() -> SelfTest {
        SelfTest { stage: 0, f: 0, steps: vec![], x0: 0.0, y0: 0.0, miny: 0.0, bx0: 0.0 }
    }

    fn push(&mut self, step: &str, ok: bool, detail: String) {
        self.steps.push((step.to_string(), ok, detail));
        self.stage += 1;
        self.f = 0;
    }

    fn place_kick(app: &mut App, px: f32) {
        app.sim.phase = Phase::Play;
        for p in app.sim.p.iter_mut() {
            p.reset();
        }
        app.sim.p[0].x = px;
        app.sim.p[1].x = if px > 640.0 { 200.0 } else { 1100.0 };
        app.sim.ball = Ball { x: px + 60.0, y: GROUND - BALL_R, ..Default::default() };
    }

    /// One frame of the script. Returns true when everything ran.
    pub fn tick(&mut self, app: &mut App) -> bool {
        let mut inp = [Input::default(); 2];
        let f = self.f;
        match self.stage {
            0 => {
                app.start_match(Mode::Local2);
                app.sim.phase = Phase::Play;
                self.x0 = app.sim.p[0].x;
                self.stage = 1;
                self.f = 0;
                return false;
            }
            1 => {
                inp[0].right = true;
                if f == 60 {
                    let dx = app.sim.p[0].x - self.x0;
                    self.push("move", dx > 150.0, format!("held right for 1 s: the player ran {:.0} px ({:.1} px/s top speed {})", dx, dx, MOVE_SPEED));
                    return false;
                }
            }
            2 => {
                if f == 0 {
                    self.y0 = app.sim.p[0].y;
                    self.miny = self.y0;
                }
                inp[0].jump = f < 3;
                self.miny = self.miny.min(app.sim.p[0].y);
                if f == 75 {
                    let rise = self.y0 - self.miny;
                    let landed = app.sim.p[0].on_ground;
                    self.push("jump", rise > 120.0 && landed, format!("jumped {:.0} px high and landed again: {}", rise, landed));
                    return false;
                }
            }
            3 => {
                if f == 0 {
                    Self::place_kick(app, 400.0);
                    self.bx0 = app.sim.ball.x;
                }
                inp[0].kick = f == 1;
                if f == 14 {
                    let b = app.sim.ball;
                    let ok = b.vx > 300.0 && b.x > self.bx0 + 40.0 && app.sim.touches[0] > 0;
                    self.push("act", ok, format!("low kick sent the ball {:.0} px forward at {:.0} px/s, {:.0} px/s upward", b.x - self.bx0, b.vx, -b.vy));
                    return false;
                }
            }
            4 => {
                if f == 0 {
                    Self::place_kick(app, 1000.0);
                    app.sim.score = [0, 0];
                }
                inp[0].kick = f == 1;
                if app.sim.score[0] > 0 || f > 150 {
                    let ok = app.sim.score[0] == 1 && app.sim.phase == Phase::Goal;
                    self.push("goal", ok, format!("a kick into the right goal counted: score {} x {}", app.sim.score[0], app.sim.score[1]));
                    return false;
                }
            }
            5 => {
                if f == 0 {
                    Self::place_kick(app, 400.0);
                    app.sim.p[0].power = 1.0;
                }
                inp[0].power = f == 1;
                inp[0].kick = f == 4;
                if f == 9 {
                    let b = app.sim.ball;
                    let sp = (b.vx * b.vx + b.vy * b.vy).sqrt();
                    let ok = b.fire > 0.0 && sp > 1200.0 && app.sim.p[0].power < 0.2;
                    self.push("power", ok, format!("full power bar used: the next kick became a power shot at {:.0} px/s", sp));
                    return false;
                }
            }
            6 => {
                if f == 0 {
                    app.start_match(Mode::Cpu);
                    app.level = 1;
                    app.sim.phase = Phase::Play;
                    app.sim.ball = Ball { x: 760.0, y: 300.0, ..Default::default() };
                    self.x0 = app.sim.p[1].x;
                }
                if f == 150 {
                    let dx = (app.sim.p[1].x - self.x0).abs();
                    let t = app.sim.touches[1];
                    self.push("cpu", dx > 30.0 || t > 0, format!("the computer player moved {:.0} px and touched the ball {} times in 2.5 s", dx, t));
                    return false;
                }
            }
            _ => return true,
        }
        app.update_match(DT, inp);
        app.held = inp;
        self.f += 1;
        false
    }

    pub fn finish(&mut self, app: &mut App, path: &str) {
        let h = app.hud;
        self.steps.push((
            "hud".into(),
            h.score && h.timer && h.power && h.buttons,
            format!("drawn: score {}, match clock {}, power bars {}, touch buttons {}", h.score, h.timer, h.power, h.buttons),
        ));
        let (ok, detail) = lan_test();
        self.steps.push(("lan".into(), ok, detail));

        let mut s = String::from("{\"version\": 1, \"steps\": [\n");
        for (i, (name, ok, d)) in self.steps.iter().enumerate() {
            let d = d.replace('\\', "/").replace('"', "'");
            s.push_str(&format!("  {{\"step\": \"{}\", \"passed\": {}, \"detail\": \"{}\"}}{}\n", name, ok, d, if i + 1 < self.steps.len() { "," } else { "" }));
        }
        s.push_str("]}\n");
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, s);
    }
}

/// Host and joiner on this computer (loopback): the joiner's button press
/// reaches the host and the host's match state reaches the joiner.
fn lan_test() -> (bool, String) {
    let mut host = match net::Host::new(0, "teste") {
        Ok(h) => h,
        Err(e) => return (false, format!("could not open a host socket: {}", e)),
    };
    let mut cl = match net::Client::new() {
        Ok(c) => c,
        Err(e) => return (false, format!("could not open a joiner socket: {}", e)),
    };
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], host.port));
    cl.connect(addr, 0.0);
    let mut sim = Sim::new(90.0);
    sim.score = [2, 1];
    sim.p[0].x = 512.0;
    for i in 0..400 {
        let now = i as f64 * 0.01;
        cl.poll(now);
        host.poll(now);
        if host.peer.is_some() {
            host.send_state(&sim);
        }
        cl.send_input(Input { right: true, kick: true, ..Default::default() });
        let got = cl.latest.as_ref().map(|s| s.score == [2, 1] && (s.p[0].x - 512.0).abs() < 0.01).unwrap_or(false);
        if got && host.remote.right && host.remote.kick {
            return (true, format!("joined a host on port {} with no server: input reached the host and the match state (score 2 x 1) reached the joiner after {} ms", host.port, i * 5));
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    (false, format!("no exchange: joiner welcomed {}, host saw player {}", cl.welcomed, host.peer.is_some()))
}
