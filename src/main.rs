//! Cabecada: 1 v 1 big-head football that plays offline.
//! Modes: against the computer, two players on one device, and local-network
//! play where one player's device hosts the match (no internet server).

mod ai;
mod assets;
mod audio;
mod game;
mod net;
mod platform;
mod selftest;
mod win;

use game::*;
use macroquad::prelude::*;
use std::cell::{Cell, RefCell};
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
/// A quarter of a disc (a fan of triangles) starting at angle `a0`.
fn quarter(cx: f32, cy: f32, r: f32, a0: f32, c: Color) {
    let n = 8;
    let step = std::f32::consts::FRAC_PI_2 / n as f32;
    for i in 0..n {
        let a = a0 + i as f32 * step;
        let b = a + step;
        tri((cx, cy), (cx + r * a.cos(), cy + r * a.sin()), (cx + r * b.cos(), cy + r * b.sin()), c);
    }
}
/// Rounded rectangle. The pieces never overlap, so see-through colours stay even.
pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    use std::f32::consts::{FRAC_PI_2, PI};
    let r = r.min(w * 0.5).min(h * 0.5);
    rectv(x + r, y, w - 2.0 * r, h, c);
    rectv(x, y + r, r, h - 2.0 * r, c);
    rectv(x + w - r, y + r, r, h - 2.0 * r, c);
    quarter(x + r, y + r, r, PI, c);
    quarter(x + w - r, y + r, r, PI + FRAC_PI_2, c);
    quarter(x + w - r, y + h - r, r, 0.0, c);
    quarter(x + r, y + h - r, r, FRAC_PI_2, c);
}
pub fn tex_draw(tex: &Texture2D, x: f32, y: f32, w: f32, h: f32, col: Color, flip_x: bool, rot: f32) {
    draw_texture_ex(
        tex,
        px(x),
        py(y),
        col,
        DrawTextureParams {
            dest_size: Some(vec2(sc(w), sc(h))),
            flip_x,
            rotation: rot,
            ..Default::default()
        },
    );
}
pub fn tex_center(tex: &Texture2D, cx: f32, cy: f32, w: f32, h: f32, col: Color, flip_x: bool, rot: f32) {
    tex_draw(tex, cx - w * 0.5, cy - h * 0.5, w, h, col, flip_x, rot);
}
pub fn tex_sub(tex: &Texture2D, sx: f32, sy: f32, sw: f32, sh: f32, x: f32, y: f32, w: f32, h: f32, col: Color, flip_x: bool, rot: f32) {
    draw_texture_ex(
        tex,
        px(x),
        py(y),
        col,
        DrawTextureParams {
            dest_size: Some(vec2(sc(w), sc(h))),
            source: Some(Rect::new(sx, sy, sw, sh)),
            flip_x,
            rotation: rot,
            ..Default::default()
        },
    );
}
/// Phones are wider than 16:9 (and tablets taller), so the 1280x720 picture leaves bars
/// at the sides (or top and bottom). Fill them with the edges of the current background,
/// stretched and a little darker, so the scene seems to go on.
pub fn draw_side_fill(tex: &Texture2D) {
    let (s, ox, oy) = view();
    let (tw, th) = (tex.width(), tex.height());
    let strip = 24.0;
    let tint = Color::new(0.7, 0.7, 0.76, 1.0);
    let draw = |src: Rect, x: f32, y: f32, w: f32, h: f32| {
        draw_texture_ex(tex, x, y, tint, DrawTextureParams { dest_size: Some(vec2(w, h)), source: Some(src), ..Default::default() });
    };
    if ox > 0.5 {
        draw(Rect::new(0.0, 0.0, strip, th), 0.0, oy, ox + 1.0, H * s);
        draw(Rect::new(tw - strip, 0.0, strip, th), ox + W * s - 1.0, oy, ox + 1.0, H * s);
    }
    if oy > 0.5 {
        draw(Rect::new(0.0, 0.0, tw, strip), ox, 0.0, W * s, oy + 1.0);
        draw(Rect::new(0.0, th - strip, tw, strip), ox, oy + H * s - 1.0, W * s, oy + 1.0);
    }
}

pub fn draw_9slice(tex: &Texture2D, x: f32, y: f32, w: f32, h: f32, col: Color) {
    let tw = tex.width();
    let th = tex.height();
    let sc_c = 32.0_f32.min(tw * 0.4).min(th * 0.4);
    let dc = (h * 0.34).min(w * 0.34).min(sc_c);
    let sx = [0.0, sc_c, tw - sc_c, tw];
    let sy = [0.0, sc_c, th - sc_c, th];
    let dx = [x, x + dc, x + w - dc, x + w];
    let dy = [y, y + dc, y + h - dc, y + h];
    for row in 0..3 {
        let dh = dy[row + 1] - dy[row];
        let sh = sy[row + 1] - sy[row];
        if dh <= 0.1 || sh <= 0.1 {
            continue;
        }
        for c in 0..3 {
            let dw = dx[c + 1] - dx[c];
            let sw = sx[c + 1] - sx[c];
            if dw <= 0.1 || sw <= 0.1 {
                continue;
            }
            tex_sub(tex, sx[c], sy[row], sw, sh, dx[c], dy[row], dw, dh, col, false, 0.0);
        }
    }
}
thread_local! {
    static FONT: RefCell<Option<Font>> = const { RefCell::new(None) };
}

/// Loads a bold font for all text: `assets/fonts/game.ttf` if it exists (next to the program,
/// two folders up from `target/release`, or in the current folder), otherwise a bold font that
/// ships with Windows. With none of them the built-in pixel font is used.
fn load_game_font() {
    let mut paths: Vec<std::path::PathBuf> = vec![];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("assets/fonts/game.ttf"));
            paths.push(dir.join("../../assets/fonts/game.ttf"));
        }
    }
    paths.push("assets/fonts/game.ttf".into());
    for f in ["ARLRDBD.TTF", "seguibl.ttf", "ariblk.ttf", "arialbd.ttf"] {
        paths.push(std::path::Path::new("C:\\Windows\\Fonts").join(f));
    }
    for p in paths {
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(font) = load_ttf_font_from_bytes(&bytes) {
                FONT.with(|f| *f.borrow_mut() = Some(font));
                return;
            }
        }
    }
    if let Ok(font) = load_ttf_font_from_bytes(assets::FONT_BYTES) {
        FONT.with(|f| *f.borrow_mut() = Some(font));
    }
}

fn has_font() -> bool {
    FONT.with(|f| f.borrow().is_some())
}

/// The built-in pixel font has no accents: without a real font they are dropped.
fn fix_text(t: &str) -> std::borrow::Cow<'_, str> {
    if t.is_ascii() || has_font() {
        return t.into();
    }
    t.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'Á' | 'À' | 'Â' | 'Ã' => 'A',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'É' | 'Ê' => 'E',
            'í' | 'ì' | 'î' => 'i',
            'Í' => 'I',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'Ó' | 'Ô' | 'Õ' => 'O',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'Ú' => 'U',
            'ç' => 'c',
            'Ç' => 'C',
            _ => c,
        })
        .collect::<String>()
        .into()
}

