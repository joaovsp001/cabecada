//! Cabecada: 1 v 1 big-head football that plays offline.
//! Modes: against the computer, two players on one device, and local-network
//! play where one player's device hosts the match (no internet server).

mod ai;
mod audio;
mod game;
mod net;
mod selftest;
mod win;

use game::*;
use macroquad::prelude::*;
use std::cell::Cell;
use std::net::{IpAddr, SocketAddr};

pub const STEP: f32 = 1.0 / 120.0;

// ---------------------------------------------------------------- drawing in virtual 1280x720

thread_local! {
    static VIEW: Cell<(f32, f32, f32)> = const { Cell::new((1.0, 0.0, 0.0)) };
}

pub fn set_view() {
    let sw = screen_width();
    let sh = screen_height();
    let s = (sw / W).min(sh / H);
    VIEW.with(|v| v.set((s, (sw - W * s) * 0.5, (sh - H * s) * 0.5)));
}
pub fn view() -> (f32, f32, f32) {
    VIEW.with(|v| v.get())
}
pub fn to_virtual(x: f32, y: f32) -> (f32, f32) {
    let (s, ox, oy) = view();
    ((x - ox) / s, (y - oy) / s)
}
fn px(x: f32) -> f32 {
    let (s, ox, _) = view();
    x * s + ox
}
fn py(y: f32) -> f32 {
    let (s, _, oy) = view();
    y * s + oy
}
fn sc(v: f32) -> f32 {
    view().0 * v
}
pub fn circ(x: f32, y: f32, r: f32, c: Color) {
    draw_circle(px(x), py(y), sc(r), c);
}
pub fn circ_line(x: f32, y: f32, r: f32, t: f32, c: Color) {
    draw_circle_lines(px(x), py(y), sc(r), sc(t), c);
}
pub fn rectv(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_rectangle(px(x), py(y), sc(w), sc(h), c);
}
pub fn rect_line(x: f32, y: f32, w: f32, h: f32, t: f32, c: Color) {
    draw_rectangle_lines(px(x), py(y), sc(w), sc(h), sc(t), c);
}
pub fn linev(x1: f32, y1: f32, x2: f32, y2: f32, t: f32, c: Color) {
    draw_line(px(x1), py(y1), px(x2), py(y2), sc(t), c);
}
pub fn ellipse(x: f32, y: f32, w: f32, h: f32, c: Color) {
    draw_ellipse(px(x), py(y), sc(w), sc(h), 0.0, c);
}
pub fn tri(a: (f32, f32), b: (f32, f32), c: (f32, f32), col: Color) {
    draw_triangle(vec2(px(a.0), py(a.1)), vec2(px(b.0), py(b.1)), vec2(px(c.0), py(c.1)), col);
}
pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let r = r.min(w * 0.5).min(h * 0.5);
    rectv(x + r, y, w - 2.0 * r, h, c);
    rectv(x, y + r, r, h - 2.0 * r, c);
    rectv(x + w - r, y + r, r, h - 2.0 * r, c);
    circ(x + r, y + r, r, c);
    circ(x + w - r, y + r, r, c);
    circ(x + r, y + h - r, r, c);
    circ(x + w - r, y + h - r, r, c);
}
pub fn text_w(t: &str, size: f32) -> f32 {
    let fs = sc(size).max(1.0);
    measure_text(t, None, fs as u16, 1.0).width / view().0
}
pub fn text(t: &str, x: f32, y: f32, size: f32, c: Color) {
    let fs = sc(size).max(1.0);
    draw_text(t, px(x), py(y), fs, c);
}
pub fn text_c(t: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = text_w(t, size);
    text(t, cx - w * 0.5, y, size, c);
}
/// Centered text with a dark outline so it reads on any background.
pub fn text_o(t: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = text_w(t, size);
    let o = (size / 18.0).max(1.5);
    for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (o, o)] {
        text(t, cx - w * 0.5 + dx, y + dy, size, Color::new(0.05, 0.05, 0.1, 0.9));
    }
    text(t, cx - w * 0.5, y, size, c);
}

