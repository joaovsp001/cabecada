//! Cabecada: 1 v 1 big-head football that plays offline.
//! Modes: against the computer, two players on one device, and local-network
//! play where one player's device hosts the match (no internet server).

mod ai;
mod assets;
mod audio;
mod game;
mod music;
mod net;
mod platform;
mod relay;
mod selftest;
mod settings;
mod ui;
mod win;

use ui::SetupKind;

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

/// Like `set_view`, but the 1280x720 box sits at the top of tall screens (matches: the
/// scoreboard stays at the top and the extra room goes to the button band).
pub fn set_view_top() {
    let sw = screen_width();
    let sh = screen_height();
    let s = (sw / W).min(sh / H);
    VIEW.with(|v| v.set((s, (sw - W * s) * 0.5, 0.0)));
}

/// Runs `f` with another view, then puts the current one back.
pub fn with_view<R>(v: (f32, f32, f32), f: impl FnOnce() -> R) -> R {
    let old = view();
    VIEW.with(|c| c.set(v));
    let r = f();
    VIEW.with(|c| c.set(old));
    r
}

/// The screen's edges in the current view's units: (left, top, right, bottom).
pub fn screen_box() -> (f32, f32, f32, f32) {
    let (s, ox, oy) = view();
    (-ox / s, -oy / s, (screen_width() - ox) / s, (screen_height() - oy) / s)
}

/// A 1280x720 picture covering the whole screen: cropped at the edges, never stretched.
pub fn cover_view() -> (f32, f32, f32) {
    let (sw, sh) = (screen_width(), screen_height());
    let s = (sw / W).max(sh / H);
    (s, (sw - W * s) * 0.5, (sh - H * s) * 0.5)
}

pub fn draw_cover(tex: &Texture2D, tint: Color) {
    with_view(cover_view(), || tex_draw(tex, 0.0, 0.0, W, H, tint, false, 0.0));
}

/// A see-through colour over the whole screen.
pub fn dim_screen(c: Color) {
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), c);
}

/// Share of the screen height kept for the button band under the field (Head Ball 2: the
/// field ends at about 80% of the height and the buttons sit in a dark strip below it).
pub const BAND: f32 = 0.20;
/// Exact bottom of the visible stadium pitch (80% of 720p).
pub const WORLD_BOTTOM: f32 = 576.0;

/// The match camera: fits the exact 1280px width across the screen with zero horizontal
/// stretching, mirroring, or overscan. `WORLD_BOTTOM` lands right at the top of the 20%
/// bottom band on 16:9 and wider phones (`20:9`, cropping excess sky at the top); on
/// taller screens (4:3) the field sits flush at `y = 0` and the bottom band grows taller.
pub fn match_view() -> (f32, f32, f32) {
    let (sw, sh) = (screen_width(), screen_height());
    let s = sw / W;
    let oy = (sh * (1.0 - BAND) - WORLD_BOTTOM * s).min(0.0);
    (s, 0.0, oy)
}