pub fn text_w(t: &str, size: f32) -> f32 {
    let t = fix_text(t);
    let fs = sc(size).max(1.0) as u16;
    FONT.with(|f| measure_text(&t, f.borrow().as_ref(), fs, 1.0).width) / view().0
}
pub fn text(t: &str, x: f32, y: f32, size: f32, c: Color) {
    let t = fix_text(t);
    let fs = sc(size).max(1.0) as u16;
    FONT.with(|f| {
        draw_text_ex(&t, px(x), py(y), TextParams { font: f.borrow().as_ref(), font_size: fs, color: c, ..Default::default() });
    });
}
pub fn text_c(t: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = text_w(t, size);
    text(t, cx - w * 0.5, y, size, c);
}
/// Centered text with a dark outline so it reads on any background.
pub fn text_o(t: &str, cx: f32, y: f32, size: f32, c: Color) {
    let w = text_w(t, size);
    let o = (size / 14.0).max(1.5);
    let k = o * 0.7;
    let dark = Color::new(0.04, 0.05, 0.1, 0.95);
    for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (-k, -k), (k, -k), (-k, k), (k, k)] {
        text(t, cx - w * 0.5 + dx, y + dy, size, dark);
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

#[derive(Clone, Copy)]
enum FxKind {
    Dust,
    Kick,
    Boom,
}

struct AnimFx {
    kind: FxKind,
    x: f32,
    y: f32,
    size: f32,
    flip_x: bool,
    life: f32,
    max: f32,
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
    assets: assets::Assets,
    pub my_power: u8, // PK_* or POWER_KINDS for "random"
    pub touch_ui: bool, // on-screen buttons (always on phones, after the first touch elsewhere, or with T)
    prev_ground: [bool; 2],
    prev_kick: [bool; 2],
    tour: Option<Tour>,
    crowd_on: bool,
    fx: Vec<AnimFx>,
    char_idx: [usize; 2],
    stadium_idx: usize,
    quit_ask: bool,     // the "leave the game?" box is open
    events_sub: usize,  // macroquad input subscriber, to hear when the app goes to the background
}

/// A knock-out cup against the computer: three rounds, each harder than the last.
#[derive(Clone, Copy)]
struct Tour {
    round: usize,
}
const TOUR_ROUNDS: [(&str, usize); 3] = [("Quartas de final", 0), ("Semifinal", 1), ("Final", 2)];

const LEVEL_NAMES: [&str; 3] = ["Fácil", "Normal", "Difícil"];

impl App {
    fn new(sounds: Option<audio::Sounds>, assets: assets::Assets) -> App {
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
            assets,
            my_power: POWER_KINDS,
            touch_ui: cfg!(target_os = "android"),
            prev_ground: [true; 2],
            prev_kick: [false; 2],
            tour: None,
            crowd_on: false,
            fx: vec![],
            char_idx: [0, 1],
            stadium_idx: 0,
            quit_ask: false,
            events_sub: macroquad::input::utils::register_input_subscriber(),
        }
    }

    /// The cup was just won: the champion screen is showing.
    fn champion_showing(&self) -> bool {
        self.screen == Screen::Match
            && self.mode == Mode::Cpu
            && self.sim.phase == Phase::Over
            && self.over_t > 1.2
            && matches!(self.tour, Some(t) if t.round + 1 >= TOUR_ROUNDS.len())
            && self.sim.winner() == Some(0)
    }

    /// The picture behind the current screen (also used to fill the sides of wide phones).
    fn backdrop_tex(&self) -> &Texture2D {
        if self.screen == Screen::Match && !self.champion_showing() {
            &self.assets.stadium_bgs[self.stadium_idx % self.assets.stadium_bgs.len()]
        } else {
            &self.assets.menu_bg
        }
    }

    /// Computer level in use: the cup sets its own, otherwise the menu choice.
    fn cpu_level(&self) -> usize {
        match self.tour {
            Some(t) => TOUR_ROUNDS[t.round.min(2)].1,
            None => self.level,
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
        self.fx.clear();
        self.prev_ground = [true; 2];
        self.prev_kick = [false; 2];
        if mode != Mode::Cpu {
            self.tour = None; // the cup is only against the computer
        }
        if let Some(t) = self.tour {
            let r = t.round.min(2);
            self.stadium_idx = r;
            let boss = [1usize, 3, 5][r];
            self.char_idx[1] = if boss == self.char_idx[0] { (boss + 1) % 6 } else { boss };
        } else if self.char_idx[1] == self.char_idx[0] {
            self.char_idx[1] = (self.char_idx[0] + 1) % 6;
        }
        // each player brings a power (random when "Aleatório" is chosen)
        let mine = self.pick_power();
        let other = rand::gen_range(0, POWER_KINDS as u32) as u8;
        match mode {
            Mode::Host => {
                self.sim.p[0].pkind = mine;
                self.sim.p[1].pkind = self.host.as_ref().map(|h| h.remote_pkind).unwrap_or(other);
            }
            Mode::Client => {}
            _ => {
                self.sim.p[0].pkind = mine;
                self.sim.p[1].pkind = other;
            }
        }
    }

    fn pick_power(&self) -> u8 {
        if self.my_power >= POWER_KINDS {
            rand::gen_range(0, POWER_KINDS as u32) as u8
        } else {
            self.my_power
        }
    }

    fn power_label(&self) -> String {
        if self.my_power >= POWER_KINDS {
            "Poder: Aleatório".into()
        } else {
            format!("Poder: {}", POWER_NAMES[self.my_power as usize])
        }
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
        if !touches().is_empty() {
            self.touch_ui = true; // a finger on the screen: show the on-screen buttons
        }
        if is_key_pressed(KeyCode::T) {
            self.touch_ui = !self.touch_ui;
        }
        if self.touch_ui {
            let pts = pointers();
            for b in touch_layout(self.mode) {
                if pts.iter().any(|&(x, y)| (x - b.x).powi(2) + (y - b.y).powi(2) < (b.r * 1.2).powi(2)) {
                    set_btn(&mut inp[b.who], b.kind);
                }
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
        if ev & EV_BOOM != 0 {
            self.shake = 0.35;
            self.fx.push(AnimFx {
                kind: FxKind::Boom,
                x: self.sim.ball.x,
                y: self.sim.ball.y,
                size: 260.0,
                flip_x: false,
                life: 0.0,
                max: 0.48,
            });
            for c in [hex(0xff5a1a), hex(0xffd34d), hex(0x555555)] {
                self.burst(self.sim.ball.x, self.sim.ball.y, 14, c);
            }
        }
        if ev & EV_FREEZE != 0 {
            for c in [hex(0x9fe6ff), WHITE] {
                self.burst(self.sim.ball.x, self.sim.ball.y, 12, c);
            }
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
            (EV_BOOM, &s.goal, 1.0),
            (EV_BOOM, &s.bounce, 1.0),
            (EV_FREEZE, &s.post, 0.6),
        ];
        for (bit, snd, vol) in list {
            if ev & bit != 0 {
                s.play(snd, vol);
            }
        }
    }

    /// A puff of dust on the grass (landing, take-off).
    fn dust(&mut self, x: f32, y: f32, n: usize) {
        self.fx.push(AnimFx {
            kind: FxKind::Dust,
            x,
            y: y - 22.0,
            size: 86.0,
            flip_x: rand::gen_range(0, 2) == 0,
            life: 0.0,
            max: 0.36,
        });
        for k in 0..n {
            let dir = if k % 2 == 0 { -1.0 } else { 1.0 };
            self.particles.push(Particle {
                x: x + rand::gen_range(-18.0, 18.0),
                y: y - 2.0,
                vx: dir * rand::gen_range(40.0, 170.0),
                vy: rand::gen_range(-90.0, -20.0),
                life: 0.0,
                max: rand::gen_range(0.25, 0.5),
                r: rand::gen_range(4.0, 8.0),
                c: Color::new(0.88, 0.84, 0.7, 1.0),
            });
        }
    }

    /// Jump / landing dust and kick swooshes, worked out from how the players moved
    /// (so they also show on the joining device).
    fn detect_fx(&mut self) {
        for i in 0..2 {
            let p = self.sim.p[i];
            let near_grass = p.y + FEET > GROUND - 30.0;
            if p.on_ground && !self.prev_ground[i] && near_grass {
                self.dust(p.x, GROUND, 7);
                self.play_one(|s| (&s.land, 0.35));
            } else if !p.on_ground && self.prev_ground[i] && p.vy < -200.0 && near_grass {
                self.dust(p.x, GROUND, 5);
                self.play_one(|s| (&s.jump, 0.3));
            }
            self.prev_ground[i] = p.on_ground;
            let kicking = p.kick_t >= 0.0;
            if kicking && !self.prev_kick[i] {
                let (fx, fy) = p.foot();
                self.fx.push(AnimFx {
                    kind: FxKind::Kick,
                    x: fx + p.side * 22.0,
                    y: fy - 6.0,
                    size: 92.0,
                    flip_x: p.side < 0.0,
                    life: 0.0,
                    max: 0.22,
                });
                self.burst(fx + p.side * 20.0, fy, 4, with_a(WHITE, 1.0));
            }
            self.prev_kick[i] = kicking;
        }
    }

    fn play_one(&self, pick: impl Fn(&audio::Sounds) -> (&macroquad::audio::Sound, f32)) {
        if !self.sound_on {
            return;
        }
        if let Some(s) = self.sounds.as_ref() {
            let (snd, vol) = pick(s);
            s.play(snd, vol);
        }
    }

    /// Crowd murmur while a match is on.
    fn crowd(&mut self, want: bool) {
        let want = want && self.sound_on;
        if let Some(s) = self.sounds.as_ref() {
            if want && !self.crowd_on {
                s.play_loop(&s.crowd, 0.12);
                self.crowd_on = true;
            } else if !want && self.crowd_on {
                s.stop(&s.crowd);
                self.crowd_on = false;
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
                    let lvl = self.cpu_level();
                    while self.acc >= STEP {
                        self.acc -= STEP;
                        let mut i = inputs;
                        if self.mode == Mode::Cpu {
                            i[1] = ai::think(&self.sim, 1, lvl, &mut self.ai[1], STEP);
                            if self.ai_both {
                                i[0] = ai::think(&self.sim, 0, lvl, &mut self.ai[0], STEP);
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
        self.detect_fx();
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
        for f in self.fx.iter_mut() {
            f.life += dt;
        }
        self.fx.retain(|f| f.life < f.max);
        if self.sim.ball.fire > 0.0 && rand::gen_range(0.0, 1.0) < 0.8 {
            let b = self.sim.ball;
            self.particles.push(Particle { x: b.x, y: b.y, vx: -b.vx * 0.1 + rand::gen_range(-40.0, 40.0), vy: -b.vy * 0.1 - 60.0, life: 0.0, max: 0.35, r: rand::gen_range(6.0, 12.0), c: hex(0xff8a1a) });
        }
        let b = self.sim.ball;
        if b.ice > 0.0 && rand::gen_range(0.0, 1.0) < 0.7 {
            self.particles.push(Particle { x: b.x, y: b.y, vx: rand::gen_range(-60.0, 60.0), vy: rand::gen_range(-80.0, 0.0), life: 0.0, max: 0.4, r: rand::gen_range(2.5, 5.0), c: hex(0xbff0ff) });
        }
        if b.bomb > 0.0 && rand::gen_range(0.0, 1.0) < 0.5 {
            self.particles.push(Particle { x: b.x, y: b.y - b.r(), vx: rand::gen_range(-30.0, 30.0), vy: -120.0, life: 0.0, max: 0.3, r: 3.0, c: hex(0xffe14a) });
        }
    }

    // ------------------------------------------------------------ screenshot scenes (--shots)

    /// Puts the app into a known state for `--shots` (menus, a goal, each power...).
    fn scene_setup(&mut self, name: &str) {
        self.paused = false;
        self.over_t = 0.0;
        self.ai_both = false;
        self.particles.clear();
        self.shake = 0.0;
        self.touch_ui = name == "play_touch";
        self.tour = match name {
            "countdown_tour" => Some(Tour { round: 1 }),
            "champion" => Some(Tour { round: 2 }),
            "eliminated" => Some(Tour { round: 1 }),
            _ => None,
        };
        match name {
            "menu" => self.screen = Screen::Menu,
            "host_lobby" => {
                self.my_ip = "192.168.0.12".into();
                self.screen = Screen::HostLobby;
            }
            "join_lobby" => {
                self.ip_text = "192.168.0.12".into();
                self.screen = Screen::JoinLobby;
            }
            _ => {
                let mode = if name == "play_local2" { Mode::Local2 } else { Mode::Cpu };
                self.my_power = PK_FIRE;
                self.level = 2;
                self.start_match(mode);
                self.ai_both = name == "play_cpu" || name == "play_touch";
                self.sim.phase = Phase::Play;
                self.sim.time_left = 74.0;
                self.sim.score = [1, 1];
                self.sim.p[0].power = 0.65;
                self.sim.p[1].power = 1.0;
                let b = &mut self.sim.ball;
                match name {
                    "goal" => {
                        self.sim.score = [2, 1];
                        self.sim.phase = Phase::Goal;
                        self.sim.phase_t = GOAL_PAUSE;
                        self.sim.last_scorer = 0;
                        self.goal_anim = 0.3;
                    }
                    "over" => {
                        self.sim.score = [3, 2];
                        self.sim.phase = Phase::Over;
                        self.sim.time_left = 0.0;
                        self.over_t = 2.0;
                    }
                    "countdown" | "countdown_tour" => {
                        self.sim.phase = Phase::Countdown;
                        self.sim.phase_t = 2.4;
                        self.sim.score = [0, 0];
                        self.sim.time_left = self.sim.match_len;
                    }
                    "overtime" => {
                        self.sim.phase = Phase::Ready;
                        self.sim.phase_t = 5.0;
                        self.sim.overtime = true;
                        self.sim.time_left = 0.0;
                        self.sim.score = [2, 2];
                    }
                    "eliminated" => {
                        self.sim.score = [1, 2];
                        self.sim.phase = Phase::Over;
                        self.sim.time_left = 0.0;
                        self.over_t = 2.0;
                    }
                    "champion" => {
                        self.sim.score = [2, 0];
                        self.sim.phase = Phase::Over;
                        self.sim.time_left = 0.0;
                        self.over_t = 2.0;
                    }
                    "pause" => self.paused = true,
                    "stun" => {
                        self.sim.p[1].stun = 0.9;
                        self.sim.p[0].power_ready = 4.0;
                        self.sim.p[0].power = 1.0;
                    }
                    "pw_fire" => {
                        *b = Ball { x: 560.0, y: 430.0, vx: 1100.0, vy: -150.0, fire: 1.1, fire_owner: 0, ..Default::default() };
                    }
                    "pw_big" => {
                        *b = Ball { x: 640.0, y: 420.0, big: 7.0, ..Default::default() };
                    }
                    "pw_ice" => {
                        *b = Ball { x: 760.0, y: 440.0, vx: 500.0, ice: 1.4, ice_owner: 0, ..Default::default() };
                        self.sim.p[1].frozen = 2.6;
                        self.sim.p[1].stun = 2.6;
                    }
                    "pw_invis" => {
                        *b = Ball { x: 640.0, y: 420.0, invis: 4.0, invis_owner: 0, ..Default::default() };
                    }
                    "pw_bomb" => {
                        *b = Ball { x: 640.0, y: 420.0, bomb: 1.6, bomb_owner: 0, ..Default::default() };
                    }
                    "pw_clone" => {
                        *b = Ball { x: 560.0, y: 430.0, vx: 700.0, vy: -300.0, clone_t: 4.0, ..Default::default() };
                        b.clones = [[600.0, 330.0, 700.0, -500.0], [600.0, 480.0, 800.0, 100.0]];
                    }
                    _ => {}
                }
            }
        }
    }

    /// One frame of a `--shots` scene: no real input, fixed time step.
    fn scene_frame(&mut self, dt: f32) {
        self.now += dt as f64;
        match self.screen {
            Screen::Menu => self.menu(),
            Screen::HostLobby => self.host_lobby(),
            Screen::JoinLobby => self.join_lobby(),
            Screen::Match => {
                self.update_match(dt, [Input::default(); 2]);
                self.draw_match();
                self.match_overlays();
            }
        }
    }

    // ------------------------------------------------------------ frame

    async fn frame(&mut self, dt: f32) {
        self.now += dt as f64;
        // the app went to the background (home button, a call...): pause a running match
        let mut life = platform::LifeEvents::default();
        macroquad::input::utils::repeat_all_miniquad_input(&mut life, self.events_sub);
        if life.went_away && self.screen == Screen::Match && matches!(self.mode, Mode::Cpu | Mode::Local2) && self.sim.phase != Phase::Over {
            self.paused = true;
        }
        set_view();
        clear_background(hex(0x0b1022));
        draw_side_fill(self.backdrop_tex());
        self.crowd(self.screen == Screen::Match);
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

    fn menu_btn_tex(&self, col: Color) -> &Texture2D {
        if col.g > col.r * 1.2 && col.g > col.b * 1.2 {
            &self.assets.btn_menu_green
        } else if col.r > 0.45 && col.b > 0.55 && col.g < 0.40 {
            &self.assets.btn_menu_purple
        } else if col.b > col.r * 1.25 && col.b > col.g * 1.1 {
            // bright blue is the blue button, dark bluish is the grey one
            if col.b < 0.5 { &self.assets.btn_menu_gray } else { &self.assets.btn_menu_blue }
        } else if col.r > 0.65 && col.g > 0.50 && col.b < 0.25 {
            &self.assets.btn_menu_gold
        } else if col.r > 0.45 && col.g > 0.30 && col.b < 0.25 {
            &self.assets.btn_menu_orange
        } else if col.r > 0.45 && col.b > 0.55 {
            &self.assets.btn_menu_purple
        } else if col.r > col.b * 1.15 && col.g < 0.35 {
            &self.assets.btn_menu_red
        } else {
            &self.assets.btn_menu_gray
        }
    }

    fn button(&self, label: &str, x: f32, y: f32, w: f32, h: f32, col: Color) -> bool {
        let (mx, my) = to_virtual(mouse_position().0, mouse_position().1);
        let hover = mx >= x && mx <= x + w && my >= y && my <= y + h;
        let tex = self.menu_btn_tex(col);
        let tint = if hover { Color::new(1.0, 1.0, 1.0, 1.0) } else { Color::new(0.93, 0.93, 0.95, 1.0) };
        rrect(x + 3.0, y + 6.0, w - 6.0, h - 2.0, 16.0, with_a(BLACK, 0.38));
        draw_9slice(tex, x, y, w, h, tint);
        text_o(label, x + w * 0.5, y + h * 0.5 + h * 0.16, h * 0.41, WHITE);
        if let Some((cx, cy)) = self.clicked() {
            return cx >= x && cx <= x + w && cy >= y && cy <= y + h;
        }
        false
    }

    fn draw_backdrop(&self) {
        tex_draw(&self.assets.menu_bg, 0.0, 0.0, W, H, WHITE, false, 0.0);
        rectv(0.0, 0.0, W, H, with_a(hex(0x0a0f24), 0.42));
    }

    /// Main menu. The phone's back button (or Esc) opens a "leave the game?" box.
    fn menu(&mut self) {
        let back = is_key_pressed(KeyCode::Back) || is_key_pressed(KeyCode::Escape);
        if !self.quit_ask {
            self.menu_main();
            if back && self.screen == Screen::Menu {
                self.quit_ask = true;
            }
            return;
        }
        self.draw_backdrop();
        tex_center(&self.assets.logo, 640.0, 150.0, 400.0, 200.0, WHITE, false, 0.0);
        rrect(390.0, 272.0, 500.0, 220.0, 24.0, with_a(BLACK, 0.6));
        text_o("Sair do jogo?", 640.0, 338.0, 46.0, WHITE);
        if self.button("Sair", 420.0, 392.0, 210.0, 72.0, hex(0x5a2d3a)) {
            platform::quit();
        }
        if self.button("Continuar", 650.0, 392.0, 210.0, 72.0, hex(0x2f8f4e)) || back {
            self.quit_ask = false;
        }
    }

    fn menu_main(&mut self) {
        tex_draw(&self.assets.menu_bg, 0.0, 0.0, W, H, WHITE, false, 0.0);
        rectv(0.0, 0.0, W, H, with_a(hex(0x080d1e), 0.22));
        let version = concat!("v", env!("CARGO_PKG_VERSION"));
        text(version, 1272.0 - text_w(version, 16.0), 712.0, 16.0, with_a(WHITE, 0.55));
        // title logo and showcase characters
        let t = self.now as f32;
        let bob1 = (t * 2.8).sin() * 6.0;
        let bob2 = (t * 2.8 + 1.6).sin() * 6.0;
        // Player 1 showcase (left)
        let c0 = self.char_idx[0] % 6;
        let c1 = self.char_idx[1] % 6;
        ellipse(190.0, 512.0, 58.0, 14.0, with_a(BLACK, 0.35));
        tex_center(&self.assets.boots[c0], 198.0, 488.0 + bob1 * 0.3, 96.0, 64.0, WHITE, false, -0.05);
        tex_center(&self.assets.heads[c0], 190.0, 404.0 + bob1, 168.0, 168.0, WHITE, false, (t * 1.4).sin() * 0.04);
        if self.button(&format!("Jogador: #{}", c0 + 1), 85.0, 536.0, 210.0, 44.0, hex(0x2d6fd0)) {
            self.char_idx[0] = (self.char_idx[0] + 1) % 6;
            if self.char_idx[1] == self.char_idx[0] {
                self.char_idx[1] = (self.char_idx[0] + 1) % 6;
            }
        }
        // Player 2 showcase (right)
        ellipse(1090.0, 512.0, 58.0, 14.0, with_a(BLACK, 0.35));
        tex_center(&self.assets.boots[c1], 1082.0, 488.0 + bob2 * 0.3, 96.0, 64.0, WHITE, true, 0.05);
        tex_center(&self.assets.heads[c1], 1090.0, 404.0 + bob2, 168.0, 168.0, WHITE, true, -(t * 1.4).sin() * 0.04);
        let stad_names = ["Noite", "Pôr do Sol", "Final"];
        if self.button(&format!("Estádio: {}", stad_names[self.stadium_idx % 3]), 985.0, 536.0, 210.0, 44.0, hex(0x34405e)) {
            self.stadium_idx = (self.stadium_idx + 1) % 3;
        }

        tex_center(&self.assets.logo, 640.0, 130.0 + (t * 2.2).sin() * 4.0, 470.0, 235.0, WHITE, false, 0.0);
        rrect(290.0, 236.0, 700.0, 34.0, 17.0, with_a(BLACK, 0.5));
        text_o("futebol de cabeça 1 contra 1  -  sem internet, sem servidor", 640.0, 261.0, 22.0, hex(0xe4f0ff));

        let bx = 390.0;
        let bw = 500.0;
        let bh = 58.0;
        if self.button("Contra o computador", bx, 284.0, bw, bh, hex(0x2f8f4e)) {
            self.tour = None;
            self.start_match(Mode::Cpu);
            return;
        }
        if self.button("Campeonato (3 fases)", bx, 350.0, bw, bh, hex(0xc9951a)) {
            self.tour = Some(Tour { round: 0 });
            self.start_match(Mode::Cpu);
            return;
        }
        if self.button("2 jogadores neste aparelho", bx, 416.0, bw, bh, hex(0x2d6fd0)) {
            self.start_match(Mode::Local2);
            return;
        }
        if self.button("Criar partida (rede local)", bx, 482.0, bw, bh, hex(0xd06a2d)) {
            let ip = net::local_ip().map(|i| i.to_string()).unwrap_or_else(|| "sem rede".into());
            let name = format!("Partida {}", ip);
            let h = net::Host::new(net::PORT, &name).or_else(|_| net::Host::new(0, &name));
            match h {
                Ok(h) => {
                    self.my_ip = if h.port == net::PORT { ip } else { format!("{}:{}", ip, h.port) };
                    self.host = Some(h);
                    self.screen = Screen::HostLobby;
                }
                Err(e) => self.status = format!("Não foi possível abrir a partida: {}", e),
            }
            return;
        }
        if self.button("Entrar em partida (rede local)", bx, 548.0, bw, bh, hex(0x8a3fc0)) {
            match net::Client::new() {
                Ok(mut c) => {
                    c.pkind = self.pick_power();
                    self.client = Some(c);
                    self.status.clear();
                    self.screen = Screen::JoinLobby;
                }
                Err(e) => self.status = format!("Rede indisponível: {}", e),
            }
            return;
        }
        let small = hex(0x34405e);
        let label = self.power_label();
        if self.button(&label, 40.0, 618.0, 330.0, 52.0, hex(0x8a5a10)) {
            self.my_power = (self.my_power + 1) % (POWER_KINDS + 1);
        }
        if self.button(&format!("Computador: {}", LEVEL_NAMES[self.level]), 385.0, 618.0, 290.0, 52.0, small) {
            self.level = (self.level + 1) % 3;
        }
        if self.button(&format!("Tempo: {} s", MATCH_LENGTHS[self.len_idx] as i32), 690.0, 618.0, 250.0, 52.0, small) {
            self.len_idx = (self.len_idx + 1) % 3;
        }
        if self.button(if self.sound_on { "Som: ligado" } else { "Som: desligado" }, 955.0, 618.0, 285.0, 52.0, small) {
            self.sound_on = !self.sound_on;
        }
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 700.0, 22.0, hex(0xff8080));
        }
    }

    fn host_lobby(&mut self) {
        if is_key_pressed(KeyCode::Back) || is_key_pressed(KeyCode::Escape) {
            self.to_menu();
            return;
        }
        self.draw_backdrop();
        let now = self.now;
        let joined = self.host.as_mut().map(|h| h.poll(now)).unwrap_or(false);
        text_o("Partida aberta neste aparelho", 640.0, 130.0, 48.0, hex(0xffd34d));
        rrect(300.0, 165.0, 680.0, 120.0, 20.0, with_a(BLACK, 0.45));
        text_o("Endereço deste aparelho:", 640.0, 205.0, 26.0, hex(0xcfe3ff));
        text_o(&self.my_ip, 640.0, 262.0, 54.0, WHITE);
        let dots = ".".repeat(((now * 2.0) as usize) % 4);
        text_o(&format!("Aguardando o outro jogador{}", dots), 640.0, 345.0, 34.0, WHITE);
        let lines = [
            "Os dois aparelhos precisam estar na mesma rede Wi-Fi,",
            "ou um conectado ao roteador (hotspot) do outro. Não usa internet.",
            "No outro aparelho: Entrar em partida. Ela aparece sozinha na lista,",
            "ou digite o endereço acima.",
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
        if is_key_pressed(KeyCode::Back) || is_key_pressed(KeyCode::Escape) {
            self.to_menu();
            return;
        }
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
            self.status = "Não foi possível conectar. Confira o endereço e a rede.".into();
        }

        // found matches
        rrect(60.0, 110.0, 560.0, 470.0, 20.0, with_a(BLACK, 0.45));
        text_o("Partidas encontradas na rede", 340.0, 150.0, 28.0, WHITE);
        let found: Vec<(SocketAddr, String)> = self.client.as_ref().map(|c| c.found.iter().map(|f| (f.addr, f.name.clone())).collect()).unwrap_or_default();
        if found.is_empty() {
            let dots = ".".repeat(((now * 2.0) as usize) % 4);
            text_o(&format!("Procurando{}", dots), 340.0, 220.0, 26.0, hex(0xb8c6e6));
            text_o("Se não aparecer, digite o endereço", 340.0, 270.0, 22.0, hex(0x93a3c8));
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
        text_o("Ou digite o endereço", 940.0, 150.0, 28.0, WHITE);
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
                None => self.status = "Endereço inválido (exemplo: 192.168.0.12)".into(),
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
                    if self.ai_both { "CPU 1" } else { "VOCÊ" }
                } else {
                    "CPU"
                }
            }
            Mode::Local2 => {
                if i == 0 { "JOGADOR 1" } else { "JOGADOR 2" }
            }
            Mode::Host => {
                if i == 0 { "VOCÊ" } else { "AMIGO" }
            }
            Mode::Client => {
                if i == 1 { "VOCÊ" } else { "AMIGO" }
            }
        }
    }

    fn match_overlays(&mut self) {
        let sim = self.sim.clone();
        // pause button
        let pause_hit = self.clicked().map(|(x, y)| (x - 640.0).powi(2) + (y - 126.0).powi(2) < 24.0f32.powi(2)).unwrap_or(false);
        circ(640.0, 126.0, 18.0, with_a(BLACK, 0.55));
        circ_line(640.0, 126.0, 18.0, 2.0, with_a(WHITE, 0.55));
        rectv(633.0, 117.0, 4.5, 18.0, WHITE);
        rectv(642.5, 117.0, 4.5, 18.0, WHITE);
        // the phone's back button: pause (or leave a finished or broken match)
        let back = is_key_pressed(KeyCode::Back);
        let over_box = sim.phase == Phase::Over && self.over_t > 1.2;
        if back && (self.lost || over_box) {
            self.tour = None;
            self.to_menu();
            return;
        }
        if (pause_hit || back || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P)) && !self.paused && !self.lost && !over_box {
            self.paused = true;
            return;
        }

        if self.lost {
            rectv(0.0, 0.0, W, H, with_a(BLACK, 0.6));
            let msg = if self.mode == Mode::Host { "O outro jogador saiu ou perdeu a conexão" } else { "Conexão com a partida perdida" };
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
            if self.button("Continuar", 490.0, 330.0, 300.0, 64.0, hex(0x2f8f4e)) || is_key_pressed(KeyCode::Escape) || back {
                self.paused = false;
            }
            if self.button("Sair para o menu", 490.0, 415.0, 300.0, 64.0, hex(0x5a2d3a)) {
                self.to_menu();
            }
            return;
        }

        if sim.phase == Phase::Over && self.over_t > 1.2 {
            let mine = self.local_players();
            let won = matches!(sim.winner(), Some(w) if mine.contains(&w));
            let score_line = format!("{}  {} x {}  {}", self.player_name(0), sim.score[0], sim.score[1], self.player_name(1));
            if let (Some(t), Mode::Cpu) = (self.tour, self.mode) {
                // ---- the cup
                if won && t.round + 1 >= TOUR_ROUNDS.len() {
                    self.champion_screen();
                    return;
                }
                rectv(0.0, 0.0, W, H, with_a(BLACK, 0.55));
                if won {
                    text_o("VITÓRIA!", 640.0, 240.0, 80.0, hex(0xffd34d));
                    text_o(&score_line, 640.0, 300.0, 36.0, WHITE);
                    text_o(&format!("Próxima fase: {}", TOUR_ROUNDS[t.round + 1].0), 640.0, 350.0, 30.0, hex(0xcfe3ff));
                    if self.button("Próxima fase", 340.0, 390.0, 280.0, 64.0, hex(0x2f8f4e)) {
                        self.tour = Some(Tour { round: t.round + 1 });
                        self.start_match(Mode::Cpu);
                    }
                } else {
                    text_o("ELIMINADO", 640.0, 240.0, 80.0, hex(0xff7a6a));
                    text_o(&score_line, 640.0, 300.0, 36.0, WHITE);
                    text_o(&format!("Você caiu na fase: {}", TOUR_ROUNDS[t.round].0), 640.0, 350.0, 30.0, hex(0xcfe3ff));
                    if self.button("Tentar de novo", 340.0, 390.0, 280.0, 64.0, hex(0x2f8f4e)) {
                        self.tour = Some(Tour { round: 0 });
                        self.start_match(Mode::Cpu);
                    }
                }
                if self.button("Menu", 660.0, 390.0, 280.0, 64.0, hex(0x5a2d3a)) {
                    self.tour = None;
                    self.to_menu();
                }
                return;
            }
            rectv(0.0, 0.0, W, H, with_a(BLACK, 0.55));
            let title = match sim.winner() {
                None => "EMPATE".to_string(),
                Some(w) => {
                    if self.mode == Mode::Local2 || self.ai_both {
                        format!("{} VENCEU!", self.player_name(w))
                    } else if mine.contains(&w) {
                        "VITÓRIA!".to_string()
                    } else {
                        "DERROTA".to_string()
                    }
                }
            };
            text_o(&title, 640.0, 250.0, 80.0, hex(0xffd34d));
            text_o(&score_line, 640.0, 310.0, 36.0, WHITE);
            if sim.overtime {
                text_o("decidido no gol de ouro", 640.0, 350.0, 24.0, hex(0xffe9a0));
            }
            if self.mode == Mode::Client {
                text_o("Aguardando o anfitrião começar outra partida...", 640.0, 370.0, 26.0, hex(0xb8c6e6));
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

    /// The cup is won: the winner on the podium with trophy and confetti.
    fn champion_screen(&mut self) {
        tex_draw(&self.assets.menu_bg, 0.0, 0.0, W, H, WHITE, false, 0.0);
        rectv(0.0, 0.0, W, H, with_a(hex(0x080d1e), 0.18));
        let t = self.now as f32;
        let c0 = self.char_idx[0] % 6;
        // 3-stage cup bracket ribbon
        rrect(310.0, 224.0, 660.0, 44.0, 22.0, with_a(BLACK, 0.55));
        text_o("Quartas  OK        Semifinal  OK        Final  OK", 640.0, 254.0, 22.0, hex(0xffe14a));
        // Winner + Trophy on podium
        ellipse(590.0, 548.0, 82.0, 16.0, with_a(BLACK, 0.32));
        ellipse(740.0, 548.0, 60.0, 14.0, with_a(BLACK, 0.30));
        tex_center(&self.assets.boots[c0], 596.0, 526.0, 96.0, 64.0, WHITE, false, -0.04);
        tex_center(&self.assets.heads[c0], 590.0, 438.0 + (t * 3.0).sin() * 5.0, 186.0, 186.0, WHITE, false, (t * 1.6).sin() * 0.05);
        tex_center(&self.assets.trophy, 742.0, 458.0 + (t * 2.4 + 1.0).sin() * 4.0, 175.0, 175.0, WHITE, false, (t * 1.2).sin() * 0.04);
        text_o("CAMPEÃO!", 640.0, 142.0, 96.0, hex(0xffd34d));
        text_o("Você venceu o campeonato!", 640.0, 198.0, 32.0, WHITE);
        if rand::gen_range(0.0, 1.0) < 0.35 {
            let c = [hex(0xffd34d), hex(0xff5a5a), hex(0x5ad1ff), WHITE, hex(0x7dff7d)][rand::gen_range(0, 5) as usize];
            self.burst(rand::gen_range(100.0, 1180.0), 60.0, 5, c);
        }
        for (idx, p) in self.particles.iter().enumerate() {
            let piece = (idx % 4) as f32;
            let a = (1.0 - p.life / p.max).clamp(0.0, 1.0);
            tex_sub(&self.assets.confetti, piece * 32.0, 0.0, 32.0, 32.0, p.x - 10.0, p.y - 10.0, 20.0, 20.0, Color::new(1.0, 1.0, 1.0, a), false, p.life * 7.0 + piece);
        }
        if self.button("Jogar de novo", 300.0, 600.0, 320.0, 64.0, hex(0x2f8f4e)) {
            self.tour = Some(Tour { round: 0 });
            self.start_match(Mode::Cpu);
        }
        if self.button("Menu", 660.0, 600.0, 320.0, 64.0, hex(0x5a2d3a)) {
            self.tour = None;
            self.to_menu();
        }
    }

    // ------------------------------------------------------------ match drawing

    fn draw_field(&self, live: bool) {
        let bg = &self.assets.stadium_bgs[self.stadium_idx % self.assets.stadium_bgs.len()];
        tex_draw(bg, 0.0, 0.0, W, H, WHITE, false, 0.0);
        if live && (self.sim.phase == Phase::Goal || self.sim.phase == Phase::Over) {
            let t = self.now as f32;
            let pulse = 0.05 + 0.07 * (t * 10.0).sin().abs();
            rectv(0.0, 0.0, W, H, with_a(hex(0xffe14a), pulse));
        }
    }

    fn draw_goal_frames(&self) {
        let gw = 152.0;
        let gh = GROUND - BAR_Y + 26.0;
        let gy = BAR_Y - 12.0;
        tex_draw(&self.assets.goal_left, -8.0, gy, gw, gh, WHITE, false, 0.0);
        tex_draw(&self.assets.goal_left, W - gw + 8.0, gy, gw, gh, WHITE, true, 0.0);
    }

    fn draw_player(&self, i: usize) {
        let p = self.sim.p[i];
        let s = p.side;
        let flip = s < 0.0;
        let cid = self.char_idx[i] % 6;
        let head_tex = &self.assets.heads[cid];
        let boot_tex = &self.assets.boots[cid];
        let x = p.x;
        let y = p.y;
        // ground shadow
        let lift = (GROUND - (y + FEET)).max(0.0);
        let k = (1.0 - lift / 300.0).max(0.3);
        ellipse(x, GROUND + 4.0, 44.0 * k, 10.0 * k, with_a(BLACK, 0.34 * k));
        // power aura
        if p.power_ready > 0.0 {
            let pulse = (self.now as f32 * 12.0).sin() * 5.0;
            circ(x, y + 8.0, HEAD_R + 28.0 + pulse, with_a(hex(0xff8a1a), 0.28));
            circ(x, y + 8.0, HEAD_R + 16.0 + pulse, with_a(hex(0xffe14a), 0.32));
        }
        // back boot (subtle walk step)
        let walk = p.walk.sin() * 7.0;
        let back_x = x - s * 10.0 - walk * s;
        let back_y = y + FEET - 16.0;
        let back_rot = if p.on_ground { -walk * 0.025 * s } else { 0.15 * s };
        tex_center(boot_tex, back_x, back_y, 56.0, 38.0, Color::new(0.72, 0.72, 0.78, 1.0), flip, back_rot);
        // kicking front boot
        let (fx, fy) = p.foot();
        let kick_rot = if p.kick_t >= 0.0 {
            let kk = (p.kick_t / KICK_TIME).clamp(0.0, 1.0);
            let swing = (kk * std::f32::consts::PI).sin();
            if p.kick_kind == 2 {
                -s * 0.72 * swing
            } else {
                -s * 0.38 * swing
            }
        } else if p.on_ground {
            walk * 0.025 * s
        } else {
            -0.12 * s
        };
        tex_center(boot_tex, fx + s * 2.0, fy + 2.0, 66.0, 44.0, WHITE, flip, kick_rot);
        // big caricature head
        let bob = if p.on_ground && p.vx.abs() > 10.0 {
            -(p.walk * 2.0).sin().abs() * 3.5
        } else {
            0.0
        };
        let head_rot = if p.stun > 0.0 {
            (self.now as f32 * 18.0).sin() * 0.24
        } else {
            (p.vx / MOVE_SPEED) * 0.08
        };
        let head_tint = if p.stun > 0.0 {
            Color::new(1.0, 0.82, 0.82, 1.0)
        } else if p.power_ready > 0.0 {
            Color::new(1.0, 0.96, 0.84, 1.0)
        } else {
            WHITE
        };
        let frozen = p.frozen > 0.0;
        let head_rot = if frozen { 0.0 } else { head_rot };
        let head_tint = if frozen { Color::new(0.62, 0.86, 1.0, 1.0) } else { head_tint };
        tex_center(head_tex, x, y - 2.0 + bob, 112.0, 112.0, head_tint, flip, head_rot);
        if frozen {
            tex_center(&self.assets.ice_block, x, y + 6.0, 136.0, 170.0, WHITE, false, 0.0);
        }
        // orbiting dizzy stars when stunned
        if p.stun > 0.0 && !frozen {
            let t = self.now as f32 * 6.5;
            for j in 0..3 {
                let a = t + j as f32 * 2.09;
                star(x + a.cos() * 38.0, y - HEAD_R - 14.0 + a.sin() * 9.0, 8.0, hex(0xffe14a));
            }
        }
    }

    /// How visible the ball is on this device (the invisible power hides it from the opponent).
    fn ball_alpha(&self) -> f32 {
        let b = self.sim.ball;
        if b.invis <= 0.0 {
            return 1.0;
        }
        let owner = b.invis_owner as usize;
        let mine = self.local_players();
        let seen_by_owner_only = !(self.mode == Mode::Local2 || (self.mode == Mode::Cpu && self.ai_both));
        // fade in during the last half second
        let back = (1.0 - b.invis / 0.5).clamp(0.0, 1.0);
        let base: f32 = if seen_by_owner_only && mine.contains(&owner) {
            0.35
        } else {
            // a faint shimmer now and then
            let t = self.now as f32;
            if (t * 1.5).fract() < 0.08 { 0.15 } else { 0.0 }
        };
        base.max(back)
    }

    fn draw_ball(&self) {
        let b = self.sim.ball;
        let r = b.r();
        let alpha = self.ball_alpha();
        let lift = (GROUND - (b.y + r)).max(0.0);
        let k = (1.0 - lift / 400.0).max(0.25);
        ellipse(b.x, GROUND + 4.0, 24.0 * k * r / BALL_R, 6.0 * k, with_a(BLACK, 0.34 * k * alpha));
        let tint = if b.ice > 0.0 {
            Color::new(0.65, 0.9, 1.0, alpha)
        } else if b.bomb > 0.0 {
            let blink = if (b.bomb * if b.bomb < 0.8 { 12.0 } else { 5.0 }).fract() < 0.5 { 1.0 } else { 0.0 };
            Color::new(0.35 + 0.65 * blink, 0.3, 0.3, alpha)
        } else {
            Color::new(1.0, 1.0, 1.0, alpha)
        };
        // fake balls from the clone power look exactly like the real one
        if b.clone_t > 0.0 {
            for c in b.clones {
                tex_center(&self.assets.ball, c[0], c[1], BALL_R * 2.15, BALL_R * 2.15, WHITE, false, b.angle);
            }
        }
        if b.fire > 0.0 {
            circ(b.x, b.y, r + 12.0, with_a(hex(0xff6a00), 0.40 * alpha));
            let d = r * 3.15;
            tex_center(&self.assets.ball_fire, b.x, b.y, d, d, Color::new(1.0, 1.0, 1.0, alpha), false, b.angle * 0.45);
        } else if b.ice > 0.0 {
            circ(b.x, b.y, r + 9.0, with_a(hex(0x9fe6ff), 0.45 * alpha));
            let d = r * 2.45;
            tex_center(&self.assets.ball_ice, b.x, b.y, d, d, Color::new(1.0, 1.0, 1.0, alpha), false, b.angle);
        } else if b.bomb > 0.0 {
            let d = r * 2.55;
            let blink = if (b.bomb * if b.bomb < 0.8 { 12.0 } else { 5.0 }).fract() < 0.5 { 1.0 } else { 0.72 };
            tex_center(&self.assets.ball_bomb, b.x, b.y - 3.0, d, d, Color::new(1.0, blink, blink, alpha), false, b.angle * 0.25);
        } else {
            let d = r * 2.15;
            tex_center(&self.assets.ball, b.x, b.y, d, d, tint, false, b.angle);
        }
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
        for f in &self.fx {
            let k = (f.life / f.max).clamp(0.0, 0.999);
            match f.kind {
                FxKind::Dust => {
                    let frame = (k * 6.0) as usize;
                    tex_sub(&self.assets.fx_dust, frame as f32 * 128.0, 0.0, 128.0, 128.0, f.x - f.size * 0.5, f.y - f.size * 0.5, f.size, f.size, WHITE, f.flip_x, 0.0);
                }
                FxKind::Kick => {
                    let frame = (k * 6.0) as usize;
                    tex_sub(&self.assets.fx_kick, frame as f32 * 128.0, 0.0, 128.0, 128.0, f.x - f.size * 0.5, f.y - f.size * 0.5, f.size, f.size, WHITE, f.flip_x, 0.0);
                }
                FxKind::Boom => {
                    let frame = (k * 8.0) as usize;
                    tex_center(&self.assets.boom[frame.min(7)], f.x, f.y, f.size, f.size, WHITE, false, 0.0);
                }
            }
        }
        VIEW.with(|v| v.set((s, ox, oy)));
        self.draw_hud();
        self.draw_touch_buttons();
    }

    fn draw_hud(&mut self) {
        let sim = &self.sim;
        // scoreboard
        let sb_x = 450.0;
        let sb_y = 2.0;
        let sb_w = 380.0;
        let sb_h = 104.0;
        tex_draw(&self.assets.scoreboard, sb_x, sb_y, sb_w, sb_h, WHITE, false, 0.0);
        text_o(&sim.score[0].to_string(), sb_x + sb_w * 0.194, 72.0, 48.0, hex(0x9be2ff));
        text_o(&sim.score[1].to_string(), sb_x + sb_w * 0.806, 72.0, 48.0, hex(0xffa49b));
        let tl = sim.time_left.max(0.0).ceil() as i32;
        let tcol = if tl <= 10 && sim.phase == Phase::Play { hex(0xff6a5a) } else { hex(0xffe14a) };
        if sim.overtime {
            let pulse = 0.5 + 0.5 * (self.now as f32 * 6.0).sin().abs();
            text_o("OURO", 640.0, 68.0, 30.0, mix(WHITE, hex(0xffd34d), pulse));
        } else {
            text_o(&format!("{}:{:02}", tl / 60, tl % 60), 640.0, 70.0, 36.0, tcol);
        }
        self.hud.score = true;
        self.hud.timer = true;
        // names and power bars
        for i in 0..2 {
            let p = sim.p[i];
            let (bx, name_x) = if i == 0 { (30.0, 30.0) } else { (W - 30.0 - 360.0, W - 30.0) };
            let name = self.player_name(i);
            let nw = text_w(name, 28.0);
            let nx = if i == 0 { name_x } else { name_x - nw };
            text(name, nx + 2.0, 35.0, 28.0, with_a(BLACK, 0.75));
            text(name, nx, 33.0, 28.0, WHITE);
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
            let pname = POWER_NAMES[(p.pkind % POWER_KINDS) as usize].to_uppercase();
            let label = if p.power_ready > 0.0 {
                format!("{}: CHUTE AGORA!", pname)
            } else if p.power >= 1.0 {
                format!("{} PRONTO!", pname)
            } else {
                format!("PODER: {}", pname)
            };
            text_o(&label, bx + 180.0, 65.0, 20.0, WHITE);
        }
        self.hud.power = true;
        if let Some(t) = self.tour {
            let label = format!("Campeonato: {}", TOUR_ROUNDS[t.round.min(2)].0);
            text(&label, 32.0, 100.0, 22.0, with_a(BLACK, 0.75));
            text(&label, 30.0, 98.0, 22.0, hex(0xffe9a0));
        }

        // big centre messages (hidden while the pause or "connection lost" box is open)
        if self.paused || self.lost {
            return;
        }
        match sim.phase {
            Phase::Countdown => {
                // "versus" banner at the very start of the match
                if sim.score == [0, 0] {
                    rrect(250.0, 196.0, 780.0, 164.0, 24.0, with_a(BLACK, 0.5));
                    let bob = (self.now as f32 * 4.0).sin() * 3.0;
                    tex_center(&self.assets.heads[self.char_idx[0] % 6], 400.0, 266.0 + bob, 124.0, 124.0, WHITE, false, 0.0);
                    tex_center(&self.assets.heads[self.char_idx[1] % 6], 880.0, 266.0 - bob, 124.0, 124.0, WHITE, true, 0.0);
                    text_o("VS", 640.0, 288.0, 84.0, hex(0xffd34d));
                    if let Some(t) = self.tour {
                        text_o(TOUR_ROUNDS[t.round.min(2)].0, 640.0, 330.0, 24.0, WHITE);
                    }
                    text_o(self.player_name(0), 400.0, 348.0, 26.0, hex(0x9be2ff));
                    text_o(self.player_name(1), 880.0, 348.0, 26.0, hex(0xffa49b));
                }
                let n = sim.phase_t.ceil() as i32;
                let k = 1.0 + (sim.phase_t.fract()) * 0.4;
                text_o(&n.max(1).to_string(), 640.0, 470.0, 120.0 * k, hex(0xffd34d));
            }
            Phase::Ready => {
                if sim.overtime {
                    text_o("GOL DE OURO!", 640.0, 300.0, 72.0, hex(0xffd34d));
                    text_o("quem marcar primeiro vence", 640.0, 346.0, 28.0, WHITE);
                } else {
                    text_o("PREPARAR...", 640.0, 300.0, 60.0, WHITE);
                }
            }
            Phase::Play => {
                if sim.time_left > sim.match_len - 0.8 {
                    text_o("JÁ!", 640.0, 320.0, 110.0, hex(0xffd34d));
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

    /// A line of key hints at the bottom of the screen (shown instead of the touch buttons).
    fn draw_controls_hint(&self) {
        let msg = match self.mode {
            Mode::Local2 => "JOGADOR 1: A D mover, W pular, F chutar, G chute alto, R poder    |    JOGADOR 2: setas, K L O    |    P pausa",
            _ => "A D mover    |    W pular    |    F chutar    |    G chute alto    |    R poder    |    P pausa    |    T botões",
        };
        let size = 19.0;
        let w = text_w(msg, size) + 44.0;
        let fade = if self.sim.time_left < self.sim.match_len - 10.0 { 0.55 } else { 1.0 };
        rrect(640.0 - w * 0.5, 683.0, w, 30.0, 15.0, with_a(BLACK, 0.5 * fade));
        text(msg, 640.0 - w * 0.5 + 22.0, 704.0, size, with_a(WHITE, 0.95 * fade));
    }

    fn draw_touch_buttons(&mut self) {
        if !self.touch_ui {
            self.draw_controls_hint();
            return;
        }
        let t = self.now as f32;
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
            let (tex, flip) = match b.kind {
                Btn::Left => (&self.assets.btn_left, false),
                Btn::Right => (&self.assets.btn_left, true),
                Btn::Kick => (&self.assets.btn_kick, b.who == 1),
                Btn::Lob => (&self.assets.btn_lob, b.who == 1),
                Btn::Jump => (&self.assets.btn_jump, b.who == 1),
                Btn::Power => (&self.assets.btn_powers[(p.pkind as usize) % 6], false),
            };
            let pulse = if b.kind == Btn::Power && ready {
                1.0 + 0.06 * (t * 8.0).sin()
            } else {
                1.0
            };
            let scale = if down { 0.90 } else { 1.0 } * pulse;
            let sz = b.r * 2.05 * scale;
            let tint = match b.kind {
                Btn::Power if !ready => Color::new(0.45, 0.45, 0.48, 0.62),
                _ if down => WHITE,
                _ => Color::new(1.0, 1.0, 1.0, 0.88),
            };
            tex_center(tex, b.x, b.y, sz, sz, tint, flip, 0.0);
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


// ---------------------------------------------------------------- start-up

fn conf() -> Conf {
    Conf {
        window_title: "Cabeçada".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: cfg!(target_os = "android"),
        window_resizable: true,
        // miniquad's Android code stops the app if the phone has no graphics mode with
        // this many samples, so phones ask for none (their screens are sharp anyway)
        sample_count: if cfg!(target_os = "android") { 1 } else { 4 },
        ..Default::default()
    }
}

enum Run {
    Normal,
    Screenshot(String),
    Shots(String),
    SelfTest(String),
}

fn parse_args() -> Run {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            return Run::Screenshot(args[i + 1].clone());
        }
        if args[i] == "--shots" && i + 1 < args.len() {
            return Run::Shots(args[i + 1].clone());
        }
        if args[i] == "--self-test" && i + 1 < args.len() {
            return Run::SelfTest(args[i + 1].clone());
        }
        i += 1;
    }
    Run::Normal
}

async fn amain(run: Run, prev_fg: isize) {
    platform::log(concat!("Cabecada ", env!("CARGO_PKG_VERSION"), ": starting"));
    let quiet = !matches!(run, Run::Normal);
    let sounds = if quiet { None } else { audio::Sounds::load().await };
    platform::log(if sounds.is_some() { "sounds ready" } else { "no sound" });
    load_game_font();
    let assets = assets::Assets::load();
    platform::log("pictures loaded");
    let mut app = App::new(sounds, assets);
    platform::log("menu");
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
        Run::Shots(dir) => {
            // `--shots <folder>`: one picture of every screen and power, to check how the game looks
            let _ = std::fs::create_dir_all(&dir);
            let scenes = [
                "menu", "host_lobby", "join_lobby", "countdown", "countdown_tour", "play_cpu", "play_touch", "play_local2", "stun",
                "goal", "overtime", "over", "eliminated", "champion", "pause", "pw_fire", "pw_big", "pw_ice", "pw_invis", "pw_bomb",
                "pw_clone",
            ];
            let mut first = true;
            for (n, name) in scenes.iter().enumerate() {
                app.scene_setup(name);
                let last = if name.starts_with("pw_") || *name == "stun" { 14 } else { 90 };
                for f in 0..=last {
                    if first {
                        win::lower_window(prev_fg);
                        first = false;
                    }
                    set_view();
                    clear_background(hex(0x0b1022));
                    app.scene_frame(1.0 / 60.0);
                    if f == last {
                        let img = get_screen_data();
                        img.export_png(&format!("{}/{:02}_{}.png", dir, n + 1, name));
                    }
                    next_frame().await;
                }
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