fn hex(v: u32) -> Color {
    Color::from_rgba((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}
fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}
fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

// ---------------------------------------------------------------- input

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Btn {
    Left,
    Right,
    Jump,
    Kick,
    Lob,
    Power,
}

#[derive(Clone, Copy)]
pub struct TouchBtn {
    pub who: usize,
    pub kind: Btn,
    pub x: f32,
    pub y: f32,
    pub r: f32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Cpu,
    Local2,
    Host,
    Client,
}

pub fn touch_layout(mode: Mode) -> Vec<TouchBtn> {
    let mut v = vec![];
    let b = |who, kind, x, y, r| TouchBtn { who, kind, x, y, r };
    match mode {
        Mode::Local2 => {
            for who in 0..2 {
                let m = |x: f32| if who == 0 { x } else { W - x };
                let (lx, rx) = if who == 0 { (55.0, 150.0) } else { (W - 150.0, W - 55.0) };
                v.push(b(who, Btn::Left, lx, 662.0, 44.0));
                v.push(b(who, Btn::Right, rx, 662.0, 44.0));
                v.push(b(who, Btn::Kick, m(265.0), 662.0, 46.0));
                v.push(b(who, Btn::Lob, m(370.0), 672.0, 36.0));
                v.push(b(who, Btn::Jump, m(265.0), 560.0, 40.0));
                v.push(b(who, Btn::Power, m(370.0), 590.0, 32.0));
            }
        }
        _ => {
            let who = if mode == Mode::Client { 1 } else { 0 };
            v.push(b(who, Btn::Left, 95.0, 640.0, 62.0));
            v.push(b(who, Btn::Right, 240.0, 640.0, 62.0));
            v.push(b(who, Btn::Kick, 1182.0, 632.0, 64.0));
            v.push(b(who, Btn::Lob, 1045.0, 660.0, 50.0));
            v.push(b(who, Btn::Jump, 1182.0, 490.0, 54.0));
            v.push(b(who, Btn::Power, 1050.0, 530.0, 42.0));
        }
    }
    v
}

/// Every finger (and a held mouse button) in virtual coordinates.
fn pointers() -> Vec<(f32, f32)> {
    let mut v: Vec<(f32, f32)> = touches()
        .iter()
        .filter(|t| t.phase != TouchPhase::Ended && t.phase != TouchPhase::Cancelled)
        .map(|t| to_virtual(t.position.x, t.position.y))
        .collect();
    if is_mouse_button_down(MouseButton::Left) {
        let (mx, my) = mouse_position();
        v.push(to_virtual(mx, my));
    }
    v
}

fn set_btn(i: &mut Input, k: Btn) {
    match k {
        Btn::Left => i.left = true,
        Btn::Right => i.right = true,
        Btn::Jump => i.jump = true,
        Btn::Kick => i.kick = true,
        Btn::Lob => i.lob = true,
        Btn::Power => i.power = true,
    }
}

fn keys_a() -> Input {
    Input {
        left: is_key_down(KeyCode::A),
        right: is_key_down(KeyCode::D),
        jump: is_key_down(KeyCode::W),
        kick: is_key_down(KeyCode::F),
        lob: is_key_down(KeyCode::G),
        power: is_key_down(KeyCode::R),
    }
}
fn keys_b() -> Input {
    Input {
        left: is_key_down(KeyCode::Left),
        right: is_key_down(KeyCode::Right),
        jump: is_key_down(KeyCode::Up),
        kick: is_key_down(KeyCode::K) || is_key_down(KeyCode::Kp1),
        lob: is_key_down(KeyCode::L) || is_key_down(KeyCode::Kp2),
        power: is_key_down(KeyCode::O) || is_key_down(KeyCode::Kp3),
    }
}
fn keys_single() -> Input {
    let extra = Input {
        jump: is_key_down(KeyCode::Space),
        kick: is_key_down(KeyCode::J),
        ..Default::default()
    };
    keys_a().or(keys_b()).or(extra)
}

// ---------------------------------------------------------------- app

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Screen {
    Menu,
    HostLobby,
    JoinLobby,
    Match,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    r: f32,
    c: Color,
}

#[derive(Default, Clone, Copy)]
pub struct HudFlags {
    pub score: bool,
    pub timer: bool,
    pub power: bool,
    pub buttons: bool,
}

pub struct App {
    pub screen: Screen,
    pub mode: Mode,
    pub sim: Sim,
    pub ai: [ai::AiMem; 2],
    pub ai_both: bool,
    pub level: usize,
    pub len_idx: usize,
    pub host: Option<net::Host>,
    pub client: Option<net::Client>,
    acc: f32,
    pub now: f64,
    paused: bool,
    over_t: f32,
    sounds: Option<audio::Sounds>,
    sound_on: bool,
    ip_text: String,
    status: String,
    connect_t: f64,
    my_ip: String,
    particles: Vec<Particle>,
    shake: f32,
    goal_anim: f32,
    pub hud: HudFlags,
    pub held: [Input; 2],
    lost: bool,
    crowd: Vec<(f32, f32, Color, f32)>,
}

const LEVEL_NAMES: [&str; 3] = ["Facil", "Normal", "Dificil"];

impl App {
    fn new(sounds: Option<audio::Sounds>) -> App {
        let mut crowd = vec![];
        let mut r = 12345u32;
        let mut rnd = || {
            r ^= r << 13;
            r ^= r >> 17;
            r ^= r << 5;
            (r % 10000) as f32 / 10000.0
        };
        let palette = [0xe8d35a, 0xd9533b, 0x4aa3df, 0xf0f0f0, 0x60c060, 0xf29a38, 0x9a6ad6, 0x2f3c66];
        let mut y = 268.0;
        while y < 460.0 {
            let mut x = 4.0 + rnd() * 10.0;
            while x < W {
                let c = hex(palette[(rnd() * palette.len() as f32) as usize % palette.len()]);
                crowd.push((x, y, c, rnd() * 6.28));
                x += 15.0 + rnd() * 7.0;
            }
            y += 22.0;
        }
        App {
            screen: Screen::Menu,
            mode: Mode::Cpu,
            sim: Sim::new(90.0),
            ai: [ai::AiMem { rng: 77, ..Default::default() }, ai::AiMem { rng: 991, ..Default::default() }],
            ai_both: false,
            level: 1,
            len_idx: 1,
            host: None,
            client: None,
            acc: 0.0,
            now: 0.0,
            paused: false,
            over_t: 0.0,
            sounds,
            sound_on: true,
            ip_text: String::new(),
            status: String::new(),
            connect_t: 0.0,
            my_ip: String::new(),
            particles: vec![],
            shake: 0.0,
            goal_anim: 0.0,
            hud: HudFlags::default(),
            held: [Input::default(); 2],
            lost: false,
            crowd,
        }
    }

    pub fn start_match(&mut self, mode: Mode) {
        self.mode = mode;
        self.sim = Sim::new(MATCH_LENGTHS[self.len_idx]);
        self.screen = Screen::Match;
        self.paused = false;
        self.over_t = 0.0;
        self.acc = 0.0;
        self.lost = false;
        self.particles.clear();
    }

    fn leave_network(&mut self) {
        if let Some(h) = self.host.as_mut() {
            h.bye();
        }
        if let Some(c) = self.client.as_mut() {
            c.bye();
        }
        self.host = None;
        self.client = None;
    }

    fn to_menu(&mut self) {
        self.leave_network();
        self.screen = Screen::Menu;
        self.paused = false;
    }

    /// Which players this device controls.
    pub fn local_players(&self) -> Vec<usize> {
        match self.mode {
            Mode::Cpu | Mode::Host => vec![0],
            Mode::Client => vec![1],
            Mode::Local2 => vec![0, 1],
        }
    }

    fn gather_input(&mut self) -> [Input; 2] {
        let mut inp = [Input::default(); 2];
        let pts = pointers();
        for b in touch_layout(self.mode) {
            if pts.iter().any(|&(x, y)| (x - b.x).powi(2) + (y - b.y).powi(2) < (b.r * 1.2).powi(2)) {
                set_btn(&mut inp[b.who], b.kind);
            }
        }
        if self.mode == Mode::Local2 {
            inp[0] = inp[0].or(keys_a());
            inp[1] = inp[1].or(keys_b());
        } else {
            let me = self.local_players()[0];
            inp[me] = inp[me].or(keys_single());
        }
        self.held = inp;
        inp
    }

    fn play_events(&mut self, ev: u32) {
        if ev & EV_KICK != 0 {
            self.burst(self.sim.ball.x, self.sim.ball.y, 8, WHITE);
        }
        if ev & EV_GOAL != 0 {
            self.goal_anim = 0.0;
            let gx = if self.sim.last_scorer == 0 { W - 50.0 } else { 50.0 };
            for c in [hex(0xffd34d), hex(0xff5a5a), hex(0x5ad1ff), WHITE] {
                self.burst(gx, 470.0, 14, c);
            }
        }
        if ev & EV_STUN != 0 {
            self.shake = 0.25;
        }
        if ev & EV_FIRE != 0 {
            self.shake = 0.15;
            self.burst(self.sim.ball.x, self.sim.ball.y, 16, hex(0xffa020));
        }
        let Some(s) = self.sounds.as_ref() else { return };
        if !self.sound_on {
            return;
        }
        let list = [
            (EV_KICK, &s.kick, 0.9),
            (EV_HEAD, &s.head, 0.8),
            (EV_BOUNCE, &s.bounce, 0.5),
            (EV_POST, &s.post, 0.7),
            (EV_WHISTLE, &s.whistle, 0.6),
            (EV_GOAL, &s.goal, 0.8),
            (EV_BEEP, &s.beep, 0.6),
            (EV_POWER, &s.power, 0.7),
            (EV_FIRE, &s.power, 0.9),
            (EV_STUN, &s.stun, 0.7),
        ];
        for (bit, snd, vol) in list {
            if ev & bit != 0 {
                s.play(snd, vol);
            }
        }
    }

    fn burst(&mut self, x: f32, y: f32, n: usize, c: Color) {
        for i in 0..n {
            let a = i as f32 / n as f32 * 6.283 + rand::gen_range(0.0, 0.6);
            let sp = rand::gen_range(120.0, 380.0);
            self.particles.push(Particle { x, y, vx: a.cos() * sp, vy: a.sin() * sp - 100.0, life: 0.0, max: rand::gen_range(0.3, 0.7), r: rand::gen_range(2.5, 5.5), c });
        }
    }

    /// Advance the match by real time `dt`.
    pub fn update_match(&mut self, dt: f32, inp: [Input; 2]) {
        let mut ev = 0;
        match self.mode {
            Mode::Client => {
                let c = self.client.as_mut().unwrap();
                c.poll(self.now);
                c.send_input(inp[1]);
                if let Some(s) = c.latest.clone() {
                    self.sim = s;
                }
                ev |= std::mem::take(&mut c.events);
                if c.host_left || !c.connected(self.now) {
                    self.lost = true;
                }
            }
            _ => {
                let mut inputs = inp;
                if self.mode == Mode::Host {
                    let h = self.host.as_mut().unwrap();
                    h.poll(self.now);
                    if h.connected(self.now) {
                        inputs[1] = h.remote;
                    } else {
                        inputs[1] = Input::default();
                        self.lost = true;
                    }
                }
                let run = !(self.paused && matches!(self.mode, Mode::Cpu | Mode::Local2)) && !self.lost;
                if run {
                    self.acc += dt.min(0.1);
                    while self.acc >= STEP {
                        self.acc -= STEP;
                        let mut i = inputs;
                        if self.mode == Mode::Cpu {
                            i[1] = ai::think(&self.sim, 1, self.level, &mut self.ai[1], STEP);
                            if self.ai_both {
                                i[0] = ai::think(&self.sim, 0, self.level, &mut self.ai[0], STEP);
                            }
                        }
                        self.sim.step(STEP, i);
                    }
                }
                ev = self.sim.events;
                if let Some(h) = self.host.as_mut() {
                    h.send_state(&self.sim);
                }
                self.sim.events = 0;
            }
        }
        if self.sim.phase == Phase::Over {
            self.over_t += dt;
        } else {
            self.over_t = 0.0;
        }
        self.play_events(ev);
        // effects
        self.shake = (self.shake - dt).max(0.0);
        self.goal_anim += dt;
        for p in self.particles.iter_mut() {
            p.life += dt;
            p.vy += 600.0 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.life < p.max);
        if self.sim.ball.fire > 0.0 && rand::gen_range(0.0, 1.0) < 0.8 {
            let b = self.sim.ball;
            self.particles.push(Particle { x: b.x, y: b.y, vx: -b.vx * 0.1 + rand::gen_range(-40.0, 40.0), vy: -b.vy * 0.1 - 60.0, life: 0.0, max: 0.35, r: rand::gen_range(6.0, 12.0), c: hex(0xff8a1a) });
        }
    }

    // ------------------------------------------------------------ frame

    async fn frame(&mut self, dt: f32) {
        self.now += dt as f64;
        set_view();
        clear_background(hex(0x0b1022));
        match self.screen {
            Screen::Menu => self.menu(),
            Screen::HostLobby => self.host_lobby(),
            Screen::JoinLobby => self.join_lobby(),
            Screen::Match => {
                let inp = self.gather_input();
                self.update_match(dt, inp);
                self.draw_match();
                self.match_overlays();
            }
        }
    }

    fn clicked(&self) -> Option<(f32, f32)> {
        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = mouse_position();
            return Some(to_virtual(x, y));
        }
        None
    }

    fn button(&self, label: &str, x: f32, y: f32, w: f32, h: f32, col: Color) -> bool {
        let (mx, my) = to_virtual(mouse_position().0, mouse_position().1);
        let hover = mx >= x && mx <= x + w && my >= y && my <= y + h;
        rrect(x, y + 5.0, w, h, 16.0, with_a(BLACK, 0.35));
        rrect(x, y, w, h, 16.0, if hover { mix(col, WHITE, 0.15) } else { col });
        rrect(x + 4.0, y + 4.0, w - 8.0, h * 0.42, 12.0, with_a(WHITE, 0.12));
        text_o(label, x + w * 0.5, y + h * 0.5 + h * 0.17, h * 0.42, WHITE);
        if let Some((cx, cy)) = self.clicked() {
            return cx >= x && cx <= x + w && cy >= y && cy <= y + h;
        }
        false
    }

    fn draw_backdrop(&self) {
        self.draw_field(false);
        rectv(0.0, 0.0, W, H, with_a(hex(0x0a0f24), 0.62));
    }

    fn menu(&mut self) {
        self.draw_backdrop();
        // title
        let t = self.now as f32;
        circ(640.0, 120.0 + (t * 2.0).sin() * 4.0, 34.0, WHITE);
        draw_ball_spots(640.0, 120.0 + (t * 2.0).sin() * 4.0, 34.0, t);
        text_o("CABECADA", 640.0, 215.0, 88.0, hex(0xffd34d));
        text_o("futebol de cabeca 1 contra 1  -  sem internet, sem servidor", 640.0, 252.0, 24.0, hex(0xcfe3ff));

        let bx = 390.0;
        let bw = 500.0;
        if self.button("Contra o computador", bx, 285.0, bw, 66.0, hex(0x2f8f4e)) {
            self.start_match(Mode::Cpu);
            return;
        }
        if self.button("2 jogadores neste aparelho", bx, 365.0, bw, 66.0, hex(0x2d6fd0)) {
            self.start_match(Mode::Local2);
            return;
        }
        if self.button("Criar partida (rede local)", bx, 445.0, bw, 66.0, hex(0xd06a2d)) {
            let ip = net::local_ip().map(|i| i.to_string()).unwrap_or_else(|| "sem rede".into());
            let name = format!("Partida {}", ip);
            let h = net::Host::new(net::PORT, &name).or_else(|_| net::Host::new(0, &name));
            match h {
                Ok(h) => {
                    self.my_ip = if h.port == net::PORT { ip } else { format!("{}:{}", ip, h.port) };
                    self.host = Some(h);
                    self.screen = Screen::HostLobby;
                }
                Err(e) => self.status = format!("Nao foi possivel abrir a partida: {}", e),
            }
            return;
        }
        if self.button("Entrar em partida (rede local)", bx, 525.0, bw, 66.0, hex(0x8a3fc0)) {
            match net::Client::new() {
                Ok(c) => {
                    self.client = Some(c);
                    self.status.clear();
                    self.screen = Screen::JoinLobby;
                }
                Err(e) => self.status = format!("Rede indisponivel: {}", e),
            }
            return;
        }
        let small = hex(0x34405e);
        if self.button(&format!("Computador: {}", LEVEL_NAMES[self.level]), 170.0, 618.0, 300.0, 52.0, small) {
            self.level = (self.level + 1) % 3;
        }
        if self.button(&format!("Tempo: {} s", MATCH_LENGTHS[self.len_idx] as i32), 490.0, 618.0, 300.0, 52.0, small) {
            self.len_idx = (self.len_idx + 1) % 3;
        }
        if self.button(if self.sound_on { "Som: ligado" } else { "Som: desligado" }, 810.0, 618.0, 300.0, 52.0, small) {
            self.sound_on = !self.sound_on;
        }
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 700.0, 22.0, hex(0xff8080));
        }
    }

    fn host_lobby(&mut self) {
        self.draw_backdrop();
        let now = self.now;
        let joined = self.host.as_mut().map(|h| h.poll(now)).unwrap_or(false);
        text_o("Partida aberta neste aparelho", 640.0, 130.0, 48.0, hex(0xffd34d));
        rrect(300.0, 165.0, 680.0, 120.0, 20.0, with_a(BLACK, 0.45));
        text_o("Endereco deste aparelho:", 640.0, 205.0, 26.0, hex(0xcfe3ff));
        text_o(&self.my_ip, 640.0, 262.0, 54.0, WHITE);
        let dots = ".".repeat(((now * 2.0) as usize) % 4);
        text_o(&format!("Aguardando o outro jogador{}", dots), 640.0, 345.0, 34.0, WHITE);
        let lines = [
            "Os dois aparelhos precisam estar na mesma rede Wi-Fi,",
            "ou um conectado ao roteador (hotspot) do outro. Nao usa internet.",
            "No outro aparelho: Entrar em partida. Ela aparece sozinha na lista,",
            "ou digite o endereco acima.",
        ];
        for (i, l) in lines.iter().enumerate() {
            text_o(l, 640.0, 400.0 + i as f32 * 32.0, 24.0, hex(0xb8c6e6));
        }
        if joined || self.host.as_ref().map(|h| h.connected(now)).unwrap_or(false) {
            self.start_match(Mode::Host);
            return;
        }
        if self.button("Voltar", 490.0, 580.0, 300.0, 64.0, hex(0x5a2d3a)) {
            self.to_menu();
        }
    }

    fn join_lobby(&mut self) {
        self.draw_backdrop();
        let now = self.now;
        if let Some(c) = self.client.as_mut() {
            c.poll(now);
        }
        text_o("Entrar em partida", 640.0, 80.0, 48.0, hex(0xffd34d));
        let (connecting, welcomed, has_state) = match &self.client {
            Some(c) => (c.host.is_some(), c.welcomed, c.latest.is_some()),
            None => (false, false, false),
        };
        if welcomed && has_state {
            self.start_match(Mode::Client);
            if let Some(s) = self.client.as_ref().and_then(|c| c.latest.clone()) {
                self.sim = s;
            }
            return;
        }
        if connecting && now - self.connect_t > 6.0 {
            if let Some(c) = self.client.as_mut() {
                c.host = None;
            }
            self.status = "Nao foi possivel conectar. Confira o endereco e a rede.".into();
        }

        // found matches
        rrect(60.0, 110.0, 560.0, 470.0, 20.0, with_a(BLACK, 0.45));
        text_o("Partidas encontradas na rede", 340.0, 150.0, 28.0, WHITE);
        let found: Vec<(SocketAddr, String)> = self.client.as_ref().map(|c| c.found.iter().map(|f| (f.addr, f.name.clone())).collect()).unwrap_or_default();
        if found.is_empty() {
            let dots = ".".repeat(((now * 2.0) as usize) % 4);
            text_o(&format!("Procurando{}", dots), 340.0, 220.0, 26.0, hex(0xb8c6e6));
            text_o("Se nao aparecer, digite o endereco", 340.0, 270.0, 22.0, hex(0x93a3c8));
            text_o("mostrado no aparelho que criou a partida.", 340.0, 298.0, 22.0, hex(0x93a3c8));
        }
        let mut pick = None;
        for (i, (addr, name)) in found.iter().enumerate().take(5) {
            if self.button(name, 90.0, 180.0 + i as f32 * 76.0, 500.0, 62.0, hex(0x2f8f4e)) {
                pick = Some(*addr);
            }
        }

        // manual address
        rrect(660.0, 110.0, 560.0, 470.0, 20.0, with_a(BLACK, 0.45));
        text_o("Ou digite o endereco", 940.0, 150.0, 28.0, WHITE);
        rrect(690.0, 168.0, 500.0, 60.0, 12.0, hex(0x10162c));
        let caret = if (now * 2.0) as i64 % 2 == 0 { "_" } else { " " };
        text_o(&format!("{}{}", self.ip_text, caret), 940.0, 212.0, 38.0, WHITE);
        while let Some(ch) = get_char_pressed() {
            if (ch.is_ascii_digit() || ch == '.' || ch == ':') && self.ip_text.len() < 21 {
                self.ip_text.push(ch);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.ip_text.pop();
        }
        let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "0", "<"];
        for (i, k) in keys.iter().enumerate() {
            let cx = 700.0 + (i % 3) as f32 * 165.0;
            let cy = 242.0 + (i / 3) as f32 * 62.0;
            if self.button(k, cx, cy, 150.0, 54.0, hex(0x34405e)) {
                if *k == "<" {
                    self.ip_text.pop();
                } else if self.ip_text.len() < 21 {
                    self.ip_text.push_str(k);
                }
            }
        }
        if self.button("Conectar", 700.0, 500.0, 480.0, 62.0, hex(0xd06a2d)) || is_key_pressed(KeyCode::Enter) {
            let t = self.ip_text.trim();
            let addr = if t.contains(':') { t.parse::<SocketAddr>().ok() } else { t.parse::<IpAddr>().ok().map(|ip| SocketAddr::new(ip, net::PORT)) };
            match addr {
                Some(a) => pick = Some(a),
                None => self.status = "Endereco invalido (exemplo: 192.168.0.12)".into(),
            }
        }
        if let Some(a) = pick {
            if let Some(c) = self.client.as_mut() {
                c.connect(a, now);
            }
            self.connect_t = now;
            self.status = format!("Conectando a {}...", a);
        }
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 615.0, 24.0, hex(0xffd0a0));
        }
        if self.button("Voltar", 490.0, 640.0, 300.0, 60.0, hex(0x5a2d3a)) {
            self.to_menu();
        }
    }

    fn player_name(&self, i: usize) -> &'static str {
        match self.mode {
            Mode::Cpu => {
                if i == 0 {
                    if self.ai_both { "CPU 1" } else { "VOCE" }
                } else {
                    "CPU"
                }
            }
            Mode::Local2 => {
                if i == 0 { "JOGADOR 1" } else { "JOGADOR 2" }
            }
            Mode::Host => {
                if i == 0 { "VOCE" } else { "AMIGO" }
            }
            Mode::Client => {
                if i == 1 { "VOCE" } else { "AMIGO" }
            }
        }
    }

    fn match_overlays(&mut self) {
        let sim = self.sim.clone();
        // pause button
        let pause_hit = self.clicked().map(|(x, y)| (x - 640.0).powi(2) + (y - 112.0).powi(2) < 26.0f32.powi(2)).unwrap_or(false);
        circ(640.0, 112.0, 20.0, with_a(BLACK, 0.45));
        rectv(632.0, 102.0, 5.0, 20.0, WHITE);
        rectv(643.0, 102.0, 5.0, 20.0, WHITE);
        if (pause_hit || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P)) && !self.paused {
            self.paused = true;
            return;
        }

        if self.lost {
            rectv(0.0, 0.0, W, H, with_a(BLACK, 0.6));
            let msg = if self.mode == Mode::Host { "O outro jogador saiu ou perdeu a conexao" } else { "Conexao com a partida perdida" };
            text_o(msg, 640.0, 300.0, 40.0, WHITE);
            if self.mode == Mode::Host {
                if self.button("Esperar de novo", 340.0, 380.0, 280.0, 64.0, hex(0x2f8f4e)) {
                    if let Some(h) = self.host.as_mut() {
                        h.peer = None;
                    }
                    self.lost = false;
                    self.screen = Screen::HostLobby;
                    return;
                }
                if self.button("Menu", 660.0, 380.0, 280.0, 64.0, hex(0x5a2d3a)) {
                    self.to_menu();
                }
            } else if self.button("Menu", 490.0, 380.0, 300.0, 64.0, hex(0x5a2d3a)) {
                self.to_menu();
            }
            return;
        }

        if self.paused {
            rectv(0.0, 0.0, W, H, with_a(BLACK, 0.6));
            text_o("Pausa", 640.0, 250.0, 64.0, WHITE);
            if matches!(self.mode, Mode::Host | Mode::Client) {
                text_o("(a partida em rede continua rodando)", 640.0, 290.0, 24.0, hex(0xb8c6e6));
            }
            if self.button("Continuar", 490.0, 330.0, 300.0, 64.0, hex(0x2f8f4e)) || is_key_pressed(KeyCode::Escape) {
                self.paused = false;
            }
            if self.button("Sair para o menu", 490.0, 415.0, 300.0, 64.0, hex(0x5a2d3a)) {
                self.to_menu();
            }
            return;
        }

        if sim.phase == Phase::Over && self.over_t > 1.2 {
            rectv(0.0, 0.0, W, H, with_a(BLACK, 0.55));
            let mine = self.local_players();
            let title = match sim.winner() {
                None => "EMPATE".to_string(),
                Some(w) => {
                    if self.mode == Mode::Local2 || self.ai_both {
                        format!("{} VENCEU!", self.player_name(w))
                    } else if mine.contains(&w) {
                        "VITORIA!".to_string()
                    } else {
                        "DERROTA".to_string()
                    }
                }
            };
            text_o(&title, 640.0, 250.0, 80.0, hex(0xffd34d));
            text_o(&format!("{}  {} x {}  {}", self.player_name(0), sim.score[0], sim.score[1], self.player_name(1)), 640.0, 310.0, 36.0, WHITE);
            if self.mode == Mode::Client {
                text_o("Aguardando o anfitriao comecar outra partida...", 640.0, 370.0, 26.0, hex(0xb8c6e6));
                if self.button("Menu", 490.0, 420.0, 300.0, 64.0, hex(0x5a2d3a)) {
                    self.to_menu();
                }
            } else {
                if self.button("Jogar de novo", 340.0, 380.0, 280.0, 64.0, hex(0x2f8f4e)) {
                    let m = self.mode;
                    self.start_match(m);
                    if let Some(h) = self.host.as_mut() {
                        h.send_rematch();
                    }
                }
                if self.button("Menu", 660.0, 380.0, 280.0, 64.0, hex(0x5a2d3a)) {
                    self.to_menu();
                }
            }
        }
    }

    // ------------------------------------------------------------ match drawing

    fn draw_field(&self, live: bool) {
        let t = self.now as f32;
        // sky
        let top = hex(0x2a3f8f);
        let bot = hex(0xf2a65a);
        for i in 0..24 {
            let k = i as f32 / 24.0;
            rectv(0.0, k * 270.0, W, 270.0 / 24.0 + 1.0, mix(top, bot, k));
        }
        circ(1040.0, 150.0, 46.0, with_a(hex(0xffe9a8), 0.9));
        // stand roof and floodlights
        rectv(0.0, 236.0, W, 18.0, hex(0x2a2f45));
        for lx in [150.0, 1130.0] {
            rectv(lx - 4.0, 120.0, 8.0, 120.0, hex(0x3b4260));
            rrect(lx - 40.0, 96.0, 80.0, 30.0, 6.0, hex(0x3b4260));
            for j in 0..4 {
                circ(lx - 27.0 + j as f32 * 18.0, 111.0, 6.0, hex(0xfff6d0));
            }
        }
        // stands with crowd
        rectv(0.0, 254.0, W, 216.0, hex(0x2c3350));
        for row in 0..9 {
            rectv(0.0, 262.0 + row as f32 * 22.0, W, 2.0, hex(0x363e60));
        }
        let cheer = live && (self.sim.phase == Phase::Goal || self.sim.phase == Phase::Over);
        for &(x, y, c, ph) in &self.crowd {
            let bob = if cheer { ((t * 14.0 + ph).sin()).abs() * 6.0 } else { (t * 2.0 + ph).sin() * 1.0 };
            circ(x, y - bob, 6.0, mix(c, hex(0x2c3350), 0.35));
            circ(x, y - 9.0 - bob, 4.5, mix(hex(0xe0b48a), hex(0x2c3350), 0.35));
        }
        // boards
        rectv(0.0, 470.0, W, 36.0, hex(0x1d2236));
        let board = [0x2d6fd0, 0xd0402d, 0x2f8f4e, 0xe0a020];
        for i in 0..8 {
            let x = i as f32 * 160.0 + 6.0;
            rectv(x, 474.0, 148.0, 28.0, hex(board[i % 4]));
            rectv(x + 14.0, 484.0, 120.0, 8.0, with_a(WHITE, 0.55));
        }
        // pitch
        for i in 0..10 {
            let c = if i % 2 == 0 { hex(0x3f9a3c) } else { hex(0x4aa845) };
            rectv(i as f32 * 128.0, 506.0, 128.0, 214.0, c);
        }
        rectv(0.0, 506.0, W, 3.0, with_a(WHITE, 0.6));
        rectv(638.0, 506.0, 4.0, 214.0, with_a(WHITE, 0.6));
        draw_ellipse_lines(px(640.0), py(600.0), sc(150.0), sc(40.0), 0.0, sc(3.0), with_a(WHITE, 0.6));
        rectv(0.0, GROUND, W, 2.0, with_a(BLACK, 0.12));
        // goal nets (behind players)
        for left in [true, false] {
            let x0 = if left { 0.0 } else { W - GOAL_DEPTH };
            rectv(x0, BAR_Y + BAR_H, GOAL_DEPTH, GROUND - BAR_Y - BAR_H, with_a(hex(0x10161a), 0.35));
            let mut gx = x0;
            while gx <= x0 + GOAL_DEPTH {
                linev(gx, BAR_Y + BAR_H, gx, GROUND, 1.5, with_a(WHITE, 0.45));
                gx += 14.0;
            }
            let mut gy = BAR_Y + BAR_H;
            while gy <= GROUND {
                linev(x0, gy, x0 + GOAL_DEPTH, gy, 1.5, with_a(WHITE, 0.45));
                gy += 14.0;
            }
        }
    }

    fn draw_goal_frames(&self) {
        for left in [true, false] {
            let bx = if left { 0.0 } else { W - BAR_LEN };
            rectv(bx, BAR_Y + 4.0, BAR_LEN, BAR_H, with_a(BLACK, 0.3));
            rectv(bx, BAR_Y, BAR_LEN, BAR_H, hex(0xf4f4f4));
            rectv(bx, BAR_Y + BAR_H - 3.0, BAR_LEN, 3.0, hex(0xc8c8c8));
            let post = if left { GOAL_DEPTH - 3.0 } else { W - GOAL_DEPTH - 3.0 };
            rectv(post, BAR_Y, 6.0, GROUND - BAR_Y, with_a(hex(0xf4f4f4), 0.85));
        }
    }

    fn draw_player(&self, i: usize) {
        let p = self.sim.p[i];
        let s = p.side;
        let (shirt, shirt2, hair, skin, boot) = if i == 0 {
            (hex(0x2d6fd0), hex(0x1d4c96), hex(0x3a2418), hex(0xf0c397), hex(0x15171f))
        } else {
            (hex(0xd8432f), hex(0x9c2a1d), hex(0xe9c75a), hex(0xd99a6c), hex(0xf2f2f2))
        };
        let x = p.x;
        let y = p.y;
        // shadow
        let lift = (GROUND - (y + FEET)).max(0.0);
        let k = (1.0 - lift / 300.0).max(0.3);
        ellipse(x, GROUND + 2.0, 40.0 * k, 8.0 * k, with_a(BLACK, 0.28 * k));
        // power aura
        if p.power_ready > 0.0 {
            let pulse = (self.now as f32 * 12.0).sin() * 4.0;
            circ(x, y + 10.0, HEAD_R + 22.0 + pulse, with_a(hex(0xff8a1a), 0.25));
            circ(x, y + 10.0, HEAD_R + 12.0 + pulse, with_a(hex(0xffc04a), 0.25));
        }
        // back foot
        let walk = (p.walk).sin() * 6.0;
        let back = (x - s * 10.0 - walk * s, y + FEET - FOOT_R + 2.0);
        linev(x - s * 4.0, y + BODY_OFF + 8.0, back.0, back.1, 9.0, skin);
        ellipse(back.0 + s * 4.0, back.1 + 4.0, 17.0, 9.0, mix(boot, BLACK, 0.3));
        // body
        circ(x, y + BODY_OFF, BODY_R, shirt);
        rectv(x - BODY_R * 0.8, y + BODY_OFF + 8.0, BODY_R * 1.6, 10.0, shirt2);
        // kicking leg
        let (fx, fy) = p.foot();
        linev(x + s * 4.0, y + BODY_OFF + 8.0, fx, fy, 10.0, skin);
        ellipse(fx + s * 4.0, fy + 3.0, 19.0, 10.0, boot);
        ellipse(fx + s * 4.0, fy + 1.0, 12.0, 3.0, with_a(if i == 0 { WHITE } else { hex(0xd8432f) }, 0.9));
        // head
        circ(x, y + 2.0, HEAD_R + 2.0, with_a(BLACK, 0.35));
        circ(x, y, HEAD_R, skin);
        // hair: a cap over the top and back of the head
        circ(x - s * 6.0, y - 14.0, HEAD_R * 0.86, hair);
        circ(x + s * 8.0, y + 6.0, HEAD_R * 0.82, skin);
        // ear
        circ(x - s * 10.0, y + 4.0, 9.0, mix(skin, BLACK, 0.1));
        // eye looking at the ball
        let ex = x + s * 20.0;
        let ey = y - 2.0;
        if p.stun > 0.0 {
            linev(ex - 8.0, ey - 8.0, ex + 8.0, ey + 8.0, 4.0, BLACK);
            linev(ex - 8.0, ey + 8.0, ex + 8.0, ey - 8.0, 4.0, BLACK);
            let t = self.now as f32 * 6.0;
            for j in 0..3 {
                let a = t + j as f32 * 2.09;
                star(x + a.cos() * 34.0, y - HEAD_R - 12.0 + a.sin() * 8.0, 7.0, hex(0xffe14a));
            }
        } else {
            ellipse(ex, ey, 12.0, 13.0, WHITE);
            let bx = self.sim.ball.x - ex;
            let by = self.sim.ball.y - ey;
            let l = (bx * bx + by * by).sqrt().max(1.0);
            circ(ex + bx / l * 5.0, ey + by / l * 5.0, 6.0, hex(0x1a1a22));
            linev(ex - 12.0, ey - 18.0, ex + 10.0, ey - 15.0 - s * 2.0, 4.0, mix(hair, BLACK, 0.3));
        }
        // nose and mouth
        circ(x + s * 36.0, y + 10.0, 7.0, mix(skin, hex(0xc06040), 0.25));
        let open = if p.kick_t >= 0.0 { 6.0 } else { 2.5 };
        ellipse(x + s * 22.0, y + 24.0, 9.0, open, hex(0x6a2a24));
    }

    fn draw_ball(&self) {
        let b = self.sim.ball;
        let lift = (GROUND - (b.y + BALL_R)).max(0.0);
        let k = (1.0 - lift / 400.0).max(0.25);
        ellipse(b.x, GROUND + 2.0, 22.0 * k, 5.0 * k, with_a(BLACK, 0.3 * k));
        if b.fire > 0.0 {
            circ(b.x, b.y, BALL_R + 10.0, with_a(hex(0xff6a00), 0.45));
        }
        circ(b.x, b.y, BALL_R + 1.5, with_a(BLACK, 0.5));
        circ(b.x, b.y, BALL_R, if b.fire > 0.0 { hex(0xffe0a0) } else { WHITE });
        draw_ball_spots(b.x, b.y, BALL_R, b.angle);
    }

    fn draw_match(&mut self) {
        let shake = if self.shake > 0.0 { (self.now as f32 * 90.0).sin() * 5.0 * self.shake / 0.25 } else { 0.0 };
        let (s, ox, oy) = view();
        VIEW.with(|v| v.set((s, ox + shake * s, oy)));
        self.draw_field(true);
        for p in &self.particles {
            if p.c.r > 0.9 && p.c.g < 0.6 {
                circ(p.x, p.y, p.r * (1.0 - p.life / p.max), with_a(p.c, 0.7));
            }
        }
        self.draw_player(0);
        self.draw_player(1);
        self.draw_ball();
        self.draw_goal_frames();
        for p in &self.particles {
            if !(p.c.r > 0.9 && p.c.g < 0.6) {
                circ(p.x, p.y, p.r, with_a(p.c, 1.0 - p.life / p.max));
            }
        }
        VIEW.with(|v| v.set((s, ox, oy)));
        self.draw_hud();
        self.draw_touch_buttons();
    }

    fn draw_hud(&mut self) {
        let sim = &self.sim;
        // scoreboard
        rrect(470.0, 10.0, 340.0, 74.0, 16.0, with_a(hex(0x0b1022), 0.85));
        rrect(478.0, 18.0, 84.0, 58.0, 12.0, hex(0x2d6fd0));
        rrect(718.0, 18.0, 84.0, 58.0, 12.0, hex(0xd8432f));
        text_o(&sim.score[0].to_string(), 520.0, 66.0, 54.0, WHITE);
        text_o(&sim.score[1].to_string(), 760.0, 66.0, 54.0, WHITE);
        let tl = sim.time_left.max(0.0).ceil() as i32;
        let tcol = if tl <= 10 && sim.phase == Phase::Play { hex(0xff6a5a) } else { WHITE };
        text_o(&format!("{}:{:02}", tl / 60, tl % 60), 640.0, 62.0, 44.0, tcol);
        self.hud.score = true;
        self.hud.timer = true;
        // names and power bars
        for i in 0..2 {
            let p = sim.p[i];
            let (bx, name_x) = if i == 0 { (30.0, 30.0) } else { (W - 30.0 - 360.0, W - 30.0) };
            let name = self.player_name(i);
            let nw = text_w(name, 26.0);
            let nx = if i == 0 { name_x } else { name_x - nw };
            text(name, nx + 2.0, 34.0, 26.0, with_a(BLACK, 0.7));
            text(name, nx, 32.0, 26.0, WHITE);
            rrect(bx, 44.0, 360.0, 26.0, 13.0, with_a(hex(0x0b1022), 0.85));
            let fill = if p.power_ready > 0.0 { p.power_ready / game::POWER_READY_TIME } else { p.power };
            let fw = (352.0 * fill).max(0.0);
            let full = p.power >= 1.0 || p.power_ready > 0.0;
            let pulse = if full { 0.25 + 0.25 * (self.now as f32 * 8.0).sin() } else { 0.0 };
            let col = mix(hex(0xff8a1a), hex(0xffe14a), pulse + 0.2);
            if fw > 2.0 {
                let fx = if i == 0 { bx + 4.0 } else { bx + 356.0 - fw };
                rrect(fx, 48.0, fw, 18.0, 9.0, col);
            }
            let label = if p.power_ready > 0.0 { "CHUTE AGORA!" } else if p.power >= 1.0 { "PODER PRONTO!" } else { "PODER" };
            text_o(label, bx + 180.0, 64.0, 18.0, WHITE);
        }
        self.hud.power = true;

        // big centre messages
        match sim.phase {
            Phase::Countdown => {
                let n = sim.phase_t.ceil() as i32;
                let k = 1.0 + (sim.phase_t.fract()) * 0.4;
                text_o(&n.max(1).to_string(), 640.0, 330.0, 120.0 * k, hex(0xffd34d));
            }
            Phase::Ready => text_o("PREPARAR...", 640.0, 300.0, 60.0, WHITE),
            Phase::Play => {
                if sim.time_left > sim.match_len - 0.8 {
                    text_o("JA!", 640.0, 320.0, 110.0, hex(0xffd34d));
                }
            }
            Phase::Goal => {
                let k = (self.goal_anim * 4.0).min(1.0);
                let size = 60.0 + 80.0 * k + (self.goal_anim * 10.0).sin() * 6.0;
                text_o("GOL!", 640.0, 320.0, size, hex(0xffd34d));
                text_o(self.player_name(sim.last_scorer as usize), 640.0, 380.0, 34.0, WHITE);
            }
            Phase::Over => {
                if self.over_t <= 1.2 {
                    text_o("FIM DE JOGO", 640.0, 320.0, 80.0, WHITE);
                }
            }
        }
    }

    fn draw_touch_buttons(&mut self) {
        for b in touch_layout(self.mode) {
            let inp = self.held[b.who];
            let down = match b.kind {
                Btn::Left => inp.left,
                Btn::Right => inp.right,
                Btn::Jump => inp.jump,
                Btn::Kick => inp.kick,
                Btn::Lob => inp.lob,
                Btn::Power => inp.power,
            };
            let p = self.sim.p[b.who];
            let ready = p.power >= 1.0 || p.power_ready > 0.0;
            let base = match b.kind {
                Btn::Kick | Btn::Lob => hex(0xd8432f),
                Btn::Jump => hex(0x2d6fd0),
                Btn::Power => {
                    if ready {
                        mix(hex(0xff8a1a), hex(0xffe14a), 0.5 + 0.5 * (self.now as f32 * 8.0).sin())
                    } else {
                        hex(0x555a66)
                    }
                }
                _ => hex(0x2a2f45),
            };
            let a = if down { 0.85 } else { 0.5 };
            circ(b.x, b.y, b.r, with_a(base, a));
            circ_line(b.x, b.y, b.r, 3.0, with_a(WHITE, 0.6));
            let r = b.r;
            let c = with_a(WHITE, 0.9);
            match b.kind {
                Btn::Left => tri((b.x - r * 0.45, b.y), (b.x + r * 0.3, b.y - r * 0.45), (b.x + r * 0.3, b.y + r * 0.45), c),
                Btn::Right => tri((b.x + r * 0.45, b.y), (b.x - r * 0.3, b.y - r * 0.45), (b.x - r * 0.3, b.y + r * 0.45), c),
                Btn::Jump => {
                    tri((b.x, b.y - r * 0.5), (b.x - r * 0.42, b.y + r * 0.05), (b.x + r * 0.42, b.y + r * 0.05), c);
                    text_o("PULO", b.x, b.y + r * 0.55, r * 0.36, WHITE);
                }
                Btn::Kick => {
                    ellipse(b.x, b.y - r * 0.12, r * 0.42, r * 0.22, c);
                    text_o("CHUTE", b.x, b.y + r * 0.5, r * 0.32, WHITE);
                }
                Btn::Lob => {
                    let s = if b.who == 0 { 1.0 } else { -1.0 };
                    linev(b.x - s * r * 0.35, b.y + r * 0.1, b.x + s * r * 0.35, b.y - r * 0.4, 4.0, c);
                    text_o("ALTO", b.x, b.y + r * 0.6, r * 0.36, WHITE);
                }
                Btn::Power => {
                    star(b.x, b.y - r * 0.12, r * 0.4, c);
                    text_o("PODER", b.x, b.y + r * 0.62, r * 0.32, WHITE);
                }
            }
        }
        self.hud.buttons = true;
    }
}