/// Where the button band starts, in the current view's units.
pub fn band_top() -> f32 {
    let (ms, _, moy) = match_view();
    let sh = screen_height();
    let screen_top = (WORLD_BOTTOM * ms + moy).min(sh * (1.0 - BAND));
    let (s, _, oy) = view();
    (screen_top - oy) / s
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
fn tint_col(c: Color, t: Color) -> Color {
    Color::new(c.r * t.r, c.g * t.g, c.b * t.b, c.a * t.a)
}

/// Stylized team-colored mini-jersey & shorts connecting a character's caricature head to
/// their boots (distinct kit per character matching `p1..6_boot.png`).
pub(crate) fn draw_char_jersey(cid: usize, cx: f32, top_y: f32, scale: f32, flip: bool, lean: f32, tint: Color) {
    // (jersey primary, stripe/collar trim, cartoon dark outline, shorts)
    const KITS: [(u32, u32, u32, u32); 6] = [
        (0x1f63d8, 0xffffff, 0x0c1938, 0x133880), // 0 Rafa: royal blue + white
        (0xd9233a, 0xffca28, 0x380812, 0x222632), // 1 Zeca: crimson + gold
        (0x18ae48, 0xffffff, 0x082e14, 0x0f5c28), // 2 Dudu: emerald + white
        (0x9622db, 0xffffff, 0x2a0842, 0x451270), // 3 Lia: vivid purple + white
        (0xe85c14, 0xffffff, 0x3b1606, 0x4a2410), // 4 Ruivo: flame orange + white
        (0xf2b918, 0xffffff, 0x3c2b04, 0x352c10), // 5 Rei: royal gold + white
    ];
    let (j_hex, tr_hex, out_hex, sh_hex) = KITS[cid % KITS.len()];
    let jersey = tint_col(hex(j_hex), tint);
    let jersey_hi = tint_col(mix(hex(j_hex), WHITE, 0.28), tint);
    let jersey_sh = tint_col(mix(hex(j_hex), BLACK, 0.28), tint);
    let trim = tint_col(hex(tr_hex), tint);
    let outline = tint_col(hex(out_hex), tint);
    let shorts = tint_col(hex(sh_hex), tint);
    let dir = if flip { -1.0 } else { 1.0 };
    let x = cx + lean * scale;
    let s = scale;
    // Bold dark cartoon outline silhouette behind shoulders, torso and shorts
    ellipse(x, top_y + 13.0 * s, 35.5 * s, 16.0 * s, outline);
    rrect(x - 28.0 * s, top_y + 8.0 * s, 56.0 * s, 30.5 * s, 11.0 * s, outline);
    // Shorts lower band
    rrect(x - 24.5 * s, top_y + 24.0 * s, 49.0 * s, 11.5 * s, 5.0 * s, shorts);
    rectv(x - 22.0 * s, top_y + 25.0 * s, 3.0 * s, 9.0 * s, with_a(trim, 0.85 * tint.a));
    rectv(x + 19.0 * s, top_y + 25.0 * s, 3.0 * s, 9.0 * s, with_a(trim, 0.85 * tint.a));
    // Rounded shoulders & jersey body
    ellipse(x, top_y + 13.0 * s, 31.5 * s, 13.0 * s, jersey);
    rrect(x - 24.5 * s, top_y + 9.0 * s, 49.0 * s, 17.5 * s, 8.0 * s, jersey);
    // Depth shading on back & glossy chest highlight on front
    ellipse(x - dir * 13.0 * s, top_y + 15.0 * s, 14.0 * s, 10.0 * s, with_a(jersey_sh, 0.52 * tint.a));
    ellipse(x + dir * 10.0 * s, top_y + 12.0 * s, 15.0 * s, 7.5 * s, with_a(jersey_hi, 0.55 * tint.a));
    // Twin vertical jersey stripes + lower waist trim
    rectv(x + dir * 5.0 * s - 2.0 * s, top_y + 11.5 * s, 4.0 * s, 13.5 * s, with_a(trim, 0.92 * tint.a));
    rectv(x + dir * 13.0 * s - 2.0 * s, top_y + 11.5 * s, 4.0 * s, 13.5 * s, with_a(trim, 0.92 * tint.a));
    rectv(x - 24.5 * s, top_y + 23.0 * s, 49.0 * s, 2.8 * s, trim);
    // V-neck collar peeking cleanly beneath the jaw
    tri(
        (x - 11.5 * s, top_y + 5.0 * s),
        (x + 11.5 * s, top_y + 5.0 * s),
        (x + dir * 2.0 * s, top_y + 16.5 * s),
        trim,
    );
    tri(
        (x - 7.8 * s, top_y + 5.0 * s),
        (x + 7.8 * s, top_y + 5.0 * s),
        (x + dir * 2.0 * s, top_y + 12.8 * s),
        outline,
    );
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

impl Btn {
    /// Its place in `Settings::btn_pos`.
    pub fn slot(self) -> usize {
        match self {
            Btn::Left => 0,
            Btn::Right => 1,
            Btn::Kick => 2,
            Btn::Lob => 3,
            Btn::Jump => 4,
            Btn::Power => 5,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TouchBtn {
    pub who: usize,
    pub kind: Btn,
    pub x: f32, // centre
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl TouchBtn {
    /// A finger at (x, y) presses it: a little extra room above and below, but less than
    /// half the gap at the sides, so one finger never presses two neighbours.
    pub fn hit(&self, x: f32, y: f32) -> bool {
        (x - self.x).abs() < self.w * 0.5 + 3.5 && (y - self.y).abs() < self.h * 0.5 + 22.0
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Cpu,
    Local2,
    Host,
    Client,
}

/// Where the on-screen buttons go, measured on Head Ball 2: one row in the dark band under
/// the field, about 1/8 of the screen height tall and 1.45 times as wide; the two arrows
/// near the left corner, kick, high kick and jump near the right corner (distances from
/// the screen edges in screen heights), and the power button in the top-right corner.
/// The size setting scales them, and buttons the player moved go where they were put.
pub fn touch_layout(mode: Mode, set: &settings::Settings) -> Vec<TouchBtn> {
    let scale = set.btn_scale as f32 / 100.0;
    let (left, top, right, bottom) = screen_box();
    let width = right - left;
    let hs = bottom - top;
    let band = band_top().clamp(top + hs * 0.5, bottom - hs * 0.1);
    let y = (band + bottom) * 0.5;
    let band_h = bottom - band;
    let mut v = vec![];
    match mode {
        Mode::Local2 => {
            // each player gets half of the band, mirrored: the arrows at the outer edge,
            // then kick, high kick, jump and power towards the middle
            let bw = (0.125 * hs * 1.45 * scale * 0.82).min((width * 0.5 - 68.0) / 5.69).min(band_h * 0.8 * 1.45);
            let bh = bw / 1.45;
            for who in 0..2 {
                let (outer, inner) = if who == 0 { (Btn::Left, Btn::Right) } else { (Btn::Right, Btn::Left) };
                let mut pos = 18.0; // from the screen edge to the next button
                for (kind, w) in [(outer, bw), (inner, bw), (Btn::Kick, bw), (Btn::Lob, bw), (Btn::Jump, bw), (Btn::Power, bh)] {
                    let c = pos + w * 0.5;
                    let x = if who == 0 { left + c } else { right - c };
                    v.push(TouchBtn { who, kind, x, y, w, h: bh });
                    pos += w + 8.0;
                }
            }
            return v;
        }
        _ => {
            let who = if mode == Mode::Client { 1 } else { 0 };
            let bh = (0.125 * hs * scale).min(band_h * 0.92);
            let bw = bh * 1.45;
            let kinds = [Btn::Left, Btn::Right, Btn::Kick, Btn::Lob, Btn::Jump];
            let mut xs = [left + 0.231 * hs, left + 0.515 * hs, right - 0.640 * hs, right - 0.418 * hs, right - 0.187 * hs];
            // keep them apart and inside the screen, whatever its shape and the size setting
            let gap = |i: usize| if i == 2 { 40.0 } else { 10.0 }; // space before button i
            xs[0] = xs[0].max(left + 12.0 + bw * 0.5);
            for i in 1..5 {
                xs[i] = xs[i].max(xs[i - 1] + bw + gap(i));
            }
            xs[4] = xs[4].min(right - 12.0 - bw * 0.5);
            for i in (0..4).rev() {
                xs[i] = xs[i].min(xs[i + 1] - bw - gap(i + 1));
            }
            for i in 0..5 {
                v.push(TouchBtn { who, kind: kinds[i], x: xs[i], y, w: bw, h: bh });
            }
            // power: the big green button in the top-right corner
            let ph = 0.14 * hs * scale.min(1.25);
            let pw = ph * 1.15;
            v.push(TouchBtn { who, kind: Btn::Power, x: right - pw * 0.5 - 0.012 * hs, y: top + ph * 0.5 + 0.035 * hs, w: pw, h: ph });
        }
    }
    // buttons the player moved (thousandths of the screen), always kept on the screen
    for b in v.iter_mut() {
        let p = set.btn_pos[b.kind.slot()];
        if p[0] != 0 {
            b.x = left + width * p[0] as f32 / 1000.0;
            b.y = top + hs * p[1] as f32 / 1000.0;
        }
        b.x = b.x.clamp(left + b.w * 0.5, right - b.w * 0.5);
        b.y = b.y.clamp(top + b.h * 0.5, bottom - b.h * 0.5);
    }
    v
}

/// A position on the screen in thousandths (for `Settings::btn_pos`).
pub fn screen_frac(x: f32, y: f32) -> [u16; 2] {
    let (l, t, r, b) = screen_box();
    let f = |v: f32, a: f32, z: f32| (((v - a) / (z - a)) * 1000.0).round().clamp(1.0, 1000.0) as u16;
    [f(x, l, r), f(y, t, b)]
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
    Multi,
    Setup,
    Settings,
    ButtonTest,
    HostLobby,
    JoinLobby,
    OnlineHost,
    OnlineJoin,
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
    grav: f32,
    behind: bool,
}

#[derive(Clone, Copy)]
enum FxKind {
    Dust,
    Kick,
    KickSwing,
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

struct IceShard {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rot: f32,
    vrot: f32,
    size: f32,
    life: f32,
    max: f32,
    col: Color,
}

struct IceBeam {
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    life: f32,
    max: f32,
}

struct ShockRing {
    x: f32,
    y: f32,
    r0: f32,
    r1: f32,
    thick: f32,
    life: f32,
    max: f32,
    col: Color,
}

struct ScorchMark {
    x: f32,
    life: f32,
    max: f32,
}

struct ConfettiPiece {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    rot: f32,
    vrot: f32,
    phase: f32,
    piece: u8,
    size: f32,
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
    pub host: Option<net::Host>,
    pub client: Option<net::Client>,
    acc: f32,
    pub now: f64,
    paused: bool,
    over_t: f32,
    sounds: Option<audio::Sounds>,
    ip_text: String,
    status: String,
    connect_t: f64,
    my_ip: String,
    particles: Vec<Particle>,
    shake: f32,
    shake_mag: f32,
    shake_dur: f32,
    goal_anim: f32,
    net_bulge: [f32; 2],
    net_bulge_vel: [f32; 2],
    net_hit_y: [f32; 2],
    post_rattle: [f32; 2],
    ball_squash: f32,
    ball_squash_vert: bool,
    ice_flash: f32,
    ice_shiver: [f32; 2],
    ice_pop: [f32; 2],
    prev_frozen: [f32; 2],
    prev_big: f32,
    big_anim: f32,
    big_shrink: f32,
    prev_invis: f32,
    prev_clone: f32,
    prev_clones: [[f32; 2]; 2],
    shards: Vec<IceShard>,
    ice_beams: Vec<IceBeam>,
    rings: Vec<ShockRing>,
    scorches: Vec<ScorchMark>,
    confetti: Vec<ConfettiPiece>,
    pub hud: HudFlags,
    pub held: [Input; 2],
    lost: bool,
    assets: assets::Assets,
    pub touch_ui: bool, // on-screen buttons (always on phones, after the first touch elsewhere, or with T)
    prev_ground: [bool; 2],
    prev_kick: [bool; 2],
    tour: Option<Tour>,
    crowd_on: bool,
    fx: Vec<AnimFx>,
    pub set: settings::Settings, // saved choices (volumes, buttons, last match options)
    setup: SetupKind,            // what the setup screen is preparing
    settings_back: Screen,       // where "Voltar" leaves the settings screen
    room_code: String,           // online room being hosted
    code_text: String,           // online room code being typed
    music: Option<macroquad::audio::Sound>,
    music_on: bool,
    drag: Option<(usize, f32, f32)>, // the button being moved in the button editor
    quit_ask: bool,     // the "leave the game?" box is open
    events_sub: usize,  // macroquad input subscriber, to hear when the app goes to the background
}

/// A knock-out cup against the computer: three rounds, each harder than the last.
#[derive(Clone, Copy)]
struct Tour {
    round: usize,
}
const TOUR_ROUNDS: [(&str, usize); 3] = [("Quartas de final", 0), ("Semifinal", 1), ("Final", 2)];

pub const LEVEL_NAMES: [&str; 3] = ["Fácil", "Normal", "Difícil"];

impl App {
    fn new(sounds: Option<audio::Sounds>, music: Option<macroquad::audio::Sound>, assets: assets::Assets) -> App {
        App {
            screen: Screen::Menu,
            mode: Mode::Cpu,
            sim: Sim::new(90.0),
            ai: [ai::AiMem { rng: 77, ..Default::default() }, ai::AiMem { rng: 991, ..Default::default() }],
            ai_both: false,
            host: None,
            client: None,
            acc: 0.0,
            now: 0.0,
            paused: false,
            over_t: 0.0,
            sounds,
            ip_text: String::new(),
            status: String::new(),
            connect_t: 0.0,
            my_ip: String::new(),
            particles: vec![],
            shake: 0.0,
            shake_mag: 0.0,
            shake_dur: 0.25,
            goal_anim: 0.0,
            net_bulge: [0.0; 2],
            net_bulge_vel: [0.0; 2],
            net_hit_y: [430.0; 2],
            post_rattle: [0.0; 2],
            ball_squash: 0.0,
            ball_squash_vert: false,
            ice_flash: 0.0,
            ice_shiver: [0.0; 2],
            ice_pop: [1.0; 2],
            prev_frozen: [0.0; 2],
            prev_big: 0.0,
            big_anim: 1.0,
            big_shrink: 0.0,
            prev_invis: 0.0,
            prev_clone: 0.0,
            prev_clones: [[0.0; 2]; 2],
            shards: vec![],
            ice_beams: vec![],
            rings: vec![],
            scorches: vec![],
            confetti: vec![],
            hud: HudFlags::default(),
            held: [Input::default(); 2],
            lost: false,
            assets,
            touch_ui: cfg!(target_os = "android"),
            prev_ground: [true; 2],
            prev_kick: [false; 2],
            tour: None,
            crowd_on: false,
            fx: vec![],
            set: settings::Settings::load(),
            setup: SetupKind::VsCpu,
            settings_back: Screen::Menu,
            room_code: String::new(),
            code_text: String::new(),
            music,
            music_on: false,
            drag: None,
            quit_ask: false,
            events_sub: macroquad::input::utils::register_input_subscriber(),
        }
    }

    fn clear_vfx(&mut self) {
        self.particles.clear();
        self.fx.clear();
        self.shards.clear();
        self.ice_beams.clear();
        self.rings.clear();
        self.scorches.clear();
        self.confetti.clear();
        self.shake = 0.0;
        self.shake_mag = 0.0;
        self.shake_dur = 0.25;
        self.goal_anim = 0.0;
        self.net_bulge = [0.0; 2];
        self.net_bulge_vel = [0.0; 2];
        self.net_hit_y = [430.0; 2];
        self.post_rattle = [0.0; 2];
        self.ball_squash = 0.0;
        self.ball_squash_vert = false;
        self.ice_flash = 0.0;
        self.ice_shiver = [0.0; 2];
        self.ice_pop = [1.0; 2];
        self.prev_frozen = [0.0; 2];
        self.prev_big = 0.0;
        self.big_anim = 1.0;
        self.big_shrink = 0.0;
        self.prev_invis = 0.0;
        self.prev_clone = 0.0;
        self.prev_clones = [[0.0; 2]; 2];
        self.prev_ground = [true; 2];
        self.prev_kick = [false; 2];
    }

    /// Computer level in use: the cup sets its own, otherwise the menu choice.
    fn cpu_level(&self) -> usize {
        match self.tour {
            Some(t) => TOUR_ROUNDS[t.round.min(2)].1,
            None => self.set.level as usize,
        }
    }

    pub fn start_match(&mut self, mode: Mode) {
        self.mode = mode;
        self.sim = Sim::new(MATCH_LENGTHS[self.set.len_idx as usize % MATCH_LENGTHS.len()]);
        self.screen = Screen::Match;
        self.paused = false;
        self.over_t = 0.0;
        self.acc = 0.0;
        self.lost = false;
        self.clear_vfx();
        if mode != Mode::Cpu {
            self.tour = None; // the cup is only against the computer
        }
        // faces and attributes: yours from the setup; the opponent's from the setup, the cup
        // round, or the other device (sent in its hello)
        let me = self.set.look % 6;
        let mut other = self.set.look2 % 6;
        let mut stadium = self.set.stadium % 3;
        if let Some(t) = self.tour {
            let r = t.round.min(2);
            stadium = r as u8;
            let boss = [1u8, 3, 5][r];
            other = if boss == me { (boss + 1) % 6 } else { boss };
        } else if other == me && mode != Mode::Host {
            other = (me + 1) % 6;
        }
        if mode == Mode::Host {
            other = self.host.as_ref().map(|h| h.remote_look % 6).unwrap_or(other);
        }
        self.sim.p[0].look = me;
        self.sim.p[1].look = other;
        self.sim.stadium = stadium; // a guest gets it with the match state
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
        if self.set.power >= POWER_KINDS {
            rand::gen_range(0, POWER_KINDS as u32) as u8
        } else {
            self.set.power
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
        match self.set.touch_mode {
            1 => self.touch_ui = true,
            2 if !cfg!(target_os = "android") => self.touch_ui = false,
            _ => {
                if !touches().is_empty() {
                    self.touch_ui = true; // a finger on the screen: show the on-screen buttons
                }
                if is_key_pressed(KeyCode::T) {
                    self.touch_ui = !self.touch_ui;
                }
            }
        }
        if self.touch_ui {
            let pts = pointers();
            for b in touch_layout(self.mode, &self.set) {
                if pts.iter().any(|&(x, y)| b.hit(x, y)) {
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

    fn add_shake(&mut self, dur: f32, mag: f32) {
        if mag >= self.shake_mag || self.shake <= 0.02 {
            self.shake = dur;
            self.shake_dur = dur.max(0.05);
            self.shake_mag = mag;
        }
    }

    fn spawn_ice_shards(&mut self, x: f32, y: f32, n: usize, strong: bool) {
        for i in 0..n {
            let a = (i as f32 / n.max(1) as f32) * std::f32::consts::TAU + rand::gen_range(-0.3, 0.3);
            let sp = if strong { rand::gen_range(180.0, 520.0) } else { rand::gen_range(110.0, 290.0) };
            let c = match i % 3 {
                0 => hex(0x88e6ff),
                1 => hex(0xd4f7ff),
                _ => WHITE,
            };
            self.shards.push(IceShard {
                x: x + rand::gen_range(-34.0, 34.0),
                y: y + rand::gen_range(-56.0, 44.0),
                vx: a.cos() * sp,
                vy: a.sin() * sp - if strong { rand::gen_range(160.0, 320.0) } else { rand::gen_range(80.0, 180.0) },
                rot: rand::gen_range(0.0, std::f32::consts::TAU),
                vrot: rand::gen_range(-11.0, 11.0),
                size: if strong { rand::gen_range(9.0, 19.0) } else { rand::gen_range(5.0, 11.0) },
                life: 0.0,
                max: if strong { rand::gen_range(0.55, 0.95) } else { rand::gen_range(0.32, 0.55) },
                col: c,
            });
        }
    }

    fn thaw_burst(&mut self, x: f32, y: f32) {
        self.spawn_ice_shards(x, y + 6.0, 18, true);
        self.rings.push(ShockRing {
            x,
            y: y + 10.0,
            r0: 36.0,
            r1: 155.0,
            thick: 9.0,
            life: 0.0,
            max: 0.32,
            col: hex(0xb8f4ff),
        });
        for _ in 0..14 {
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            let sp = rand::gen_range(60.0, 210.0);
            self.particles.push(Particle {
                x: x + rand::gen_range(-28.0, 28.0),
                y: y + rand::gen_range(-40.0, 40.0),
                vx: a.cos() * sp,
                vy: a.sin() * sp - 40.0,
                life: 0.0,
                max: rand::gen_range(0.35, 0.65),
                r: rand::gen_range(6.0, 13.0),
                c: Color::new(0.82, 0.96, 1.0, 0.78),
                grav: -35.0,
                behind: false,
            });
        }
        self.add_shake(0.14, 5.0);
    }

    fn spawn_goal_confetti(&mut self, gx: f32, gy: f32, n: usize) {
        for i in 0..n {
            let from_top = i >= n / 2;
            let (x, y, vx, vy) = if from_top {
                (
                    rand::gen_range(60.0, W - 60.0),
                    rand::gen_range(-10.0, 80.0),
                    rand::gen_range(-80.0, 80.0),
                    rand::gen_range(70.0, 210.0),
                )
            } else {
                let dir = if gx > W * 0.5 { -1.0 } else { 1.0 };
                (
                    gx + rand::gen_range(-25.0, 25.0),
                    gy + rand::gen_range(-50.0, 40.0),
                    dir * rand::gen_range(120.0, 460.0) + rand::gen_range(-100.0, 100.0),
                    rand::gen_range(-480.0, -140.0),
                )
            };
            self.confetti.push(ConfettiPiece {
                x,
                y,
                vx,
                vy,
                rot: rand::gen_range(0.0, std::f32::consts::TAU),
                vrot: rand::gen_range(-9.0, 9.0),
                phase: rand::gen_range(0.0, std::f32::consts::TAU),
                piece: (i % 4) as u8,
                size: rand::gen_range(16.0, 26.0),
                life: 0.0,
                max: rand::gen_range(1.6, 2.5),
            });
        }
    }

    fn play_events(&mut self, ev: u32) {
        let b = self.sim.ball;
        if ev & EV_KICK != 0 {
            self.ball_squash = 0.075;
            self.ball_squash_vert = b.vy.abs() > b.vx.abs() * 1.15;
            self.fx.push(AnimFx {
                kind: FxKind::Kick,
                x: b.x,
                y: b.y,
                size: 112.0,
                flip_x: b.vx < 0.0,
                life: 0.0,
                max: 0.22,
            });
            self.burst(b.x, b.y, 7, WHITE);
            self.burst(b.x, b.y, 4, hex(0xffe14a));
        }
        if ev & EV_HEAD != 0 {
            self.ball_squash = 0.070;
            self.ball_squash_vert = b.vy.abs() > b.vx.abs();
            self.burst(b.x, b.y, 6, WHITE);
        }
        if ev & EV_BOUNCE != 0 {
            self.ball_squash = 0.072;
            self.ball_squash_vert = false;
            if b.y + b.r() >= GROUND - 16.0 {
                if b.big > 0.0 {
                    self.dust_sized(b.x, GROUND, 10, 120.0);
                    self.add_shake(0.11, 3.5);
                } else if b.invis > 0.0 {
                    self.dust_sized(b.x, GROUND, 4, 62.0);
                }
            }
        }
        if ev & EV_POST != 0 {
            self.ball_squash = 0.075;
            self.ball_squash_vert = b.vy.abs() > b.vx.abs();
            let gi = if b.x < W * 0.5 { 0 } else { 1 };
            self.post_rattle[gi] = 0.24;
            self.burst(b.x, b.y, 6, hex(0xfff0a8));
        }
        if ev & EV_NET_HIT != 0 && ev & EV_GOAL == 0 {
            let gi = if b.x < W * 0.5 { 0 } else { 1 };
            self.net_bulge_vel[gi] = 12.0;
            self.net_hit_y[gi] = b.y.clamp(BAR_Y + 30.0, GROUND - 20.0);
        }
        if ev & EV_GOAL != 0 {
            self.goal_anim = 0.0;
            let gi = if self.sim.last_scorer == 0 { 1 } else { 0 };
            self.net_bulge_vel[gi] = 17.5;
            self.net_hit_y[gi] = b.y.clamp(BAR_Y + 30.0, GROUND - 20.0);
            self.add_shake(0.34, 11.5);
            let gx = if self.sim.last_scorer == 0 { W - 64.0 } else { 64.0 };
            self.spawn_goal_confetti(gx, 420.0, 52);
            for c in [hex(0xffd34d), hex(0xff5a5a), hex(0x5ad1ff), WHITE] {
                self.burst(gx, 420.0, 10, c);
            }
        }
        if ev & EV_STUN != 0 {
            if b.fire > 0.0 {
                self.add_shake(0.26, 10.5);
                self.rings.push(ShockRing {
                    x: b.x,
                    y: b.y,
                    r0: 24.0,
                    r1: 145.0,
                    thick: 10.0,
                    life: 0.0,
                    max: 0.24,
                    col: hex(0xff9224),
                });
                self.burst(b.x, b.y, 16, hex(0xff8020));
            } else {
                self.add_shake(0.22, 6.5);
            }
        }
        if ev & EV_FIRE != 0 {
            self.add_shake(0.18, 7.5);
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: 24.0,
                r1: 120.0,
                thick: 8.0,
                life: 0.0,
                max: 0.25,
                col: hex(0xffa020),
            });
            self.burst(b.x, b.y, 18, hex(0xffa020));
        }
        if ev & EV_BOOM != 0 {
            self.add_shake(0.38, 15.0);
            self.fx.push(AnimFx {
                kind: FxKind::Boom,
                x: b.x,
                y: b.y,
                size: 295.0,
                flip_x: false,
                life: 0.0,
                max: 0.50,
            });
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: 22.0,
                r1: BOMB_RADIUS * 1.08,
                thick: 12.0,
                life: 0.0,
                max: 0.34,
                col: hex(0xff7a20),
            });
            self.scorches.push(ScorchMark { x: b.x.clamp(40.0, W - 40.0), life: 0.0, max: 5.5 });
            for c in [hex(0xff5a1a), hex(0xffd34d), hex(0x555555)] {
                self.burst(b.x, b.y, 16, c);
            }
        }
        if ev & (EV_ICE_CAST | EV_FREEZE) != 0 {
            self.ice_flash = 0.28;
            self.add_shake(0.16, 5.5);
            for i in 0..2 {
                let target = self.sim.p[i];
                if target.frozen > 0.0 && (self.prev_frozen[i] <= 0.0 || target.frozen > FREEZE_TIME - 0.2) {
                    let caster = self.sim.p[1 - i];
                    self.ice_shiver[i] = 0.24;
                    self.ice_pop[i] = 0.0;
                    self.ice_beams.push(IceBeam {
                        from_x: caster.x,
                        from_y: caster.y - 6.0,
                        to_x: target.x,
                        to_y: target.y + 4.0,
                        life: 0.0,
                        max: 0.42,
                    });
                    self.rings.push(ShockRing {
                        x: target.x,
                        y: target.y + 8.0,
                        r0: 26.0,
                        r1: 150.0,
                        thick: 9.0,
                        life: 0.0,
                        max: 0.32,
                        col: hex(0x74e8ff),
                    });
                    for c in [hex(0x9fe6ff), WHITE] {
                        self.burst(target.x, target.y, 10, c);
                    }
                }
            }
        }
        if ev & EV_ICE_CRACK != 0 {
            for i in 0..2 {
                let p = self.sim.p[i];
                if p.frozen > 0.0 {
                    self.ice_shiver[i] = 0.16;
                    self.spawn_ice_shards(p.x, p.y + 6.0, 6, false);
                    self.add_shake(0.08, 3.0);
                }
            }
        }
        if ev & EV_BIG_CAST != 0 {
            self.big_anim = 0.001;
            self.big_shrink = 0.0;
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: BALL_R,
                r1: 150.0,
                thick: 9.0,
                life: 0.0,
                max: 0.34,
                col: hex(0xffd34d),
            });
            self.burst(b.x, b.y, 14, hex(0xffe14a));
        }
        if ev & EV_INVIS_CAST != 0 {
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: 18.0,
                r1: 110.0,
                thick: 7.0,
                life: 0.0,
                max: 0.30,
                col: hex(0xa8f2ff),
            });
            for _ in 0..18 {
                let a = rand::gen_range(0.0, std::f32::consts::TAU);
                let sp = rand::gen_range(40.0, 180.0);
                self.particles.push(Particle {
                    x: b.x + rand::gen_range(-14.0, 14.0),
                    y: b.y + rand::gen_range(-14.0, 14.0),
                    vx: a.cos() * sp,
                    vy: a.sin() * sp - 30.0,
                    life: 0.0,
                    max: rand::gen_range(0.28, 0.55),
                    r: rand::gen_range(4.0, 10.0),
                    c: Color::new(0.84, 0.96, 1.0, 0.85),
                    grav: -50.0,
                    behind: false,
                });
            }
        }
        if ev & EV_CLONE_CAST != 0 {
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: 20.0,
                r1: 120.0,
                thick: 8.0,
                life: 0.0,
                max: 0.28,
                col: WHITE,
            });
            self.burst(b.x, b.y, 14, hex(0xb8f0ff));
            self.burst(b.x, b.y, 8, WHITE);
        }
        let g = self.sfx_gain();
        let Some(s) = self.sounds.as_ref() else { return };
        if g <= 0.0 {
            return;
        }
        let list = [
            (EV_KICK, &s.kick, 0.9),
            (EV_HEAD, &s.head, 0.8),
            (EV_BOUNCE, &s.bounce, 0.5),
            (EV_POST, &s.post, 0.7),
            (EV_WHISTLE, &s.whistle, 0.6),
            (EV_GOAL, &s.goal, 0.85),
            (EV_BEEP, &s.beep, 0.6),
            (EV_POWER, &s.power, 0.7),
            (EV_FIRE, &s.power, 0.9),
            (EV_STUN, &s.stun, 0.7),
            (EV_BOOM, &s.goal, 1.0),
            (EV_BOOM, &s.bounce, 1.0),
            (EV_ICE_CRACK, &s.ice_crack, 0.80),
            (EV_THAW, &s.ice_shatter, 0.90),
            (EV_BIG_CAST, &s.power_pop, 0.85),
            (EV_INVIS_CAST, &s.power_pop, 0.75),
            (EV_CLONE_CAST, &s.power_pop, 0.85),
        ];
        for (bit, snd, vol) in list {
            if ev & bit != 0 {
                s.play(snd, vol * g);
            }
        }
        if ev & (EV_ICE_CAST | EV_FREEZE) != 0 {
            s.play(&s.ice_cast, 0.85 * g);
        }
        if ev & EV_NET_HIT != 0 && ev & EV_GOAL == 0 {
            s.play(&s.bounce, 0.35 * g);
        }
    }

    /// A puff of dust on the grass (landing, take-off, big-ball bounce).
    fn dust(&mut self, x: f32, y: f32, n: usize) {
        self.dust_sized(x, y, n, 86.0);
    }

    fn dust_sized(&mut self, x: f32, y: f32, n: usize, size: f32) {
        self.fx.push(AnimFx {
            kind: FxKind::Dust,
            x,
            y: y - size * 0.25,
            size,
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
                grav: 500.0,
                behind: false,
            });
        }
    }

    /// Jump / landing dust, subtle kick swooshes, and state-transition VFX (so joining
    /// network clients also see freeze, thaw, big, invis, and clone transitions cleanly).
    fn detect_fx(&mut self, ev: u32) {
        let mut thawed_any = false;
        for i in 0..2 {
            let p = self.sim.p[i];
            let near_grass = p.y + FEET > GROUND - 34.0;
            if p.on_ground && !self.prev_ground[i] && near_grass {
                self.dust(p.x, GROUND, 7);
                self.play_one(|s| (&s.land, 0.35));
            } else if !p.on_ground && self.prev_ground[i] && p.vy < -220.0 && near_grass {
                self.dust(p.x, GROUND, 5);
                self.play_one(|s| (&s.jump, 0.3));
            }
            self.prev_ground[i] = p.on_ground;
            let kicking = p.kick_t >= 0.0;
            if kicking && !self.prev_kick[i] {
                // Subtle air swing swoosh only — bright impact sparks fire exclusively on EV_KICK
                let (fx, fy) = p.foot();
                self.fx.push(AnimFx {
                    kind: FxKind::KickSwing,
                    x: fx + p.side * 24.0,
                    y: fy - 6.0,
                    size: 72.0,
                    flip_x: p.side < 0.0,
                    life: 0.0,
                    max: 0.14,
                });
            }
            self.prev_kick[i] = kicking;
            // Freeze start / thaw shatter detection
            if p.frozen > 0.0 && self.prev_frozen[i] <= 0.0 && ev & (EV_ICE_CAST | EV_FREEZE) == 0 {
                let caster = self.sim.p[1 - i];
                self.ice_flash = 0.28;
                self.ice_shiver[i] = 0.22;
                self.ice_pop[i] = 0.0;
                self.ice_beams.push(IceBeam {
                    from_x: caster.x,
                    from_y: caster.y - 6.0,
                    to_x: p.x,
                    to_y: p.y + 4.0,
                    life: 0.0,
                    max: 0.42,
                });
            } else if p.frozen <= 0.0 && self.prev_frozen[i] > 0.0 {
                self.thaw_burst(p.x, p.y);
                thawed_any = true;
            }
            self.prev_frozen[i] = p.frozen;
        }
        if ev & EV_THAW != 0 && !thawed_any {
            // Defensive fallback if EV_THAW arrived after a single-frame freeze
            let p = self.sim.p[1];
            self.thaw_burst(p.x, p.y);
        }
        let b = self.sim.ball;
        // Giant ball inflation / deflation transition
        if b.big > 0.0 && self.prev_big <= 0.0 && ev & EV_BIG_CAST == 0 {
            self.big_anim = 0.001;
            self.big_shrink = 0.0;
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: BALL_R,
                r1: 150.0,
                thick: 9.0,
                life: 0.0,
                max: 0.34,
                col: hex(0xffd34d),
            });
        } else if b.big <= 0.0 && self.prev_big > 0.0 {
            self.big_shrink = 0.24;
            self.burst(b.x, b.y, 10, hex(0xffe14a));
        }
        self.prev_big = b.big;
        // Invisible ball dissolve / re-materialization
        if b.invis <= 0.0 && self.prev_invis > 0.0 {
            self.rings.push(ShockRing {
                x: b.x,
                y: b.y,
                r0: 14.0,
                r1: 88.0,
                thick: 6.0,
                life: 0.0,
                max: 0.24,
                col: WHITE,
            });
            self.burst(b.x, b.y, 10, hex(0xd0f6ff));
        }
        self.prev_invis = b.invis;
        // Clone tracking + pop poof when clones expire
        if b.clone_t > 0.0 {
            self.prev_clones = [[b.clones[0][0], b.clones[0][1]], [b.clones[1][0], b.clones[1][1]]];
        } else if self.prev_clone > 0.0 {
            let last = self.prev_clones;
            for [cx, cy] in last {
                self.rings.push(ShockRing {
                    x: cx,
                    y: cy,
                    r0: 14.0,
                    r1: 72.0,
                    thick: 6.0,
                    life: 0.0,
                    max: 0.24,
                    col: WHITE,
                });
                self.burst(cx, cy, 10, hex(0xe8f6ff));
            }
        }
        self.prev_clone = b.clone_t;
    }

    fn play_one(&self, pick: impl Fn(&audio::Sounds) -> (&macroquad::audio::Sound, f32)) {
        let g = self.sfx_gain();
        if g <= 0.0 {
            return;
        }
        if let Some(s) = self.sounds.as_ref() {
            let (snd, vol) = pick(s);
            s.play(snd, vol * g);
        }
    }

    /// Sound effects volume, 0..1.
    pub fn sfx_gain(&self) -> f32 {
        self.set.sfx as f32 / 10.0
    }

    /// On-screen button size, 0.7..1.5.
    pub fn btn_scale(&self) -> f32 {
        self.set.btn_scale as f32 / 100.0
    }

    /// Puts the volume settings on the sounds that are already playing.
    pub fn apply_volumes(&mut self) {
        if let Some(s) = self.sounds.as_ref() {
            if self.crowd_on {
                macroquad::audio::set_sound_volume(&s.crowd, 0.12 * self.sfx_gain());
            }
        }
        if let Some(m) = self.music.as_ref() {
            if self.music_on {
                macroquad::audio::set_sound_volume(m, self.music_volume());
            }
        }
    }

    fn music_volume(&self) -> f32 {
        self.set.music as f32 / 10.0 * 0.55
    }

    /// Menu music: plays outside matches when its volume is above zero.
    fn update_music(&mut self) {
        let want = !self.in_match() && self.set.music > 0;
        if let Some(m) = self.music.as_ref() {
            if want && !self.music_on {
                macroquad::audio::play_sound(m, macroquad::audio::PlaySoundParams { looped: true, volume: self.music_volume() });
                self.music_on = true;
            } else if !want && self.music_on {
                macroquad::audio::stop_sound(m);
                self.music_on = false;
            }
        }
    }

    /// Crowd murmur while a match is on.
    fn crowd(&mut self, want: bool) {
        let want = want && self.set.sfx > 0;
        let g = self.sfx_gain();
        if let Some(s) = self.sounds.as_ref() {
            if want && !self.crowd_on {
                s.play_loop(&s.crowd, 0.12 * g);
                self.crowd_on = true;
            } else if !want && self.crowd_on {
                s.stop(&s.crowd);
                self.crowd_on = false;
            }
        }
    }

    fn burst(&mut self, x: f32, y: f32, n: usize, c: Color) {
        for i in 0..n {
            let a = i as f32 / n.max(1) as f32 * 6.283 + rand::gen_range(0.0, 0.6);
            let sp = rand::gen_range(120.0, 380.0);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 100.0,
                life: 0.0,
                max: rand::gen_range(0.3, 0.7),
                r: rand::gen_range(2.5, 5.5),
                c,
                grav: 600.0,
                behind: false,
            });
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
        self.detect_fx(ev);
        // timers & spring animations
        self.shake = (self.shake - dt).max(0.0);
        if self.shake <= 0.0 {
            self.shake_mag = 0.0;
        }
        self.goal_anim += dt;
        self.ball_squash = (self.ball_squash - dt).max(0.0);
        self.ice_flash = (self.ice_flash - dt).max(0.0);
        if self.sim.ball.big > 0.0 {
            self.big_anim = (self.big_anim + dt).min(1.0);
        }
        self.big_shrink = (self.big_shrink - dt).max(0.0);
        for i in 0..2 {
            self.ice_shiver[i] = (self.ice_shiver[i] - dt).max(0.0);
            self.ice_pop[i] = (self.ice_pop[i] + dt / 0.12).min(1.0);
            self.post_rattle[i] = (self.post_rattle[i] - dt).max(0.0);
            // Underdamped net bulge spring
            let acc = -150.0 * self.net_bulge[i] - 6.5 * self.net_bulge_vel[i];
            self.net_bulge_vel[i] += acc * dt;
            self.net_bulge[i] += self.net_bulge_vel[i] * dt;
            if self.net_bulge[i].abs() < 0.002 && self.net_bulge_vel[i].abs() < 0.01 {
                self.net_bulge[i] = 0.0;
                self.net_bulge_vel[i] = 0.0;
            }
        }
        for p in self.particles.iter_mut() {
            p.life += dt;
            p.vy += p.grav * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.life < p.max);
        for f in self.fx.iter_mut() {
            f.life += dt;
        }
        self.fx.retain(|f| f.life < f.max);
        for sh in self.shards.iter_mut() {
            sh.life += dt;
            sh.vy += 1100.0 * dt;
            sh.x += sh.vx * dt;
            sh.y += sh.vy * dt;
            sh.rot += sh.vrot * dt;
        }
        self.shards.retain(|sh| sh.life < sh.max);
        for ib in self.ice_beams.iter_mut() {
            ib.life += dt;
        }
        self.ice_beams.retain(|ib| ib.life < ib.max);
        for r in self.rings.iter_mut() {
            r.life += dt;
        }
        self.rings.retain(|r| r.life < r.max);
        for sc in self.scorches.iter_mut() {
            sc.life += dt;
        }
        self.scorches.retain(|sc| sc.life < sc.max);
        let t_now = self.now as f32;
        for cf in self.confetti.iter_mut() {
            cf.life += dt;
            cf.vy += 190.0 * dt;
            cf.vx *= 1.0 - 1.6 * dt;
            cf.x += (cf.vx + (t_now * 5.5 + cf.phase).sin() * 55.0) * dt;
            cf.y += cf.vy * dt;
            cf.rot += cf.vrot * dt;
        }
        self.confetti.retain(|cf| cf.life < cf.max && cf.y < WORLD_BOTTOM + 60.0);

        // Continuous power & celebration emitters
        let b = self.sim.ball;
        if b.fire > 0.0 {
            let sp = (b.vx * b.vx + b.vy * b.vy).sqrt().max(1.0);
            let (dx, dy) = (-b.vx / sp, -b.vy / sp);
            for k in 0..2 {
                let c = if k == 0 { hex(0xffb420) } else { hex(0xff4a10) };
                self.particles.push(Particle {
                    x: b.x + dx * rand::gen_range(8.0, 26.0) + rand::gen_range(-10.0, 10.0),
                    y: b.y + dy * rand::gen_range(8.0, 26.0) + rand::gen_range(-10.0, 10.0),
                    vx: dx * rand::gen_range(80.0, 220.0) + rand::gen_range(-40.0, 40.0),
                    vy: dy * rand::gen_range(80.0, 220.0) + rand::gen_range(-40.0, 40.0),
                    life: 0.0,
                    max: rand::gen_range(0.18, 0.34),
                    r: rand::gen_range(6.0, 14.0),
                    c,
                    grav: -80.0,
                    behind: true,
                });
            }
        }
        // Drifting snowflakes & frosty mist around any frozen player
        for i in 0..2 {
            let p = self.sim.p[i];
            if p.frozen > 0.0 && rand::gen_range(0.0, 1.0) < 0.65 {
                let mist = rand::gen_range(0, 3) == 0;
                self.particles.push(Particle {
                    x: p.x + rand::gen_range(-78.0, 78.0),
                    y: p.y + rand::gen_range(-90.0, 68.0),
                    vx: rand::gen_range(-24.0, 24.0),
                    vy: if mist { rand::gen_range(-28.0, -8.0) } else { rand::gen_range(12.0, 46.0) },
                    life: 0.0,
                    max: rand::gen_range(0.45, 0.90),
                    r: if mist { rand::gen_range(7.0, 13.0) } else { rand::gen_range(2.2, 4.4) },
                    c: if mist { Color::new(0.75, 0.94, 1.0, 0.45) } else { Color::new(0.92, 0.98, 1.0, 0.95) },
                    grav: if mist { -15.0 } else { 18.0 },
                    behind: false,
                });
            }
        }
        if b.bomb > 0.0 {
            let rate = if b.bomb < 0.9 { 0.9 } else { 0.55 };
            if rand::gen_range(0.0, 1.0) < rate {
                self.particles.push(Particle {
                    x: b.x + rand::gen_range(-6.0, 6.0),
                    y: b.y - b.r() * 1.05,
                    vx: rand::gen_range(-65.0, 65.0),
                    vy: rand::gen_range(-190.0, -90.0),
                    life: 0.0,
                    max: rand::gen_range(0.18, 0.34),
                    r: rand::gen_range(2.5, 5.0),
                    c: if rand::gen_range(0, 2) == 0 { hex(0xffe14a) } else { hex(0xff6a20) },
                    grav: 360.0,
                    behind: false,
                });
            }
        }
        if self.sim.phase == Phase::Goal && rand::gen_range(0.0, 1.0) < 0.7 {
            self.confetti.push(ConfettiPiece {
                x: rand::gen_range(50.0, W - 50.0),
                y: rand::gen_range(-12.0, 24.0),
                vx: rand::gen_range(-50.0, 50.0),
                vy: rand::gen_range(90.0, 200.0),
                rot: rand::gen_range(0.0, std::f32::consts::TAU),
                vrot: rand::gen_range(-8.0, 8.0),
                phase: rand::gen_range(0.0, std::f32::consts::TAU),
                piece: rand::gen_range(0, 4) as u8,
                size: rand::gen_range(16.0, 24.0),
                life: 0.0,
                max: rand::gen_range(1.6, 2.4),
            });
        }
    }

    // ------------------------------------------------------------ screenshot scenes (--shots)

    /// Puts the app into a known state for `--shots` (menus, a goal, each power...).
    fn scene_setup(&mut self, name: &str) {
        self.paused = false;
        self.over_t = 0.0;
        self.ai_both = false;
        self.clear_vfx();
        self.touch_ui = name.contains("touch");
        self.set.btn_pos = [[0; 2]; 6];
        if name == "play_touch_moved" {
            self.set.btn_pos[Btn::Kick.slot()] = [560, 640];
            self.set.btn_pos[Btn::Power.slot()] = [680, 905];
        }
        self.tour = match name {
            "countdown_tour" => Some(Tour { round: 1 }),
            "champion" => Some(Tour { round: 2 }),
            "eliminated" => Some(Tour { round: 1 }),
            _ => None,
        };
        match name {
            "menu" => self.screen = Screen::Menu,
            "multi" => self.screen = Screen::Multi,
            "setup_cpu" | "setup_cup" | "setup_local2" | "setup_lan_host" | "setup_online_host" | "setup_online_join" => {
                self.setup = match name {
                    "setup_cpu" => SetupKind::VsCpu,
                    "setup_cup" => SetupKind::Cup,
                    "setup_local2" => SetupKind::Local2,
                    "setup_lan_host" => SetupKind::LanHost,
                    "setup_online_host" => SetupKind::OnlineHost,
                    _ => SetupKind::OnlineJoin,
                };
                self.screen = Screen::Setup;
            }
            "settings" => {
                self.settings_back = Screen::Menu;
                self.screen = Screen::Settings;
            }
            "button_test" => self.screen = Screen::ButtonTest,
            "online_join" => {
                self.code_text = "4829".into();
                self.screen = Screen::OnlineJoin;
            }
            "online_host" => {
                self.room_code = "482913".into();
                self.screen = Screen::OnlineHost;
            }
            "host_lobby" => {
                self.my_ip = "192.168.0.12".into();
                self.screen = Screen::HostLobby;
            }
            "join_lobby" => {
                self.ip_text = "192.168.0.12".into();
                self.screen = Screen::JoinLobby;
            }
            _ => {
                let mode = if name.starts_with("play_local2") { Mode::Local2 } else { Mode::Cpu };
                self.set.power = PK_FIRE;
                self.set.level = 2;
                self.start_match(mode);
                self.ai_both = name == "play_cpu" || name.starts_with("play_touch");
                self.sim.phase = Phase::Play;
                self.sim.time_left = 74.0;
                self.sim.score = [1, 1];
                self.sim.p[0].power = 0.65;
                self.sim.p[1].power = if name.starts_with("pw_") { 0.55 } else { 1.0 };
                match name {
                    "goal" => {
                        self.sim.score = [2, 1];
                        self.sim.phase = Phase::Goal;
                        self.sim.phase_t = GOAL_PAUSE;
                        self.sim.last_scorer = 0;
                        self.sim.ball = Ball { x: W - 46.0, y: 452.0, vx: 300.0, vy: 40.0, ..Default::default() };
                        self.goal_anim = 0.0;
                        self.net_bulge_vel[1] = 18.0;
                        self.net_hit_y[1] = 438.0;
                        self.spawn_goal_confetti(W - 64.0, 420.0, 56);
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
                        self.sim.ball = Ball { x: 560.0, y: 395.0, vx: 1200.0, vy: -90.0, fire: 1.1, fire_owner: 0, ..Default::default() };
                    }
                    "pw_big" => {
                        self.sim.p[0].pkind = PK_BIG;
                        self.sim.ball = Ball { x: 640.0, y: 390.0, big: 7.0, ..Default::default() };
                        self.prev_big = 7.0;
                        self.big_anim = 0.14;
                        self.rings.push(ShockRing { x: 640.0, y: 390.0, r0: BALL_R, r1: 155.0, thick: 9.0, life: 0.0, max: 0.45, col: hex(0xffd34d) });
                    }
                    "pw_ice" => {
                        self.sim.p[0].pkind = PK_ICE;
                        self.sim.ball = Ball { x: 540.0, y: 400.0, vx: 180.0, vy: -120.0, ..Default::default() };
                        // Showcase reworked HB2 Freeze: cracked ice block + frost decal + frost beam + snowflakes
                        self.sim.p[1].frozen = FREEZE_TIME * 0.38;
                        self.sim.p[1].stun = 0.0;
                        self.prev_frozen[1] = self.sim.p[1].frozen;
                        self.ice_pop[1] = 1.0;
                        self.ice_shiver[1] = 0.14;
                        self.ice_beams.push(IceBeam {
                            from_x: self.sim.p[0].x,
                            from_y: self.sim.p[0].y - 6.0,
                            to_x: self.sim.p[1].x,
                            to_y: self.sim.p[1].y + 4.0,
                            life: 0.0,
                            max: 0.50,
                        });
                    }
                    "pw_invis" => {
                        self.sim.p[0].pkind = PK_INVIS;
                        self.sim.ball = Ball { x: 640.0, y: 390.0, invis: 4.0, invis_owner: 0, ..Default::default() };
                        self.prev_invis = 4.0;
                        self.rings.push(ShockRing { x: 640.0, y: 390.0, r0: 20.0, r1: 115.0, thick: 7.0, life: 0.0, max: 0.42, col: hex(0xa8f2ff) });
                    }
                    "pw_bomb" => {
                        self.sim.p[0].pkind = PK_BOMB;
                        self.sim.ball = Ball { x: 640.0, y: 395.0, bomb: 0.78, bomb_owner: 0, ..Default::default() };
                    }
                    "pw_clone" => {
                        self.sim.p[0].pkind = PK_CLONE;
                        let mut b = Ball { x: 560.0, y: 395.0, vx: 650.0, vy: -240.0, clone_t: 4.0, ..Default::default() };
                        b.clones = [[620.0, 295.0, 620.0, -340.0], [600.0, 445.0, 740.0, 90.0]];
                        self.sim.ball = b;
                        self.prev_clone = 4.0;
                        self.prev_clones = [[620.0, 295.0], [600.0, 445.0]];
                    }
                    _ => {}
                }
            }
        }
    }

    /// One frame of a `--shots` scene: no real input, fixed time step.
    fn scene_frame(&mut self, dt: f32) {
        self.now += dt as f64;
        if self.screen == Screen::Match {
            set_view_top();
        }
        if self.screen == Screen::Match {
            self.update_match(dt, [Input::default(); 2]);
            self.draw_match();
            self.match_overlays();
        } else {
            self.screen_frame(dt);
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
        if self.screen == Screen::Match {
            set_view_top();
        } else {
            set_view();
        }
        clear_background(hex(0x0b1022));
        self.crowd(self.in_match());
        self.update_music();
        self.screen_frame(dt);
    }

    /// A match is on (also while its settings are open from the pause box).
    fn in_match(&self) -> bool {
        self.screen == Screen::Match || (matches!(self.screen, Screen::Settings | Screen::ButtonTest) && self.settings_back == Screen::Match)
    }

    /// Draws and runs the current screen.
    fn screen_frame(&mut self, dt: f32) {
        match self.screen {
            Screen::Menu => self.menu(),
            Screen::Multi => self.multi(),
            Screen::Setup => self.setup_screen(),
            Screen::Settings => self.settings_screen(),
            Screen::ButtonTest => self.button_test(),
            Screen::HostLobby => self.host_lobby(),
            Screen::JoinLobby => self.join_lobby(),
            Screen::OnlineHost => self.online_host_lobby(),
            Screen::OnlineJoin => self.online_join(),
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
        draw_cover(&self.assets.menu_bg, WHITE);
        dim_screen(with_a(hex(0x0a0f24), 0.42));
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

    fn host_lobby(&mut self) {
        if is_key_pressed(KeyCode::Back) || is_key_pressed(KeyCode::Escape) {
            self.leave_network();
            self.screen = Screen::Multi;
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
            self.leave_network();
            self.screen = Screen::Multi;
        }
    }

    fn join_lobby(&mut self) {
        if is_key_pressed(KeyCode::Back) || is_key_pressed(KeyCode::Escape) {
            self.leave_network();
            self.screen = Screen::Multi;
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
            self.leave_network();
            self.screen = Screen::Multi;
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
        let (sl, st, _, _) = screen_box();
        let pause_hit = ui::round_button(self, sl + 48.0, st + 48.0, 32.0, "icon_pause", |x, y, r| {
            rectv(x - r * 0.32, y - r * 0.4, r * 0.22, r * 0.8, WHITE);
            rectv(x + r * 0.1, y - r * 0.4, r * 0.22, r * 0.8, WHITE);
        });
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
            dim_screen(with_a(BLACK, 0.6));
            let msg = if self.mode == Mode::Host { "O outro jogador saiu ou perdeu a conexão" } else { "Conexão com a partida perdida" };
            text_o(msg, 640.0, 300.0, 40.0, WHITE);
            if self.mode == Mode::Host {
                if self.button("Esperar de novo", 340.0, 380.0, 280.0, 64.0, hex(0x2f8f4e)) {
                    if let Some(h) = self.host.as_mut() {
                        h.peer = None;
                    }
                    self.lost = false;
                    let online = self.host.as_ref().and_then(|h| h.link()).is_some();
                    self.screen = if online { Screen::OnlineHost } else { Screen::HostLobby };
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
            dim_screen(with_a(BLACK, 0.6));
            text_o("Pausa", 640.0, 250.0, 64.0, WHITE);
            if matches!(self.mode, Mode::Host | Mode::Client) {
                text_o("(a partida em rede continua rodando)", 640.0, 290.0, 24.0, hex(0xb8c6e6));
            }
            if self.button("Continuar", 490.0, 330.0, 300.0, 64.0, hex(0x2f8f4e)) || is_key_pressed(KeyCode::Escape) || back {
                self.paused = false;
            }
            let local = matches!(self.mode, Mode::Cpu | Mode::Local2);
            if local && self.button("Configurações", 490.0, 415.0, 300.0, 64.0, hex(0x34405e)) {
                self.open_settings(Screen::Match);
                return;
            }
            if self.button("Sair para o menu", 490.0, if local { 500.0 } else { 415.0 }, 300.0, 64.0, hex(0x5a2d3a)) {
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
                dim_screen(with_a(BLACK, 0.55));
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
            dim_screen(with_a(BLACK, 0.55));
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
        draw_cover(&self.assets.menu_bg, WHITE);
        dim_screen(with_a(hex(0x080d1e), 0.18));
        let t = self.now as f32;
        let c0 = self.sim.p[0].look as usize % 6;
        // 3-stage cup bracket ribbon
        rrect(310.0, 224.0, 660.0, 44.0, 22.0, with_a(BLACK, 0.55));
        text_o("Quartas  OK        Semifinal  OK        Final  OK", 640.0, 254.0, 22.0, hex(0xffe14a));
        // Winner + Trophy on the podium (drawn with the background's own crop)
        with_view(cover_view(), || {
            let bob = (t * 3.0).sin() * 5.0;
            ellipse(590.0, 548.0, 82.0, 16.0, with_a(BLACK, 0.32));
            ellipse(740.0, 548.0, 60.0, 14.0, with_a(BLACK, 0.30));
            draw_char_jersey(c0, 590.0, 476.0 + bob * 0.45, 1.32, false, 0.0, WHITE);
            tex_center(&self.assets.boots[c0], 596.0, 526.0, 96.0, 64.0, WHITE, false, -0.04);
            tex_center(&self.assets.heads[c0], 590.0, 426.0 + bob, 186.0, 186.0, WHITE, false, (t * 1.6).sin() * 0.05);
            tex_center(&self.assets.trophy, 742.0, 458.0 + (t * 2.4 + 1.0).sin() * 4.0, 175.0, 175.0, WHITE, false, (t * 1.2).sin() * 0.04);
        });
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
        let bg = &self.assets.stadium_bgs[self.sim.stadium as usize % self.assets.stadium_bgs.len()];
        tex_draw(bg, 0.0, 0.0, W, H, WHITE, false, 0.0);
        // Goal / match-over celebratory floodlight beams + subtle crowd flash
        if live && (self.sim.phase == Phase::Goal || self.sim.phase == Phase::Over) {
            let t = self.now as f32;
            let pulse = 0.04 + 0.05 * (t * 9.0).sin().abs();
            rectv(0.0, 0.0, W, WORLD_BOTTOM, with_a(hex(0xffe14a), pulse));
            for (i, lx) in [220.0f32, 1060.0].iter().enumerate() {
                let dir = if i == 0 { 1.0 } else { -1.0 };
                let sweep = (t * 3.8 + i as f32 * 2.1).sin() * 190.0;
                let tx = 640.0 + sweep + dir * 80.0;
                tri((*lx, 0.0), (tx - 115.0, WORLD_BOTTOM), (tx + 115.0, WORLD_BOTTOM), with_a(hex(0xfff4b0), 0.075));
            }
        }
    }

    fn draw_scorch_marks(&self) {
        for sc in &self.scorches {
            let k = 1.0 - (sc.life / sc.max).clamp(0.0, 1.0);
            let a = (k * 1.4).min(1.0);
            ellipse(sc.x, GROUND + 1.0, 62.0, 14.0, with_a(hex(0x161210), 0.42 * a));
            ellipse(sc.x, GROUND + 1.0, 38.0, 8.5, with_a(BLACK, 0.50 * a));
            if sc.life < 0.7 {
                let glow = 1.0 - sc.life / 0.7;
                ellipse(sc.x, GROUND + 1.0, 26.0, 5.5, with_a(hex(0xff6a18), 0.45 * glow));
            }
        }
    }

    /// Back layer of the 2.5D goals (far post + back net, drawn BEHIND players and ball)
    /// with spring-damped multi-strip net bulge distortion on goals/net hits.
    fn draw_goal_back(&self) {
        let Some(tex) = self.assets.ui("goal_back") else { return };
        const GX: f32 = -8.0;
        const GY: f32 = 296.0;
        const GW: f32 = 170.0;
        const GH: f32 = 277.0;
        let (tw, th) = (tex.width(), tex.height());
        for gi in 0..2 {
            let flip = gi == 1;
            let gx = if flip { W - GW - GX } else { GX };
            let bulge = self.net_bulge[gi];
            if bulge.abs() < 0.004 {
                tex_draw(tex, gx, GY, GW, GH, WHITE, flip, 0.0);
                continue;
            }
            let strips = 12usize;
            let hit_v = ((self.net_hit_y[gi] - GY) / GH).clamp(0.15, 0.85);
            for s_i in 0..strips {
                let v0 = s_i as f32 / strips as f32;
                let v1 = (s_i + 1) as f32 / strips as f32;
                let vm = (v0 + v1) * 0.5;
                // Keep top crossbar (v=0) and ground base (v=1) pinned
                let pin = (vm * std::f32::consts::PI).sin();
                let focus = (-((vm - hit_v) / 0.34).powi(2)).exp();
                let w = pin * (0.45 + 0.55 * focus);
                let ripple = (bulge * 2.4 + vm * 10.0).sin() * bulge.abs() * 4.5 * w;
                let disp = (bulge * w * 18.0 + ripple).clamp(-10.0, 28.0);
                let sy = v0 * th;
                let sh = (v1 - v0) * th;
                let dy = GY + v0 * GH;
                let dh = (v1 - v0) * GH + 0.5;
                let (dx, dw) = if !flip { (gx - disp, GW + disp) } else { (gx, GW + disp) };
                tex_sub(tex, 0.0, sy, tw, sh, dx, dy, dw, dh, WHITE, flip, 0.0);
            }
        }
    }

    /// Front layer of the 2.5D goals (front post + crossbar + front rim, drawn IN FRONT of
    /// players and ball). Falls back cleanly to `goal_left` when `goal_front` is absent.
    fn draw_goal_front(&self) {
        let tex = self.assets.ui("goal_front").unwrap_or(&self.assets.goal_left);
        const GX: f32 = -8.0;
        const GY: f32 = 296.0;
        const GW: f32 = 170.0;
        const GH: f32 = 277.0;
        let t = self.now as f32;
        for gi in 0..2 {
            let flip = gi == 1;
            let gx = if flip { W - GW - GX } else { GX };
            let r = self.post_rattle[gi];
            let rx = if r > 0.0 { (t * 95.0).sin() * 2.2 * (r / 0.24) } else { 0.0 };
            let ry = if r > 0.0 { (t * 118.0).cos() * 1.4 * (r / 0.24) } else { 0.0 };
            tex_draw(tex, gx + rx, GY + ry, GW, GH, WHITE, flip, 0.0);
        }
    }

    /// Frost/snow crystal decal on the grass beneath any frozen player.
    fn draw_ground_frost(&self) {
        for i in 0..2 {
            let p = self.sim.p[i];
            if p.frozen <= 0.0 {
                continue;
            }
            let lift = (GROUND - (p.y + FEET)).max(0.0);
            let k = (1.0 - lift / 280.0).clamp(0.28, 1.0);
            if let Some(tex) = self.assets.ui("ice_frost") {
                tex_center(tex, p.x, GROUND - 2.0, 214.0 * k, 80.0 * k, with_a(WHITE, 0.92 * k), false, 0.0);
            } else {
                // Crisp crystalline frost patch fallback
                ellipse(p.x, GROUND + 1.0, 96.0 * k, 24.0 * k, with_a(hex(0x6ad8ff), 0.36 * k));
                ellipse(p.x, GROUND + 1.0, 74.0 * k, 16.0 * k, with_a(hex(0xcaf6ff), 0.55 * k));
                for sp in 0..6 {
                    let a = sp as f32 * std::f32::consts::TAU / 6.0;
                    let tx = p.x + a.cos() * 102.0 * k;
                    let ty = GROUND + 1.0 + a.sin() * 24.0 * k;
                    let b1 = a + 0.22;
                    let b2 = a - 0.22;
                    tri(
                        (tx, ty),
                        (p.x + b1.cos() * 58.0 * k, GROUND + 1.0 + b1.sin() * 14.0 * k),
                        (p.x + b2.cos() * 58.0 * k, GROUND + 1.0 + b2.sin() * 14.0 * k),
                        with_a(hex(0xdcf9ff), 0.62 * k),
                    );
                }
            }
        }
    }

    fn draw_ice_cracks_fallback(&self, ix: f32, iy: f32, iw: f32, ih: f32, stage2: bool) {
        let c1 = with_a(WHITE, 0.90);
        let c2 = with_a(hex(0xa8ecff), 0.72);
        // Stage 1 sharp diagonal fracture lines
        linev(ix + iw * 0.22, iy + ih * 0.18, ix + iw * 0.52, iy + ih * 0.44, 3.0, c1);
        linev(ix + iw * 0.52, iy + ih * 0.44, ix + iw * 0.36, iy + ih * 0.68, 2.5, c2);
        linev(ix + iw * 0.78, iy + ih * 0.28, ix + iw * 0.56, iy + ih * 0.50, 2.6, c1);
        linev(ix + iw * 0.56, iy + ih * 0.50, ix + iw * 0.74, iy + ih * 0.76, 2.2, c2);
        if stage2 {
            // Stage 2 dense spiderweb cracks across the block
            linev(ix + iw * 0.52, iy + ih * 0.44, ix + iw * 0.16, iy + ih * 0.54, 2.5, c1);
            linev(ix + iw * 0.52, iy + ih * 0.44, ix + iw * 0.84, iy + ih * 0.62, 2.5, c1);
            linev(ix + iw * 0.36, iy + ih * 0.68, ix + iw * 0.62, iy + ih * 0.86, 2.4, c1);
            linev(ix + iw * 0.44, iy + ih * 0.12, ix + iw * 0.52, iy + ih * 0.44, 2.2, c2);
            linev(ix + iw * 0.24, iy + ih * 0.82, ix + iw * 0.48, iy + ih * 0.60, 2.4, c1);
        }
    }

    fn draw_player(&self, i: usize) {
        let p = self.sim.p[i];
        let s = p.side;
        let flip = s < 0.0;
        let cid = self.sim.p[i].look as usize % 6;
        let head_tex = &self.assets.heads[cid];
        let boot_tex = &self.assets.boots[cid];
        let frozen = p.frozen > 0.0;
        let frac = if frozen { (p.frozen / FREEZE_TIME).clamp(0.0, 1.0) } else { 1.0 };
        // Progressive shiver shake when frozen (plus sharp mash jolt on EV_ICE_CRACK)
        let (sh_x, sh_y) = if frozen {
            let stage_amp = if frac < 0.33 {
                3.4
            } else if frac < 0.66 {
                1.4
            } else {
                0.0
            };
            let mash_amp = (self.ice_shiver[i] / 0.16).clamp(0.0, 1.0) * 6.5;
            let t = self.now as f32;
            ((t * 64.0 + i as f32 * 2.1).sin() * (stage_amp + mash_amp), (t * 48.0).cos() * (mash_amp * 0.35))
        } else {
            (0.0, 0.0)
        };
        let x = p.x + sh_x;
        let y = p.y + sh_y;
        // Dynamic ground shadow shrinking & softening smoothly with altitude
        let lift = (GROUND - (p.y + FEET)).max(0.0);
        let k = (1.0 - lift / 360.0).clamp(0.22, 1.0);
        ellipse(p.x, GROUND + 2.0, 56.0 * k, 13.0 * k, with_a(BLACK, 0.20 * k * k));
        ellipse(p.x, GROUND + 2.0, 42.0 * k, 9.0 * k, with_a(BLACK, 0.28 * k * k));
        // Power-ready aura
        if p.power_ready > 0.0 {
            let pulse = (self.now as f32 * 12.0).sin() * 5.0;
            circ(x, y + 6.0, HEAD_R + 30.0 + pulse, with_a(hex(0xff8a1a), 0.24));
            circ(x, y + 6.0, HEAD_R + 17.0 + pulse, with_a(hex(0xffe14a), 0.30));
        }
        let body_tint = if frozen {
            Color::new(0.64, 0.88, 1.0, 1.0)
        } else if p.stun > 0.0 {
            Color::new(1.0, 0.82, 0.82, 1.0)
        } else if p.power_ready > 0.0 {
            Color::new(1.0, 0.96, 0.84, 1.0)
        } else {
            WHITE
        };
        let bob = if !frozen && p.on_ground && p.vx.abs() > 10.0 {
            -(p.walk * 2.0).sin().abs() * 4.0
        } else {
            0.0
        };
        // Back boot (72x48, sole aligned flush with GROUND = 560.0)
        let walk = if frozen { 0.0 } else { p.walk.sin() * 9.0 };
        let back_x = x - s * 16.0 - walk * s;
        let back_y = y + FEET - 21.6;
        let back_rot = if frozen {
            0.0
        } else if p.on_ground {
            -walk * 0.025 * s
        } else {
            0.16 * s
        };
        let back_tint = tint_col(Color::new(0.75, 0.75, 0.82, 1.0), body_tint);
        tex_center(boot_tex, back_x, back_y, 72.0, 48.0, back_tint, flip, back_rot);
        // Stylized team-colored mini-jersey & shorts connecting head to boots
        let lean = if frozen { 0.0 } else { (p.vx / MOVE_SPEED) * 4.0 };
        draw_char_jersey(cid, x, y + 30.0 + bob * 0.5, 1.0, flip, lean, body_tint);
        // Front kicking boot (84x56, sole aligned flush on GROUND = 560.0 when standing)
        let (fx, fy) = p.foot();
        let kick_rot = if frozen {
            0.0
        } else if p.kick_t >= 0.0 {
            let kk = (p.kick_t / KICK_TIME).clamp(0.0, 1.0);
            let swing = (kk * std::f32::consts::PI).sin();
            if p.kick_kind == 2 {
                -s * 0.74 * swing
            } else {
                -s * 0.40 * swing
            }
        } else if p.on_ground {
            walk * 0.024 * s
        } else {
            -0.12 * s
        };
        tex_center(boot_tex, fx + sh_x - s * 4.0, fy + sh_y - 4.4, 84.0, 56.0, body_tint, flip, kick_rot);
        // Big caricature head (~132x132 centered cleanly above jersey collar)
        let head_rot = if frozen {
            0.0
        } else if p.stun > 0.0 {
            (self.now as f32 * 18.0).sin() * 0.24
        } else {
            (p.vx / MOVE_SPEED) * 0.08
        };
        tex_center(head_tex, x, y - 6.0 + bob, 132.0, 132.0, body_tint, flip, head_rot);
        // Faceted ice block + 3-stage cracks when frozen
        if frozen {
            let pop = self.ice_pop[i].clamp(0.0, 1.0);
            let pop_s = 0.85 + 0.15 * (1.0 - (1.0 - pop).powi(3));
            let iw = 176.0 * pop_s;
            let ih = 208.0 * pop_s;
            // Flat base of ice_block.png sits at v = 0.962 -> rests flush on GROUND when grounded,
            // or wraps squarely around feet when frozen mid-air!
            let feet_y = if p.y + FEET >= GROUND - 6.0 { GROUND } else { p.y + FEET };
            let ix = x - iw * 0.5;
            let iy = feet_y + sh_y - ih * 0.962;
            tex_draw(&self.assets.ice_block, ix, iy, iw, ih, WHITE, false, 0.0);
            if frac < 0.66 {
                if let Some(c1) = self.assets.ui("ice_crack1") {
                    tex_draw(c1, ix, iy, iw, ih, WHITE, false, 0.0);
                } else {
                    self.draw_ice_cracks_fallback(ix, iy, iw, ih, frac < 0.33);
                }
            }
            if frac < 0.33 {
                if let Some(c2) = self.assets.ui("ice_crack2") {
                    tex_draw(c2, ix, iy, iw, ih, WHITE, false, 0.0);
                } else if self.assets.ui("ice_crack1").is_some() {
                    self.draw_ice_cracks_fallback(ix, iy, iw, ih, true);
                }
            }
        }
        // Crisp 3D-orbiting dizzy stars when stunned (separate from freeze)
        if p.stun > 0.0 && !frozen {
            let t = self.now as f32 * 6.8;
            let cy = y - HEAD_R - 14.0;
            draw_ellipse_lines(px(x), py(cy), sc(48.0), sc(12.0), 0.0, sc(2.0), with_a(hex(0xffe14a), 0.32));
            let mut stars = [0usize, 1, 2];
            stars.sort_by(|&a, &b| {
                let sa = (t + a as f32 * 2.094).sin();
                let sb = (t + b as f32 * 2.094).sin();
                sa.total_cmp(&sb)
            });
            for j in stars {
                let a = t + j as f32 * 2.094;
                let depth = (a.sin() + 1.0) * 0.5; // 0 = back, 1 = front
                let sx = x + a.cos() * 48.0;
                let sy = cy + a.sin() * 12.0;
                let sr = 7.5 + 4.0 * depth;
                let alpha = 0.72 + 0.28 * depth;
                star_outlined(sx, sy, sr, with_a(hex(0xffe14a), alpha));
            }
        }
    }

    /// Visual ball radius with Giant Ball spring inflation/deflation curve and final-1.2s warning pulse.
    fn ball_visual_r(&self) -> f32 {
        let b = self.sim.ball;
        if b.big > 0.0 {
            let k = (self.big_anim / 0.30).clamp(0.0, 1.0);
            // Overshoots to ~2.00 then settles cleanly to BIG_SCALE (1.85)
            let osc = (-3.0 * k).exp() * (k * std::f32::consts::TAU * 1.15).cos() * (1.0 - k);
            let mut scale = 1.0 + (BIG_SCALE - 1.0) * (1.0 - osc);
            if b.big < 1.2 {
                let warn = ((1.2 - b.big) / 1.2).clamp(0.0, 1.0);
                scale *= 1.0 + (0.045 + 0.04 * warn) * ((1.2 - b.big) * 18.0).sin();
            }
            BALL_R * scale
        } else if self.big_shrink > 0.0 {
            let k = (1.0 - self.big_shrink / 0.24).clamp(0.0, 1.0);
            let osc = (-4.5 * k).exp() * (k * std::f32::consts::TAU * 0.9).cos() * (1.0 - k);
            let scale = 1.0 + (BIG_SCALE - 1.0) * osc;
            BALL_R * scale.max(0.88)
        } else {
            BALL_R
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
            if (t * 1.5).fract() < 0.08 { 0.16 } else { 0.0 }
        };
        base.max(back)
    }

    fn is_invis_owner_view(&self) -> bool {
        let b = self.sim.ball;
        if b.invis <= 0.0 {
            return false;
        }
        let owner = b.invis_owner as usize;
        let mine = self.local_players();
        let seen_by_owner_only = !(self.mode == Mode::Local2 || (self.mode == Mode::Cpu && self.ai_both));
        seen_by_owner_only && mine.contains(&owner)
    }

    fn draw_ball_shadow(&self, bx: f32, by: f32, r: f32, alpha: f32) {
        if alpha <= 0.01 {
            return;
        }
        let lift = (GROUND - (by + r)).max(0.0);
        let k = (1.0 - lift / 440.0).clamp(0.20, 1.0);
        let rx = r * 1.05 * k;
        let ry = r * 0.27 * k;
        ellipse(bx, GROUND + 2.0, rx * 1.22, ry * 1.22, with_a(BLACK, 0.16 * k * k * alpha));
        ellipse(bx, GROUND + 2.0, rx, ry, with_a(BLACK, 0.30 * k * k * alpha));
    }

    fn draw_ball(&self) {
        let b = self.sim.ball;
        let r = self.ball_visual_r();
        let alpha = self.ball_alpha();
        let t = self.now as f32;
        // Squash & stretch on bounces/kicks (~1.15 x 0.87 for 75 ms)
        let sq = (self.ball_squash / 0.075).clamp(0.0, 1.0);
        let (sq_x, sq_y) = if self.ball_squash_vert {
            (1.0 - 0.13 * sq, 1.0 + 0.15 * sq)
        } else {
            (1.0 + 0.15 * sq, 1.0 - 0.13 * sq)
        };
        // Real ball shadow
        self.draw_ball_shadow(b.x, b.y, r, alpha);
        // Fake clone balls: draw MATCHING ground shadows and respect `big` scale & `alpha` so there is zero tell!
        if b.clone_t > 0.0 {
            let cd = r * 2.15;
            for c in b.clones {
                self.draw_ball_shadow(c[0], c[1], r, alpha);
                tex_center(&self.assets.ball, c[0], c[1], cd, cd, Color::new(1.0, 1.0, 1.0, alpha), false, b.angle);
            }
        }
        // Giant Ball golden rim glow (pulses faster in final 1.2 s)
        if b.big > 0.0 && alpha > 0.05 {
            let pulse = if b.big < 1.2 {
                0.28 + 0.22 * ((1.2 - b.big) * 18.0).sin().abs()
            } else {
                0.18 + 0.06 * (t * 6.0).sin()
            };
            circ(b.x, b.y, r + 9.0, with_a(hex(0xffd34d), pulse * alpha));
            circ_line(b.x, b.y, r + 4.0, 3.0, with_a(hex(0xfff0a0), (pulse + 0.18) * alpha));
        }
        if b.fire > 0.0 {
            // Directional tapered flame comet behind velocity vector
            let sp = (b.vx * b.vx + b.vy * b.vy).sqrt();
            if sp > 120.0 {
                let (dx, dy) = (-b.vx / sp, -b.vy / sp);
                let (px_n, py_n) = (-dy, dx);
                let flick = 1.0 + 0.14 * (t * 36.0).sin();
                let len = (r * 3.4 * flick).min(135.0);
                // Outer red-orange flame cone
                tri(
                    (b.x + dx * len, b.y + dy * len),
                    (b.x + px_n * r * 1.15, b.y + py_n * r * 1.15),
                    (b.x - px_n * r * 1.15, b.y - py_n * r * 1.15),
                    with_a(hex(0xff3d00), 0.48 * alpha),
                );
                // Mid orange tongue
                tri(
                    (b.x + dx * len * 0.72, b.y + dy * len * 0.72),
                    (b.x + px_n * r * 0.82, b.y + py_n * r * 0.82),
                    (b.x - px_n * r * 0.82, b.y - py_n * r * 0.82),
                    with_a(hex(0xff9414), 0.68 * alpha),
                );
                // Hot yellow-white core
                tri(
                    (b.x + dx * len * 0.42, b.y + dy * len * 0.42),
                    (b.x + px_n * r * 0.48, b.y + py_n * r * 0.48),
                    (b.x - px_n * r * 0.48, b.y - py_n * r * 0.48),
                    with_a(hex(0xfff4a8), 0.78 * alpha),
                );
            }
            // Layered roaring flame aura + fire ball sprite
            circ(b.x, b.y, r + 15.0 + (t * 28.0).sin() * 3.0, with_a(hex(0xff5200), 0.34 * alpha));
            circ(b.x, b.y, r + 8.0, with_a(hex(0xffb420), 0.42 * alpha));
            let d = r * 3.15;
            tex_center(&self.assets.ball_fire, b.x, b.y, d * sq_x, d * sq_y, Color::new(1.0, 1.0, 1.0, alpha), false, b.angle * 0.45);
        } else if b.ice > 0.0 {
            circ(b.x, b.y, r + 10.0, with_a(hex(0x9fe6ff), 0.45 * alpha));
            let d = r * 2.45;
            tex_center(&self.assets.ball_ice, b.x, b.y, d * sq_x, d * sq_y, Color::new(1.0, 1.0, 1.0, alpha), false, b.angle);
        } else if b.bomb > 0.0 {
            let urgent = b.bomb < 0.9;
            let rate = if urgent { 14.0 } else { 5.0 };
            let flash = if (b.bomb * rate).fract() < 0.5 { 1.0 } else { 0.0 };
            // Subtle pulsing danger radius ring under 0.9 s
            if urgent {
                let p_ring = 0.20 + 0.22 * (t * 18.0).sin().abs();
                circ(b.x, b.y, BOMB_RADIUS, with_a(hex(0xff3818), 0.05 * p_ring * alpha));
                circ_line(b.x, b.y, BOMB_RADIUS, 3.0, with_a(hex(0xff5a2a), p_ring * alpha));
                circ_line(b.x, b.y, BOMB_RADIUS * 0.85, 1.5, with_a(hex(0xffc040), p_ring * 0.55 * alpha));
            }
            let throb = 1.0 + if urgent { 0.12 * (t * 22.0).sin().abs() } else { 0.04 * (t * 10.0).sin().abs() };
            let d = r * 2.55 * throb;
            let tint = if flash > 0.5 {
                Color::new(1.0, if urgent { 0.92 } else { 1.0 }, if urgent { 0.82 } else { 1.0 }, alpha)
            } else {
                Color::new(1.0, 0.42, 0.36, alpha)
            };
            if urgent && flash > 0.5 {
                circ(b.x, b.y, r * 1.25, with_a(hex(0xff4220), 0.36 * alpha));
            }
            tex_center(&self.assets.ball_bomb, b.x, b.y - 3.0, d * sq_x, d * sq_y, tint, false, b.angle * 0.25);
        } else {
            let d = r * 2.15;
            tex_center(&self.assets.ball, b.x, b.y, d * sq_x, d * sq_y, Color::new(1.0, 1.0, 1.0, alpha), false, b.angle);
        }
        // Invisible ball ethereal ghost ring/shimmer for owner, subtle glint for opponent
        if b.invis > 0.0 {
            if self.is_invis_owner_view() {
                let glow = 0.45 + 0.25 * (t * 8.0).sin();
                circ_line(b.x, b.y, r + 4.0, 3.0, with_a(hex(0x88f0ff), glow));
                for j in 0..3 {
                    let a = t * 4.5 + j as f32 * 2.094;
                    let sx = b.x + a.cos() * (r + 7.0);
                    let sy = b.y + a.sin() * (r + 7.0);
                    circ(sx, sy, 3.2, with_a(WHITE, 0.85));
                }
            } else if alpha > 0.05 && alpha < 0.35 {
                // Brief subtle glint cross for opponent
                linev(b.x - 14.0, b.y, b.x + 14.0, b.y, 2.2, with_a(hex(0xc8f6ff), alpha * 2.5));
                linev(b.x, b.y - 14.0, b.x, b.y + 14.0, 2.2, with_a(hex(0xc8f6ff), alpha * 2.5));
            }
        }
    }

    fn draw_ice_beams(&self) {
        for ib in &self.ice_beams {
            let k = (ib.life / ib.max).clamp(0.0, 1.0);
            let a = (1.0 - k).powf(0.8);
            let dx = ib.to_x - ib.from_x;
            let dy = ib.to_y - ib.from_y;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let (nx, ny) = (-dy / len, dx / len);
            let segs = 7usize;
            let mut prev = (ib.from_x, ib.from_y);
            for s in 1..=segs {
                let u = s as f32 / segs as f32;
                let jitter = if s == segs {
                    0.0
                } else {
                    ((s as f32 * 3.7 + ib.life * 28.0).sin()) * 16.0 * (1.0 - k * 0.5)
                };
                let cur = (ib.from_x + dx * u + nx * jitter, ib.from_y + dy * u + ny * jitter);
                linev(prev.0, prev.1, cur.0, cur.1, 18.0 * a, with_a(hex(0x45d8ff), 0.32 * a));
                linev(prev.0, prev.1, cur.0, cur.1, 8.5 * a, with_a(hex(0xa8f2ff), 0.72 * a));
                linev(prev.0, prev.1, cur.0, cur.1, 3.5 * a, with_a(WHITE, 0.95 * a));
                prev = cur;
            }
            circ(ib.to_x, ib.to_y, 34.0 * (0.6 + 0.5 * k), with_a(hex(0x88eeff), 0.45 * a));
        }
    }

    fn draw_rings(&self) {
        for r in &self.rings {
            let k = (r.life / r.max).clamp(0.0, 1.0);
            let ease = 1.0 - (1.0 - k).powi(2);
            let rad = r.r0 + (r.r1 - r.r0) * ease;
            let a = (1.0 - k).powf(0.9);
            circ_line(r.x, r.y, rad, (r.thick * (1.0 - k * 0.6)).max(1.5), with_a(r.col, 0.85 * a));
        }
    }

    fn draw_ice_shards(&self) {
        for sh in &self.shards {
            let a = (1.0 - sh.life / sh.max).clamp(0.0, 1.0);
            let (cs, sn) = (sh.rot.cos(), sh.rot.sin());
            let pt = |lx: f32, ly: f32| (sh.x + lx * cs - ly * sn, sh.y + lx * sn + ly * cs);
            let v0 = pt(0.0, -sh.size);
            let v1 = pt(sh.size * 0.55, -sh.size * 0.12);
            let v2 = pt(0.0, sh.size * 0.88);
            let v3 = pt(-sh.size * 0.52, sh.size * 0.14);
            let body = with_a(sh.col, 0.84 * a);
            tri(v0, v1, v2, body);
            tri(v0, v2, v3, with_a(mix(sh.col, hex(0x28b8f0), 0.35), 0.80 * a));
            tri(v0, v1, (sh.x, sh.y), with_a(WHITE, 0.85 * a));
        }
    }

    fn draw_confetti(&self) {
        for cf in &self.confetti {
            let a = (1.0 - (cf.life / cf.max).powi(3)).clamp(0.0, 1.0);
            let piece = (cf.piece % 4) as f32;
            tex_sub(
                &self.assets.confetti,
                piece * 32.0,
                0.0,
                32.0,
                32.0,
                cf.x - cf.size * 0.5,
                cf.y - cf.size * 0.5,
                cf.size,
                cf.size,
                Color::new(1.0, 1.0, 1.0, a),
                false,
                cf.rot,
            );
        }
    }

    /// The field, players, ball, 2-layer goals and effects, through the match camera
    /// (with vertical-only shake and edge-clamped goal-celebration zoom so no gap ever opens).
    pub(crate) fn draw_world(&self, live: bool) {
        let (sw, sh) = (screen_width(), screen_height());
        let (s0, ox0, oy0) = match_view();
        let screen_band_top = (WORLD_BOTTOM * s0 + oy0).min(sh * (1.0 - BAND));
        // Subtle 1.0 -> 1.07x camera zoom toward scoring goal during goal celebration
        let (s, ox_base, oy_base) = if live && self.sim.phase == Phase::Goal {
            let zin = (self.goal_anim / 0.25).clamp(0.0, 1.0);
            let zin = 1.0 - (1.0 - zin).powi(3);
            let zout = ((GOAL_PAUSE - self.goal_anim) / 0.45).clamp(0.0, 1.0);
            let zoom = 1.0 + 0.07 * zin * zout;
            let fx = if self.sim.last_scorer == 0 { W * 0.78 } else { W * 0.22 };
            let fy = 430.0;
            let s = s0 * zoom;
            let ox = ox0 + fx * s0 * (1.0 - zoom);
            let oy = oy0 + fy * s0 * (1.0 - zoom);
            (s, ox, oy)
        } else {
            (s0, ox0, oy0)
        };
        // Vertical-only damped camera shake (never moves horizontally so sides never show a gap)
        let sy_off = if live && self.shake > 0.0 && self.shake_dur > 0.0 {
            let env = (self.shake / self.shake_dur).clamp(0.0, 1.0);
            let amp = self.shake_mag * env * env;
            let t = self.now as f32;
            if oy_base < -amp * s {
                (t * 95.0).sin() * amp
            } else {
                // Flush at top edge (16:9 / 4:3): bounce upward into the pitch apron covered by the band
                -(t * 52.0).sin().abs() * amp
            }
        } else {
            0.0
        };
        // Clamp camera bounds strictly so `0..W` covers `0..sw` and `0..H` covers `0..screen_band_top`
        let min_ox = (sw - W * s).min(0.0);
        let min_oy = (screen_band_top - H * s).min(0.0);
        let ox = ox_base.clamp(min_ox, 0.0);
        let oy = (oy_base + sy_off * s).clamp(min_oy, 0.0);
        with_view((s, ox, oy), || {
            self.draw_field(live);
            self.draw_scorch_marks();
            self.draw_goal_back();
            self.draw_ground_frost();
            for p in &self.particles {
                if p.behind {
                    let a = (1.0 - p.life / p.max).clamp(0.0, 1.0);
                    circ(p.x, p.y, p.r * a, with_a(p.c, p.c.a * a * 0.85));
                }
            }
            self.draw_player(0);
            self.draw_player(1);
            self.draw_ball();
            self.draw_goal_front();
            self.draw_ice_beams();
            self.draw_rings();
            for p in &self.particles {
                if !p.behind {
                    let a = (1.0 - p.life / p.max).clamp(0.0, 1.0);
                    circ(p.x, p.y, p.r, with_a(p.c, p.c.a * a));
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
                    FxKind::KickSwing => {
                        let frame = (k * 6.0) as usize;
                        let a = 0.36 * (1.0 - k);
                        tex_sub(&self.assets.fx_kick, frame as f32 * 128.0, 0.0, 128.0, 128.0, f.x - f.size * 0.5, f.y - f.size * 0.5, f.size, f.size, Color::new(0.88, 0.95, 1.0, a), f.flip_x, 0.0);
                    }
                    FxKind::Boom => {
                        let frame = (k * 8.0) as usize;
                        tex_center(&self.assets.boom[frame.min(7)], f.x, f.y, f.size, f.size, WHITE, false, 0.0);
                    }
                }
            }
            self.draw_ice_shards();
            self.draw_confetti();
        });
    }

    /// Brief icy cyan screen glint overlay on `EV_ICE_CAST`.
    fn draw_ice_flash(&self) {
        if self.ice_flash <= 0.0 {
            return;
        }
        let k = (self.ice_flash / 0.28).clamp(0.0, 1.0);
        let (l, t, r, _) = screen_box();
        let h = band_top() - t;
        rectv(l, t, r - l, h, with_a(hex(0x64e6ff), 0.20 * k * k));
        rect_line(l + 4.0, t + 4.0, (r - l) - 8.0, h - 8.0, 12.0, with_a(hex(0xb8f6ff), 0.55 * k));
    }

    /// The dark strip under the field where the buttons sit (`assets/ui/band.png` if it exists).
    pub(crate) fn draw_band(&self) {
        let (l, _, r, b) = screen_box();
        let top = band_top();
        if top >= b {
            return;
        }
        if let Some(tex) = self.assets.ui("band") {
            tex_draw(tex, l, top, r - l, b - top, WHITE, false, 0.0);
            return;
        }
        // Sleek dark navy/charcoal arcade bottom deck fallback with beveled metallic top lip
        let h = b - top;
        rectv(l, top, r - l, h, hex(0x12192b));
        for i in 0..6 {
            let y = top + h * (0.35 + i as f32 * 0.11);
            rectv(l, y, r - l, h * 0.12 + 1.0, with_a(hex(0x070a14), 0.12 + i as f32 * 0.06));
        }
        rectv(l, top, r - l, 8.0, hex(0x2d436e));
        rectv(l, top, r - l, 2.5, with_a(hex(0x88b8ff), 0.45));
        rectv(l, top + 8.0, r - l, 6.0, with_a(BLACK, 0.50));
    }

    /// When the ball flies above the visible top edge of the screen, a little ball and arrow
    /// mark its horizontal position (accounts for top crop on wide phones like 20:9).
    fn draw_ball_marker(&self) {
        let b = self.sim.ball;
        let (ms, mox, moy) = match_view();
        let seen_top = -moy / ms;
        if b.y + b.r() >= seen_top || self.ball_alpha() < 0.5 {
            return;
        }
        let (s, ox, _) = view();
        let (sl, t, sr, _) = screen_box();
        let x = ((b.x * ms + mox - ox) / s).clamp(sl + 24.0, sr - 24.0);
        let y = t + 126.0;
        let a = 0.65 + 0.30 * (self.now as f32 * 8.0).sin().abs();
        tri((x, y - 32.0), (x - 15.0, y - 15.0), (x + 15.0, y - 15.0), with_a(hex(0x0b1022), a * 0.85));
        tri((x, y - 30.0), (x - 12.0, y - 17.0), (x + 12.0, y - 17.0), with_a(WHITE, a));
        tex_center(&self.assets.ball, x, y + 2.0, 30.0, 30.0, with_a(WHITE, a), false, b.angle);
    }

    fn draw_match(&mut self) {
        self.draw_world(true);
        self.draw_ice_flash();
        self.draw_band();
        self.draw_ball_marker();
        self.draw_hud();
        self.draw_touch_buttons();
    }

    fn draw_hud(&mut self) {
        let sim = &self.sim;
        let t = self.now as f32;
        // Compact glossy top-center scoreboard (Y = 2..90, X = 478..802)
        let sb_w = 324.0;
        let sb_h = 88.0;
        let sb_x = 640.0 - sb_w * 0.5;
        let sb_y = 2.0;
        tex_draw(&self.assets.scoreboard, sb_x, sb_y, sb_w, sb_h, WHITE, false, 0.0);
        text_o(&sim.score[0].to_string(), sb_x + sb_w * 0.194, 61.0, 42.0, hex(0x9be2ff));
        text_o(&sim.score[1].to_string(), sb_x + sb_w * 0.806, 61.0, 42.0, hex(0xffa49b));
        let tl = sim.time_left.max(0.0).ceil() as i32;
        if sim.overtime {
            let pulse = 0.5 + 0.5 * (t * 6.0).sin().abs();
            text_o("OURO", 640.0, 58.0, 27.0, mix(WHITE, hex(0xffd34d), pulse));
        } else {
            let urgent = tl <= 10 && sim.phase == Phase::Play;
            let (tcol, tsz) = if urgent {
                let p = (t * 9.0).sin().abs();
                (mix(hex(0xff3e30), hex(0xffd2cc), p * 0.45), 32.0 * (1.0 + 0.11 * p))
            } else {
                (hex(0xffe14a), 32.0)
            };
            text_o(&format!("{}:{:02}", tl / 60, tl % 60), 640.0, 59.0, tsz, tcol);
        }
        self.hud.score = true;
        self.hud.timer = true;

        // Flanking HB2 player HUD cards (strictly inside X = 172..1108 -> >=18 px clear of top-right Power btn even at 150% scale!)
        const PW_COLS: [u32; 6] = [0xff7a18, 0xffc830, 0x44d8ff, 0xa878ff, 0xff4632, 0x38d898];
        for i in 0..2 {
            let p = sim.p[i];
            let look = p.look as usize % 6;
            let pkind = (p.pkind % POWER_KINDS) as usize;
            let ready = p.power >= 1.0 || p.power_ready > 0.0;
            // Bar & portrait coordinates (mirrored symmetrically around X = 640)
            let bar_w = 222.0;
            let bar_x = if i == 0 { 248.0 } else { 810.0 };
            let bar_y = 44.0;
            let bar_h = 22.0;
            let port_x = if i == 0 { 206.0 } else { 1074.0 };
            let port_y = 46.0;
            let port_r = 34.0;
            let badge_x = if i == 0 { 240.0 } else { 1040.0 };
            let badge_y = 63.0;

            // Player name above bar
            let name = self.player_name(i);
            let nw = text_w(name, 23.0);
            let nx = if i == 0 { bar_x + 6.0 } else { bar_x + bar_w - 6.0 - nw };
            text(name, nx + 2.0, 36.0, 23.0, with_a(BLACK, 0.8));
            text(name, nx, 34.0, 23.0, WHITE);

            // Sleek power charge bar track & fill
            rrect(bar_x, bar_y, bar_w, bar_h, 11.0, with_a(hex(0x0b1022), 0.88));
            rrect(bar_x + 1.0, bar_y + 1.0, bar_w - 2.0, bar_h - 2.0, 10.0, with_a(hex(0x1b2646), 0.65));
            let fill = if p.power_ready > 0.0 {
                (p.power_ready / game::POWER_READY_TIME).clamp(0.0, 1.0)
            } else {
                p.power.clamp(0.0, 1.0)
            };
            let fw = ((bar_w - 6.0) * fill).max(0.0);
            let pulse = if ready { 0.25 + 0.25 * (t * 8.0).sin() } else { 0.0 };
            let base_col = mix(hex(0xff8a1a), hex(PW_COLS[pkind]), 0.45);
            let col = mix(base_col, hex(0xfff0a0), pulse);
            if fw > 2.0 {
                let fx = if i == 0 { bar_x + 3.0 } else { bar_x + bar_w - 3.0 - fw };
                rrect(fx, bar_y + 3.0, fw, bar_h - 6.0, 8.0, col);
                rrect(fx + 2.0, bar_y + 4.0, (fw - 4.0).max(0.0), 5.0, 2.5, with_a(WHITE, 0.35));
            }
            let pname = POWER_NAMES[pkind].to_uppercase();
            let label = if p.power_ready > 0.0 {
                "CHUTE AGORA!".to_string()
            } else if p.power >= 1.0 {
                format!("{} PRONTO!", pname)
            } else {
                pname
            };
            let label_cx = if i == 0 { bar_x + bar_w * 0.53 } else { bar_x + bar_w * 0.47 };
            let max_lw = bar_w - 22.0;
            let lw0 = text_w(&label, 16.5);
            let lsz = if lw0 > max_lw { (16.5 * max_lw / lw0).max(12.0) } else { 16.5 };
            text_o(&label, label_cx, bar_y + 17.0, lsz, WHITE);

            // Circular framed character mini-portrait
            let team_bg = if i == 0 { hex(0x1a5cd4) } else { hex(0xcf2438) };
            let ring_col = if ready { mix(hex(0xffd34d), WHITE, 0.3 + 0.3 * (t * 8.0).sin()) } else { hex(0xdce8ff) };
            circ(port_x, port_y + 3.0, port_r + 2.0, with_a(BLACK, 0.45));
            circ(port_x, port_y, port_r, team_bg);
            circ(port_x, port_y - 6.0, port_r * 0.72, with_a(WHITE, 0.14));
            tex_center(&self.assets.heads[look], port_x, port_y + 1.0, 54.0, 54.0, WHITE, i == 1, 0.0);
            circ_line(port_x, port_y, port_r, 3.5, ring_col);
            circ_line(port_x, port_y, port_r + 2.2, 1.8, with_a(hex(0x0b1022), 0.9));

            // Mini power icon badge pulsing when ready
            let b_pulse = if ready { 1.0 + 0.14 * (t * 9.0).sin().abs() } else { 1.0 };
            if ready {
                circ(badge_x, badge_y, 20.0 * b_pulse, with_a(hex(0xffe14a), 0.38));
            }
            circ(badge_x, badge_y, 16.5, hex(0x101830));
            circ_line(badge_x, badge_y, 16.5, 2.2, if ready { hex(0xffd34d) } else { hex(0x5a709c) });
            let icon_tint = if ready { WHITE } else { Color::new(0.70, 0.72, 0.80, 0.90) };
            let icon_sz = 27.0 * b_pulse;
            tex_center(&self.assets.btn_powers[pkind], badge_x, badge_y, icon_sz, icon_sz, icon_tint, false, 0.0);
        }
        self.hud.power = true;
        if let Some(tour) = self.tour {
            let label = format!("Campeonato: {}", TOUR_ROUNDS[tour.round.min(2)].0);
            let lw = text_w(&label, 19.0) + 28.0;
            rrect(640.0 - lw * 0.5, 92.0, lw, 26.0, 13.0, with_a(hex(0x0b1022), 0.75));
            text_o(&label, 640.0, 111.0, 19.0, hex(0xffe9a0));
        }

        // Big centre messages (hidden while the pause or "connection lost" box is open)
        if self.paused || self.lost {
            return;
        }
        match sim.phase {
            Phase::Countdown => {
                // "versus" banner at the very start of the match
                if sim.score == [0, 0] {
                    rrect(250.0, 172.0, 780.0, 164.0, 24.0, with_a(BLACK, 0.52));
                    let bob = (t * 4.0).sin() * 3.0;
                    tex_center(&self.assets.heads[sim.p[0].look as usize % 6], 400.0, 242.0 + bob, 124.0, 124.0, WHITE, false, 0.0);
                    tex_center(&self.assets.heads[sim.p[1].look as usize % 6], 880.0, 242.0 - bob, 124.0, 124.0, WHITE, true, 0.0);
                    text_o("VS", 640.0, 264.0, 84.0, hex(0xffd34d));
                    if let Some(tour) = self.tour {
                        text_o(TOUR_ROUNDS[tour.round.min(2)].0, 640.0, 306.0, 24.0, WHITE);
                    }
                    text_o(self.player_name(0), 400.0, 324.0, 26.0, hex(0x9be2ff));
                    text_o(self.player_name(1), 880.0, 324.0, 26.0, hex(0xffa49b));
                }
                let n = sim.phase_t.ceil() as i32;
                let k = 1.0 + (sim.phase_t.fract()) * 0.4;
                text_o(&n.max(1).to_string(), 640.0, 448.0, 120.0 * k, hex(0xffd34d));
            }
            Phase::Ready => {
                if sim.overtime {
                    text_o("GOL DE OURO!", 640.0, 286.0, 72.0, hex(0xffd34d));
                    text_o("quem marcar primeiro vence", 640.0, 332.0, 28.0, WHITE);
                } else {
                    text_o("PREPARAR...", 640.0, 290.0, 60.0, WHITE);
                }
            }
            Phase::Play => {
                if sim.time_left > sim.match_len - 0.8 {
                    text_o("JÁ!", 640.0, 304.0, 110.0, hex(0xffd34d));
                }
            }
            Phase::Goal => {
                let ga = self.goal_anim;
                let k = (ga / 0.35).clamp(0.0, 1.0);
                let elastic = if k < 1.0 {
                    1.0 + 0.34 * (-5.0 * k).exp() * (k * 11.0).sin() - (1.0 - k).powi(3)
                } else {
                    1.0 + 0.04 * (ga * 8.0).sin()
                };
                let cy = 278.0;
                // Rotating golden starburst rays behind "GOL!"
                let rays = 12usize;
                let r_outer = (225.0 * elastic).clamp(60.0, 290.0);
                for r_i in 0..rays {
                    let a0 = t * 1.1 + (r_i as f32) * (std::f32::consts::TAU / rays as f32);
                    let a1 = a0 + 0.19;
                    tri(
                        (640.0, cy - 16.0),
                        (640.0 + a0.cos() * r_outer, cy - 16.0 + a0.sin() * r_outer * 0.68),
                        (640.0 + a1.cos() * r_outer, cy - 16.0 + a1.sin() * r_outer * 0.68),
                        with_a(hex(0xffe14a), 0.24),
                    );
                }
                circ(640.0, cy - 16.0, (92.0 * elastic).max(20.0), with_a(hex(0xffb820), 0.22));
                let size = (138.0 * elastic).max(38.0);
                text_o("GOL!", 640.0, cy + 28.0, size, hex(0xffd34d));
                let scorer = sim.last_scorer as usize % 2;
                let sname = self.player_name(scorer);
                let pill_w = (text_w(sname, 30.0) + 64.0).max(180.0);
                let pill_col = if scorer == 0 { hex(0x1a5cd4) } else { hex(0xcf2438) };
                rrect(640.0 - pill_w * 0.5, cy + 50.0, pill_w, 44.0, 22.0, with_a(pill_col, 0.90));
                rect_line(640.0 - pill_w * 0.5 + 2.0, cy + 52.0, pill_w - 4.0, 40.0, 2.0, with_a(hex(0xffd34d), 0.75));
                text_o(sname, 640.0, cy + 82.0, 30.0, WHITE);
            }
            Phase::Over => {
                if self.over_t <= 1.2 {
                    text_o("FIM DE JOGO", 640.0, 304.0, 80.0, WHITE);
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
        let (_, _, _, b) = screen_box();
        let cy = (band_top() + b) * 0.5;
        rrect(640.0 - w * 0.5, cy - 15.0, w, 30.0, 15.0, with_a(BLACK, 0.5 * fade));
        text(msg, 640.0 - w * 0.5 + 22.0, cy + 6.0, size, with_a(WHITE, 0.95 * fade));
    }

    fn draw_touch_buttons(&mut self) {
        if !self.touch_ui {
            self.draw_controls_hint();
            return;
        }
        self.draw_buttons_for(self.mode);
        self.hud.buttons = true;
    }

    /// The on-screen buttons of `mode`, with the size and opacity from the settings.
    /// Uses the Head Ball 2-shaped pictures `assets/ui/ctrl_*.png` when they exist, and
    /// the square button pictures otherwise.
    pub(crate) fn draw_buttons_for(&self, mode: Mode) {
        let t = self.now as f32;
        let alpha = self.set.btn_alpha as f32 / 100.0;
        for b in touch_layout(mode, &self.set) {
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
            let mirror = b.who == 1 && mode == Mode::Local2;
            // the Head Ball 2-shaped pictures, when they exist: (picture, mirrored)
            let ui = |name: &str, flip: bool| self.assets.ui(name).map(|t| (t, flip));
            let wide = match b.kind {
                Btn::Left => ui("ctrl_left", false),
                Btn::Right => ui("ctrl_right", false).or_else(|| ui("ctrl_left", true)),
                Btn::Kick => ui("ctrl_kick", mirror),
                Btn::Lob => ui("ctrl_lob", mirror),
                Btn::Jump => ui("ctrl_jump", mirror),
                Btn::Power => None,
            };
            let pulse = if b.kind == Btn::Power && ready { 1.0 + 0.06 * (t * 8.0).sin() } else { 1.0 };
            let k = if down { 0.92 } else { 1.0 } * pulse;
            if b.kind == Btn::Power {
                // grey while charging, filling up with colour from the bottom
                let tex = &self.assets.btn_powers[(p.pkind as usize) % 6];
                let sz = b.h * k;
                let fill = if ready { 1.0 } else { p.power.clamp(0.0, 1.0) };
                tex_center(tex, b.x, b.y, sz, sz, Color::new(0.42, 0.42, 0.46, 0.75 * alpha), false, 0.0);
                if fill > 0.01 {
                    let (tw, th) = (tex.width(), tex.height());
                    let a = if ready { alpha.max(0.9) } else { alpha * 0.85 };
                    tex_sub(tex, 0.0, th * (1.0 - fill), tw, th * fill, b.x - sz * 0.5, b.y + sz * 0.5 - sz * fill, sz, sz * fill, Color::new(1.0, 1.0, 1.0, a), false, 0.0);
                }
                continue;
            }
            let tint = match b.kind {
                Btn::Power if !ready => Color::new(0.45, 0.45, 0.48, 0.65 * alpha),
                _ if down => Color::new(1.0, 1.0, 1.0, alpha.max(0.85)),
                _ => Color::new(1.0, 1.0, 1.0, alpha),
            };
            if let Some((tex, flip)) = wide {
                tex_center(tex, b.x, b.y, b.w * k, b.h * k, tint, flip, 0.0);
            } else {
                let (tex, flip) = match b.kind {
                    Btn::Left => (&self.assets.btn_left, false),
                    Btn::Right => (&self.assets.btn_left, true),
                    Btn::Kick => (&self.assets.btn_kick, mirror),
                    Btn::Lob => (&self.assets.btn_lob, mirror),
                    Btn::Jump => (&self.assets.btn_jump, mirror),
                    Btn::Power => (&self.assets.btn_powers[(p.pkind as usize) % 6], false),
                };
                let sz = b.h * 1.08 * k;
                tex_center(tex, b.x, b.y, sz, sz, tint, flip, 0.0);
            }
        }
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

fn star_outlined(x: f32, y: f32, r: f32, c: Color) {
    star(x, y + 1.0, r * 1.25, with_a(hex(0x140f08), c.a * 0.92));
    star(x, y, r, c);
    circ(x - r * 0.12, y - r * 0.12, r * 0.25, with_a(WHITE, c.a * 0.85));
}


// ---------------------------------------------------------------- start-up

fn conf() -> Conf {
    // `--size 1600x720`: another window shape, to see how wide phones or tablets look
    let size = std::env::args().skip_while(|a| a != "--size").nth(1).and_then(|s| {
        let (w, h) = s.split_once('x')?;
        Some((w.parse().ok()?, h.parse().ok()?))
    });
    let (w, h) = size.unwrap_or((1280, 720));
    Conf {
        window_title: "Cabeçada".to_owned(),
        window_width: w,
        window_height: h,
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
    let music = if quiet { None } else { macroquad::audio::load_sound_from_bytes(&music::menu_track()).await.ok() };
    let mut app = App::new(sounds, music, assets);
    if quiet {
        app.set = settings::Settings::default();
    }
    platform::log("menu");
    match run {
        Run::Normal => loop {
            let dt = get_frame_time();
            app.frame(dt).await;
            next_frame().await;
        },
        Run::Screenshot(path) => {
            app.ai_both = true;
            app.set.level = 2;
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
                set_view_top();
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
                "menu", "multi", "setup_cpu", "setup_cup", "setup_local2", "setup_lan_host", "setup_online_host", "setup_online_join", "settings", "button_test", "online_host",
                "online_join", "host_lobby", "join_lobby", "countdown", "countdown_tour", "play_cpu", "play_touch", "play_touch_moved", "play_local2", "play_local2_touch", "stun",
                "goal", "overtime", "over", "eliminated", "champion", "pause", "pw_fire", "pw_big", "pw_ice", "pw_invis", "pw_bomb",
                "pw_clone",
            ];
            // `--only menu,settings`: just these
            let only: Option<Vec<String>> = std::env::args().skip_while(|a| a != "--only").nth(1).map(|s| s.split(',').map(|x| x.to_string()).collect());
            let mut first = true;
            for (n, name) in scenes.iter().enumerate() {
                if let Some(o) = &only {
                    if !o.iter().any(|x| x == name) {
                        continue;
                    }
                }
                app.scene_setup(name);
                let last = if name.starts_with("pw_") || *name == "stun" {
                    14
                } else if *name == "goal" {
                    24
                } else {
                    90
                };
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
                set_view_top();
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
    // `--online-test <file>`: try an online room through the internet (no window)
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--online-test") {
        let (ok, detail) = selftest::online_test();
        let line = format!("{{\"step\": \"online\", \"passed\": {}, \"detail\": {:?}}}", ok, detail);
        println!("{}", line);
        if let Some(path) = args.get(i + 1) {
            let _ = std::fs::write(path, &line);
        }
        std::process::exit(if ok { 0 } else { 1 });
    }
    let run = parse_args();
    let prev = win::foreground();
    macroquad::Window::from_config(conf(), amain(run, prev));
}