fn star(x: f32, y: f32, r: f32, c: Color) {
    for k in 0..5 {
        let a = -1.5708 + k as f32 * 1.2566;
        let b1 = a + 0.6283;
        let b0 = a - 0.6283;
        tri((x + a.cos() * r, y + a.sin() * r), (x + b0.cos() * r * 0.42, y + b0.sin() * r * 0.42), (x + b1.cos() * r * 0.42, y + b1.sin() * r * 0.42), c);
    }
    circ(x, y, r * 0.45, c);
}

fn draw_ball_spots(x: f32, y: f32, r: f32, angle: f32) {
    let dark = hex(0x1d1f26);
    circ(x, y, r * 0.3, dark);
    for k in 0..5 {
        let a = angle + k as f32 * 1.2566;
        circ(x + a.cos() * r * 0.72, y + a.sin() * r * 0.72, r * 0.2, dark);
        linev(x + a.cos() * r * 0.3, y + a.sin() * r * 0.3, x + a.cos() * r * 0.6, y + a.sin() * r * 0.6, r * 0.07, with_a(dark, 0.6));
    }
}

// ---------------------------------------------------------------- start-up

fn conf() -> Conf {
    Conf {
        window_title: "Cabecada".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: cfg!(target_os = "android"),
        window_resizable: true,
        sample_count: 4,
        ..Default::default()
    }
}

enum Run {
    Normal,
    Screenshot(String),
    SelfTest(String),
}

fn parse_args() -> Run {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            return Run::Screenshot(args[i + 1].clone());
        }
        if args[i] == "--self-test" && i + 1 < args.len() {
            return Run::SelfTest(args[i + 1].clone());
        }
        i += 1;
    }
    Run::Normal
}

async fn amain(run: Run, prev_fg: isize) {
    let quiet = !matches!(run, Run::Normal);
    let sounds = if quiet { None } else { audio::Sounds::load().await };
    let mut app = App::new(sounds);
    match run {
        Run::Normal => loop {
            let dt = get_frame_time();
            app.frame(dt).await;
            next_frame().await;
        },
        Run::Screenshot(path) => {
            app.ai_both = true;
            app.level = 2;
            app.start_match(Mode::Cpu);
            app.sim.phase = Phase::Play;
            app.sim.time_left = 74.0;
            app.sim.score = [1, 1];
            app.sim.p[0].power = 0.65;
            app.sim.p[1].power = 1.0;
            // let the match run a moment so the picture shows play
            for _ in 0..120 {
                app.update_match(1.0 / 60.0, [Input::default(); 2]);
            }
            for f in 0..120 {
                if f == 1 {
                    win::lower_window(prev_fg);
                }
                set_view();
                clear_background(hex(0x0b1022));
                app.now += 1.0 / 60.0;
                app.update_match(1.0 / 60.0, [Input::default(); 2]);
                app.draw_match();
                if f == 119 {
                    let img = get_screen_data();
                    img.export_png(&path);
                }
                next_frame().await;
            }
            std::process::exit(0);
        }
        Run::SelfTest(path) => {
            let mut frame = 0u32;
            let mut t = selftest::SelfTest::new();
            loop {
                if frame == 1 {
                    win::lower_window(prev_fg);
                }
                frame += 1;
                set_view();
                clear_background(hex(0x0b1022));
                app.now += 1.0 / 60.0;
                let done = t.tick(&mut app);
                app.draw_match();
                if done {
                    t.finish(&mut app, &path);
                    std::process::exit(0);
                }
                next_frame().await;
            }
        }
    }
}

fn main() {
    let run = parse_args();
    let prev = win::foreground();
    macroquad::Window::from_config(conf(), amain(run, prev));
}
