//! The menus: main menu, multiplayer, match setup (choose everything, then play),
//! settings, button test and the online rooms. Everything is drawn on the 1280x720
//! virtual screen. Optional pictures from `assets/ui/` are used when they exist
//! (see PARA_GEMINI_2.md); otherwise these screens draw their own shapes.

use crate::game::*;
use crate::settings::{BTN_ALPHA_MIN, BTN_SCALE_MAX, BTN_SCALE_MIN};
use crate::*;
use macroquad::prelude::*;

/// Which match the setup screen is preparing.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SetupKind {
    VsCpu,
    Cup,
    Local2,
    LanHost,
    LanJoin,
    OnlineHost,
    OnlineJoin,
}

pub const STADIUM_NAMES: [&str; 3] = ["Noite", "Pôr do Sol", "Final"];
pub const POWER_INFO: [&str; 7] = [
    "o próximo chute vai reto no gol e derruba quem estiver no caminho",
    "a bola fica gigante por alguns segundos",
    "congela o adversário na hora (aperte botões para quebrar o gelo)",
    "a bola some para o adversário por alguns segundos",
    "o próximo chute vira uma bomba que explode perto do adversário",
    "o próximo chute solta duas bolas falsas para confundir",
    "um poder sorteado a cada partida",
];

// ------------------------------------------------------------------ small widgets

/// A round button with an icon drawn by `icon` (or the picture `ui_name` if it exists).
pub fn round_button(app: &App, cx: f32, cy: f32, r: f32, ui_name: &str, icon: impl Fn(f32, f32, f32)) -> bool {
    let (mx, my) = to_virtual(mouse_position().0, mouse_position().1);
    let hover = (mx - cx).powi(2) + (my - cy).powi(2) < r * r;
    if let Some(tex) = app.assets.ui(ui_name) {
        let k = if hover { 1.06 } else { 1.0 };
        tex_center(tex, cx, cy, r * 2.0 * k, r * 2.0 * k, WHITE, false, 0.0);
    } else {
        circ(cx, cy + 4.0, r, with_a(BLACK, 0.35));
        circ(cx, cy, r, if hover { hex(0x3d5a9a) } else { hex(0x2c4377) });
        circ_line(cx, cy, r, 3.0, with_a(WHITE, 0.75));
        icon(cx, cy, r);
    }
    if let Some((x, y)) = app.clicked() {
        return (x - cx).powi(2) + (y - cy).powi(2) < (r * 1.15).powi(2);
    }
    false
}

/// A gear (settings icon) drawn with shapes.
pub fn gear_icon(cx: f32, cy: f32, r: f32) {
    let c = WHITE;
    for k in 0..8 {
        let a = k as f32 * std::f32::consts::TAU / 8.0;
        circ(cx + a.cos() * r * 0.52, cy + a.sin() * r * 0.52, r * 0.17, c);
    }
    circ(cx, cy, r * 0.48, c);
    circ(cx, cy, r * 0.22, hex(0x2c4377));
}

/// A Wi-Fi sign drawn with shapes (until `assets/ui/icon_wifi.png` exists).
fn wifi_icon(cx: f32, cy: f32, r: f32) {
    let base = cy + r * 0.62;
    let c = hex(0xcfe3ff);
    for k in 1..=3 {
        let rr = r * 0.34 * k as f32;
        draw_arc(px(cx), py(base), 40, sc(rr), 222.0, sc(r * 0.15), 96.0, c);
    }
    circ(cx, base, r * 0.13, c);
}

/// A globe drawn with shapes (until `assets/ui/icon_online.png` exists).
fn globe_icon(cx: f32, cy: f32, r: f32) {
    let c = hex(0xcfe3ff);
    let t = r * 0.1;
    circ_line(cx, cy, r, t, c);
    draw_ellipse_lines(px(cx), py(cy), sc(r * 0.45), sc(r), 0.0, sc(t), c);
    linev(cx - r, cy, cx + r, cy, t, c);
    linev(cx, cy - r, cx, cy + r, t, c);
    for s in [-1.0f32, 1.0] {
        let half = r * 0.84;
        linev(cx - half, cy + s * r * 0.52, cx + half, cy + s * r * 0.52, t * 0.8, c);
    }
}

/// An arrow (triangle) icon pointing left (-1) or right (+1).
pub fn arrow_icon(cx: f32, cy: f32, r: f32, dir: f32) {
    let s = r * 0.42;
    tri((cx + dir * s, cy), (cx - dir * s * 0.7, cy - s), (cx - dir * s * 0.7, cy + s), WHITE);
}

/// Options side by side; returns the one clicked.
pub fn segmented(app: &App, x: f32, y: f32, w: f32, h: f32, labels: &[&str], selected: usize) -> Option<usize> {
    let n = labels.len() as f32;
    let gap = 8.0;
    let bw = (w - gap * (n - 1.0)) / n;
    let mut pick = None;
    for (i, l) in labels.iter().enumerate() {
        let bx = x + i as f32 * (bw + gap);
        let col = if i == selected { hex(0x2f8f4e) } else { hex(0x34405e) };
        if app.button(l, bx, y, bw, h, col) {
            pick = Some(i);
        }
    }
    pick
}

/// "−  [bar]  +" control. Returns -1, 0 or +1.
pub fn stepper(app: &App, x: f32, y: f32, w: f32, frac: f32, value: &str) -> i32 {
    let h = 54.0;
    let mut d = 0;
    if app.button("", x, y, 64.0, h, hex(0x34405e)) {
        d = -1;
    }
    if app.button("", x + w - 64.0, y, 64.0, h, hex(0x34405e)) {
        d = 1;
    }
    // the minus and plus signs, drawn (the font's are too thin)
    let cy = y + h * 0.5 - 1.0;
    rectv(x + 20.0, cy - 3.0, 24.0, 6.0, WHITE);
    rectv(x + w - 44.0, cy - 3.0, 24.0, 6.0, WHITE);
    rectv(x + w - 35.0, cy - 12.0, 6.0, 24.0, WHITE);
    let bx = x + 78.0;
    let bw = w - 156.0;
    rrect(bx, y + 14.0, bw, h - 28.0, 13.0, with_a(BLACK, 0.55));
    if frac > 0.0 {
        rrect(bx + 3.0, y + 17.0, ((bw - 6.0) * frac.clamp(0.0, 1.0)).max(20.0), h - 34.0, 10.0, hex(0xffc83a));
    }
    text_o(value, bx + bw * 0.5, y + h * 0.5 + 9.0, 24.0, WHITE);
    d
}

/// A dark rounded panel (or the 9-slice picture `panel` if it exists).
pub fn panel(app: &App, x: f32, y: f32, w: f32, h: f32) {
    if let Some(tex) = app.assets.ui("panel") {
        draw_9slice(tex, x, y, w, h, WHITE);
    } else {
        rrect(x, y + 6.0, w, h, 22.0, with_a(BLACK, 0.3));
        rrect(x, y, w, h, 22.0, with_a(hex(0x0b1430), 0.78));
    }
}

fn back_pressed() -> bool {
    is_key_pressed(KeyCode::Back) || is_key_pressed(KeyCode::Escape)
}

impl App {
    /// "Voltar" in the bottom-left corner (also the phone's back button / Esc).
    fn back_button(&self) -> bool {
        self.button("Voltar", 40.0, 630.0, 230.0, 62.0, hex(0x5a2d3a)) || back_pressed()
    }

    // -------------------------------------------------------------- main menu

    pub(crate) fn menu_main(&mut self) {
        draw_cover(&self.assets.menu_bg, WHITE);
        dim_screen(with_a(hex(0x080d1e), 0.22));
        let (_, st, sr, sb) = screen_box();
        let version = concat!("v", env!("CARGO_PKG_VERSION"));
        text(version, sr - 8.0 - text_w(version, 16.0), sb - 8.0, 16.0, with_a(WHITE, 0.55));
        let t = self.now as f32;
        let bob1 = (t * 2.8).sin() * 6.0;
        let bob2 = (t * 2.8 + 1.6).sin() * 6.0;
        let (c0, c1) = (self.set.look as usize % 6, self.set.look2 as usize % 6);
        ellipse(190.0, 512.0, 62.0, 14.0, with_a(BLACK, 0.35));
        draw_char_jersey(c0, 190.0, 444.0 + bob1 * 0.5, 1.25, false, 0.0, WHITE);
        tex_center(&self.assets.boots[c0], 198.0, 488.0 + bob1 * 0.3, 96.0, 64.0, WHITE, false, -0.05);
        tex_center(&self.assets.heads[c0], 190.0, 394.0 + bob1, 168.0, 168.0, WHITE, false, (t * 1.4).sin() * 0.04);
        ellipse(1090.0, 512.0, 62.0, 14.0, with_a(BLACK, 0.35));
        draw_char_jersey(c1, 1090.0, 444.0 + bob2 * 0.5, 1.25, true, 0.0, WHITE);
        tex_center(&self.assets.boots[c1], 1082.0, 488.0 + bob2 * 0.3, 96.0, 64.0, WHITE, true, 0.05);
        tex_center(&self.assets.heads[c1], 1090.0, 394.0 + bob2, 168.0, 168.0, WHITE, true, -(t * 1.4).sin() * 0.04);
        text_o(CHARS[c0].name, 190.0, 560.0, 28.0, WHITE);
        text_o(CHARS[c1].name, 1090.0, 560.0, 28.0, WHITE);

        tex_center(&self.assets.logo, 640.0, 128.0 + (t * 2.2).sin() * 4.0, 470.0, 235.0, WHITE, false, 0.0);
        rrect(290.0, 236.0, 700.0, 34.0, 17.0, with_a(BLACK, 0.5));
        text_o("futebol de cabeça 1 contra 1", 640.0, 261.0, 22.0, hex(0xe4f0ff));

        let (bx, bw, bh) = (390.0, 500.0, 70.0);
        if self.button("JOGAR", bx, 290.0, bw, bh, hex(0x2f8f4e)) {
            self.open_setup(SetupKind::VsCpu);
            return;
        }
        if self.button("CAMPEONATO", bx, 372.0, bw, bh, hex(0xc9951a)) {
            self.open_setup(SetupKind::Cup);
            return;
        }
        if self.button("2 JOGADORES", bx, 454.0, bw, bh, hex(0x2d6fd0)) {
            self.open_setup(SetupKind::Local2);
            return;
        }
        if self.button("MULTIJOGADOR", bx, 536.0, bw, bh, hex(0x8a3fc0)) {
            self.screen = Screen::Multi;
            return;
        }
        if round_button(self, sr - 66.0, st + 62.0, 38.0, "icon_settings", gear_icon) {
            self.open_settings(Screen::Menu);
            return;
        }
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 650.0, 22.0, hex(0xff8080));
        }
    }

    pub(crate) fn open_settings(&mut self, back: Screen) {
        self.settings_back = back;
        self.screen = Screen::Settings;
    }

    pub(crate) fn open_setup(&mut self, kind: SetupKind) {
        self.setup = kind;
        self.status.clear();
        if self.set.look2 == self.set.look {
            self.set.look2 = (self.set.look + 1) % 6;
        }
        self.screen = Screen::Setup;
    }

    // -------------------------------------------------------------- multiplayer

    pub(crate) fn multi(&mut self) {
        self.draw_backdrop();
        text_o("MULTIJOGADOR", 640.0, 86.0, 54.0, hex(0xffd34d));
        for (i, (title, lines)) in [
            ("NA MESMA REDE WI-FI", ["Os dois aparelhos no mesmo Wi-Fi", "(ou um no roteador do outro).", "Sem atraso e sem internet."]),
            ("PELA INTERNET", ["Para jogar com quem está longe.", "Um cria a sala e manda o código.", "Pode ter um pouco de atraso."]),
        ]
        .iter()
        .enumerate()
        {
            let x = if i == 0 { 90.0 } else { 670.0 };
            panel(self, x, 130.0, 520.0, 460.0);
            if let Some(tex) = self.assets.ui(if i == 0 { "icon_wifi" } else { "icon_online" }) {
                tex_center(tex, x + 260.0, 215.0, 110.0, 110.0, WHITE, false, 0.0);
            } else if i == 0 {
                wifi_icon(x + 260.0, 212.0, 52.0);
            } else {
                globe_icon(x + 260.0, 215.0, 44.0);
            }
            text_o(title, x + 260.0, 300.0, 32.0, WHITE);
            for (k, l) in lines.iter().enumerate() {
                text_o(l, x + 260.0, 340.0 + k as f32 * 30.0, 22.0, hex(0xb8c6e6));
            }
        }
        if self.button("CRIAR PARTIDA", 130.0, 440.0, 440.0, 60.0, hex(0xd06a2d)) {
            self.open_setup(SetupKind::LanHost);
            return;
        }
        if self.button("ENTRAR NA PARTIDA", 130.0, 510.0, 440.0, 60.0, hex(0x2d6fd0)) {
            self.open_setup(SetupKind::LanJoin);
            return;
        }
        if self.button("CRIAR SALA", 710.0, 440.0, 440.0, 60.0, hex(0xd06a2d)) {
            self.open_setup(SetupKind::OnlineHost);
            return;
        }
        if self.button("ENTRAR COM CÓDIGO", 710.0, 510.0, 440.0, 60.0, hex(0x2d6fd0)) {
            self.open_setup(SetupKind::OnlineJoin);
            return;
        }
        if self.back_button() {
            self.screen = Screen::Menu;
        }
    }

    // -------------------------------------------------------------- match setup

    pub(crate) fn setup_screen(&mut self) {
        let kind = self.setup;
        self.draw_backdrop();
        let title = match kind {
            SetupKind::VsCpu => "CONTRA O COMPUTADOR",
            SetupKind::Cup => "CAMPEONATO",
            SetupKind::Local2 => "2 JOGADORES",
            SetupKind::LanHost => "CRIAR PARTIDA (WI-FI)",
            SetupKind::LanJoin => "ENTRAR NA PARTIDA (WI-FI)",
            SetupKind::OnlineHost => "CRIAR SALA ONLINE",
            SetupKind::OnlineJoin => "ENTRAR NA SALA ONLINE",
        };
        text_o(title, 640.0, 70.0, 46.0, hex(0xffd34d));

        // ---- left: your character
        panel(self, 40.0, 100.0, 560.0, 510.0);
        let you = if kind == SetupKind::Local2 { "JOGADOR 1" } else { "SEU JOGADOR" };
        text_o(you, 320.0, 140.0, 26.0, hex(0xcfe3ff));
        let look = self.set.look as usize % 6;
        let t = self.now as f32;
        let bob = (t * 2.6).sin() * 4.0;
        ellipse(320.0, 354.0, 74.0, 14.0, with_a(BLACK, 0.35));
        draw_char_jersey(look, 320.0, 282.0 + bob * 0.45, 1.38, false, 0.0, WHITE);
        tex_center(&self.assets.boots[look], 330.0, 330.0, 110.0, 74.0, WHITE, false, -0.05);
        tex_center(&self.assets.heads[look], 320.0, 232.0 + bob, 190.0, 190.0, WHITE, false, 0.0);
        if round_button(self, 110.0, 250.0, 34.0, "icon_prev", |x, y, r| arrow_icon(x, y, r, -1.0)) {
            self.set.look = (self.set.look + 5) % 6;
        }
        if round_button(self, 530.0, 250.0, 34.0, "icon_next", |x, y, r| arrow_icon(x, y, r, 1.0)) {
            self.set.look = (self.set.look + 1) % 6;
        }
        let st = char_stats(self.set.look);
        text_o(st.name, 320.0, 404.0, 38.0, WHITE);
        for (i, (label, v)) in [("Velocidade", st.speed), ("Pulo", st.jump), ("Chute", st.kick), ("Poder", st.charge)].iter().enumerate() {
            let y = 428.0 + i as f32 * 40.0;
            text(label, 80.0, y + 24.0, 22.0, hex(0xcfe3ff));
            rrect(230.0, y + 6.0, 330.0, 22.0, 11.0, with_a(BLACK, 0.55));
            let frac = ((v - 0.86) / 0.3).clamp(0.12, 1.0);
            let col = if *v > 1.01 { hex(0x5fd35f) } else if *v < 0.99 { hex(0xff9a4a) } else { hex(0xffc83a) };
            rrect(232.0, y + 8.0, 326.0 * frac, 18.0, 9.0, col);
        }
        if look_eq_fix(&mut self.set) {}

        // ---- right: power and match options
        panel(self, 640.0, 100.0, 600.0, 510.0);
        text_o("PODER ESPECIAL", 940.0, 140.0, 26.0, hex(0xcfe3ff));
        let sel = self.set.power.min(POWER_KINDS) as usize;
        for k in 0..=POWER_KINDS as usize {
            let cx = 690.0 + k as f32 * 83.0;
            let cy = 196.0;
            let s = if k == sel { 76.0 } else { 66.0 };
            if k == sel {
                rrect(cx - 42.0, cy - 42.0, 84.0, 84.0, 16.0, hex(0xffc83a));
            }
            if k < POWER_KINDS as usize {
                tex_center(&self.assets.btn_powers[k], cx, cy, s, s, WHITE, false, 0.0);
            } else if let Some(tex) = self.assets.ui("power_random") {
                tex_center(tex, cx, cy, s, s, WHITE, false, 0.0);
            } else {
                rrect(cx - s * 0.5, cy - s * 0.5, s, s, 14.0, hex(0x6a4fb0));
                text_o("?", cx, cy + 16.0, 46.0, WHITE);
            }
            if let Some((x, y)) = self.clicked() {
                if (x - cx).abs() < 40.0 && (y - cy).abs() < 40.0 {
                    self.set.power = k as u8;
                }
            }
        }
        let pname = if sel < POWER_KINDS as usize { POWER_NAMES[sel] } else { "Aleatório" };
        text_o(pname, 940.0, 268.0, 30.0, WHITE);
        text_o(POWER_INFO[sel], 940.0, 296.0, 18.0, hex(0xb8c6e6));

        let mut y = 322.0;
        // opponent
        match kind {
            SetupKind::VsCpu | SetupKind::Local2 => {
                let label = if kind == SetupKind::Local2 { "JOGADOR 2" } else { "ADVERSÁRIO" };
                text(label, 670.0, y + 38.0, 24.0, hex(0xcfe3ff));
                let l2 = self.set.look2 as usize % 6;
                tex_center(&self.assets.heads[l2], 1000.0, y + 30.0, 64.0, 64.0, WHITE, true, 0.0);
                text_o(CHARS[l2].name, 1100.0, y + 40.0, 26.0, WHITE);
                if round_button(self, 925.0, y + 30.0, 24.0, "icon_prev", |x, y, r| arrow_icon(x, y, r, -1.0)) {
                    self.set.look2 = (self.set.look2 + 5) % 6;
                }
                if round_button(self, 1190.0, y + 30.0, 24.0, "icon_next", |x, y, r| arrow_icon(x, y, r, 1.0)) {
                    self.set.look2 = (self.set.look2 + 1) % 6;
                }
            }
            SetupKind::Cup => {
                text("FASES", 670.0, y + 38.0, 24.0, hex(0xcfe3ff));
                text_o("Quartas  >  Semifinal  >  Final", 1010.0, y + 38.0, 22.0, WHITE);
            }
            _ => {
                text("ADVERSÁRIO", 670.0, y + 38.0, 24.0, hex(0xcfe3ff));
                text_o("escolhe no aparelho dele", 1030.0, y + 38.0, 22.0, WHITE);
            }
        }
        y += 70.0;
        let host_decides = matches!(kind, SetupKind::LanJoin | SetupKind::OnlineJoin);
        if !host_decides && kind != SetupKind::Cup {
            text("ESTÁDIO", 670.0, y + 36.0, 24.0, hex(0xcfe3ff));
            let sidx = self.set.stadium as usize % 3;
            tex_draw(&self.assets.stadium_bgs[sidx], 940.0, y + 4.0, 96.0, 54.0, WHITE, false, 0.0);
            text_o(STADIUM_NAMES[sidx], 1110.0, y + 38.0, 22.0, WHITE);
            if round_button(self, 895.0, y + 31.0, 24.0, "icon_prev", |x, y, r| arrow_icon(x, y, r, -1.0)) {
                self.set.stadium = (self.set.stadium + 2) % 3;
            }
            if round_button(self, 1190.0, y + 31.0, 24.0, "icon_next", |x, y, r| arrow_icon(x, y, r, 1.0)) {
                self.set.stadium = (self.set.stadium + 1) % 3;
            }
            y += 70.0;
        }
        if !host_decides {
            text("TEMPO", 670.0, y + 36.0, 24.0, hex(0xcfe3ff));
            if let Some(i) = segmented(self, 840.0, y + 6.0, 370.0, 52.0, &["60 s", "90 s", "120 s"], self.set.len_idx as usize) {
                self.set.len_idx = i as u8;
            }
            y += 70.0;
        }
        if kind == SetupKind::VsCpu {
            text("NÍVEL", 670.0, y + 36.0, 24.0, hex(0xcfe3ff));
            if let Some(i) = segmented(self, 840.0, y + 6.0, 370.0, 52.0, &crate::LEVEL_NAMES, self.set.level as usize) {
                self.set.level = i as u8;
            }
        }
        if host_decides {
            text_o("O estádio e o tempo são escolhidos por quem criou.", 940.0, y + 40.0, 20.0, hex(0xb8c6e6));
        }

        // ---- bottom
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 616.0 + 18.0, 22.0, hex(0xff8080));
        }
        if self.back_button() {
            self.screen = if matches!(kind, SetupKind::VsCpu | SetupKind::Cup | SetupKind::Local2) { Screen::Menu } else { Screen::Multi };
            return;
        }
        let go = match kind {
            SetupKind::VsCpu | SetupKind::Local2 => "JOGAR",
            SetupKind::Cup => "COMEÇAR",
            SetupKind::LanHost => "CRIAR PARTIDA",
            SetupKind::LanJoin => "PROCURAR",
            SetupKind::OnlineHost => "CRIAR SALA",
            SetupKind::OnlineJoin => "CONTINUAR",
        };
        if self.button(go, 900.0, 628.0, 340.0, 70.0, hex(0xc9951a)) || is_key_pressed(KeyCode::Enter) {
            self.set.save();
            self.setup_go(kind);
        }
    }

    /// The setup is done: start the match or open the right lobby.
    fn setup_go(&mut self, kind: SetupKind) {
        match kind {
            SetupKind::VsCpu => {
                self.tour = None;
                self.start_match(Mode::Cpu);
            }
            SetupKind::Cup => {
                self.tour = Some(Tour { round: 0 });
                self.start_match(Mode::Cpu);
            }
            SetupKind::Local2 => self.start_match(Mode::Local2),
            SetupKind::LanHost => {
                let ip = net::local_ip().map(|i| i.to_string()).unwrap_or_else(|| "sem rede".into());
                let name = format!("Partida {}", ip);
                match net::Host::new(net::PORT, &name).or_else(|_| net::Host::new(0, &name)) {
                    Ok(h) => {
                        self.my_ip = if h.port == net::PORT { ip } else { format!("{}:{}", ip, h.port) };
                        self.host = Some(h);
                        self.screen = Screen::HostLobby;
                    }
                    Err(e) => self.status = format!("Não foi possível abrir a partida: {}", e),
                }
            }
            SetupKind::LanJoin => match net::Client::new() {
                Ok(mut c) => {
                    c.pkind = self.pick_power();
                    c.look = self.set.look;
                    self.client = Some(c);
                    self.status.clear();
                    self.screen = Screen::JoinLobby;
                }
                Err(e) => self.status = format!("Rede indisponível: {}", e),
            },
            SetupKind::OnlineHost => {
                self.room_code.clear();
                self.host = Some(net::Host::new_online());
                self.status.clear();
                self.screen = Screen::OnlineHost;
            }
            SetupKind::OnlineJoin => {
                self.code_text.clear();
                self.status.clear();
                self.screen = Screen::OnlineJoin;
            }
        }
    }

    // -------------------------------------------------------------- settings

    pub(crate) fn settings_screen(&mut self) {
        if self.settings_back == Screen::Match {
            self.draw_world(false);
            self.draw_band();
            dim_screen(with_a(BLACK, 0.55));
        } else {
            self.draw_backdrop();
        }
        text_o("CONFIGURAÇÕES", 640.0, 76.0, 50.0, hex(0xffd34d));
        panel(self, 160.0, 104.0, 960.0, 500.0);
        let rows = ["Efeitos sonoros", "Música", "Tamanho dos botões", "Visibilidade dos botões", "Botões na tela"];
        for (i, label) in rows.iter().enumerate() {
            text(label, 210.0, 168.0 + i as f32 * 76.0, 28.0, WHITE);
        }
        let x = 620.0;
        let w = 450.0;
        let d = stepper(self, x, 132.0, w, self.set.sfx as f32 / 10.0, &format!("{}%", self.set.sfx as u32 * 10));
        self.set.sfx = (self.set.sfx as i32 + d).clamp(0, 10) as u8;
        let d = stepper(self, x, 208.0, w, self.set.music as f32 / 10.0, &format!("{}%", self.set.music as u32 * 10));
        self.set.music = (self.set.music as i32 + d).clamp(0, 10) as u8;
        let span = (BTN_SCALE_MAX - BTN_SCALE_MIN) as f32;
        let d = stepper(self, x, 284.0, w, (self.set.btn_scale - BTN_SCALE_MIN) as f32 / span, &format!("{}%", self.set.btn_scale));
        self.set.btn_scale = (self.set.btn_scale as i32 + d * 10).clamp(BTN_SCALE_MIN as i32, BTN_SCALE_MAX as i32) as u8;
        let d = stepper(self, x, 360.0, w, (self.set.btn_alpha - BTN_ALPHA_MIN) as f32 / (100 - BTN_ALPHA_MIN) as f32, &format!("{}%", self.set.btn_alpha));
        self.set.btn_alpha = (self.set.btn_alpha as i32 + d * 10).clamp(BTN_ALPHA_MIN as i32, 100) as u8;
        // "Nunca" only on the computer: a phone without on-screen buttons could not be played
        let modes: &[&str] = if cfg!(target_os = "android") { &["Automático", "Sempre"] } else { &["Automático", "Sempre", "Nunca"] };
        let cur = (self.set.touch_mode as usize).min(modes.len() - 1);
        if let Some(i) = segmented(self, x, 436.0, w, 54.0, modes, cur) {
            self.set.touch_mode = i as u8;
        }
        if self.button("Mover botões", 210.0, 520.0, 330.0, 62.0, hex(0x2d6fd0)) {
            self.set.save();
            self.screen = Screen::ButtonTest;
            return;
        }
        if self.button("Restaurar padrão", 560.0, 520.0, 330.0, 62.0, hex(0x34405e)) {
            let keep = self.set;
            self.set = crate::settings::Settings::default();
            // the match choices are not settings: keep them
            self.set.look = keep.look;
            self.set.look2 = keep.look2;
            self.set.power = keep.power;
            self.set.stadium = keep.stadium;
            self.set.len_idx = keep.len_idx;
            self.set.level = keep.level;
        }
        self.apply_volumes();
        if self.back_button() {
            self.set.save();
            self.screen = self.settings_back;
        }
    }

    /// The button editor: the match picture with its buttons; drag a button to move it
    /// (saved), touch one to see it pressed.
    pub(crate) fn button_test(&mut self) {
        if self.settings_back != Screen::Match {
            // not over a paused match: show the chosen faces
            self.sim.p[0].look = self.set.look;
            self.sim.p[1].look = self.set.look2;
        }
        self.draw_world(false);
        self.draw_band();
        dim_screen(with_a(BLACK, 0.25));
        let (l, t, r, b) = screen_box();
        rrect(250.0, t + 114.0, 780.0, 96.0, 20.0, with_a(hex(0x0b1430), 0.78));
        text_o("Arraste os botões para onde quiser", 640.0, t + 154.0, 34.0, WHITE);
        text_o(&format!("Tamanho {}%  -  Visibilidade {}%  (mude nas configurações)", self.set.btn_scale, self.set.btn_alpha), 640.0, t + 192.0, 22.0, hex(0xcfe3ff));
        let layout = touch_layout(Mode::Cpu, &self.set);
        let (mx, my) = to_virtual(mouse_position().0, mouse_position().1);
        if is_mouse_button_pressed(MouseButton::Left) {
            self.drag = layout.iter().find(|bt| bt.hit(mx, my)).map(|bt| (bt.kind.slot(), mx - bt.x, my - bt.y));
        }
        if !is_mouse_button_down(MouseButton::Left) {
            self.drag = None;
        }
        if let Some((slot, dx, dy)) = self.drag {
            if let Some(bt) = layout.iter().find(|bt| bt.kind.slot() == slot) {
                let x = (mx - dx).clamp(l + bt.w * 0.5, r - bt.w * 0.5);
                let y = (my - dy).clamp(t + bt.h * 0.5, b - bt.h * 0.5);
                self.set.btn_pos[slot] = screen_frac(x, y);
            }
        }
        let layout = touch_layout(Mode::Cpu, &self.set);
        let mut held = Input::default();
        let pts = pointers();
        for bt in &layout {
            if pts.iter().any(|&(x, y)| bt.hit(x, y)) {
                set_btn(&mut held, bt.kind);
            }
        }
        self.held = [held, Input::default()];
        // show the power button lit, without touching a paused match's power bar
        let power = self.sim.p[0].power;
        self.sim.p[0].power = 1.0;
        self.draw_buttons_for(Mode::Cpu);
        self.sim.p[0].power = power;
        if let Some((slot, ..)) = self.drag {
            if let Some(bt) = layout.iter().find(|bt| bt.kind.slot() == slot) {
                rect_line(bt.x - bt.w * 0.5 - 5.0, bt.y - bt.h * 0.5 - 5.0, bt.w + 10.0, bt.h + 10.0, 3.0, hex(0xffd34d));
            }
        }
        if self.button("Pronto", 400.0, t + 34.0, 220.0, 62.0, hex(0x2f8f4e)) || back_pressed() {
            self.drag = None;
            self.set.save();
            self.screen = Screen::Settings;
            return;
        }
        if self.button("Posição original", 640.0, t + 34.0, 260.0, 62.0, hex(0x34405e)) {
            self.set.btn_pos = [[0; 2]; 6];
        }
    }

    // -------------------------------------------------------------- online rooms

    pub(crate) fn online_host_lobby(&mut self) {
        self.draw_backdrop();
        let now = self.now;
        let joined = self.host.as_mut().map(|h| h.poll(now)).unwrap_or(false);
        let link = self.host.as_ref().and_then(|h| h.link());
        text_o("SALA ONLINE", 640.0, 86.0, 54.0, hex(0xffd34d));
        panel(self, 290.0, 120.0, 700.0, 420.0);
        // the code exists once the broker answered (its first digit names the broker)
        if let Some(c) = self.host.as_ref().map(|h| h.room_code()) {
            if !c.is_empty() {
                self.room_code = c;
            }
        }
        let has_code = self.room_code.len() == 6;
        text_o("Código da sala", 640.0, 175.0, 28.0, hex(0xcfe3ff));
        let code = if has_code { format!("{} {}", &self.room_code[..3], &self.room_code[3..]) } else { "- - -  - - -".to_string() };
        text_o(&code, 640.0, 270.0, 96.0, if has_code { WHITE } else { with_a(WHITE, 0.35) });
        let (msg, col) = match &link {
            Some(crate::relay::LinkState::Connecting) => ("Conectando ao servidor...".to_string(), hex(0xcfe3ff)),
            Some(crate::relay::LinkState::Ready) => {
                let dots = ".".repeat(((now * 2.0) as usize) % 4);
                (format!("Aguardando o seu amigo{}", dots), WHITE)
            }
            Some(crate::relay::LinkState::Failed(e)) => (format!("Sem conexão com o servidor: {}", e), hex(0xff8080)),
            None => (String::new(), WHITE),
        };
        text_o(&msg, 640.0, 330.0, 26.0, col);
        text_o("Mande este código para o seu amigo.", 640.0, 385.0, 22.0, hex(0xb8c6e6));
        text_o("No jogo dele: Multijogador > Entrar com código.", 640.0, 415.0, 22.0, hex(0xb8c6e6));
        let failed = matches!(link, Some(crate::relay::LinkState::Failed(_)));
        if has_code && self.button("Copiar código", if failed { 360.0 } else { 505.0 }, 450.0, 270.0, 62.0, hex(0x2d6fd0)) {
            platform::copy_text(&self.room_code);
            self.status = "Código copiado".into();
        }
        if failed && self.button("Tentar de novo", if has_code { 650.0 } else { 505.0 }, 450.0, 270.0, 62.0, hex(0xd06a2d)) {
            self.leave_network();
            self.room_code.clear();
            self.status.clear();
            self.host = Some(net::Host::new_online());
        }
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 590.0, 22.0, hex(0x9be2a0));
        }
        if joined || self.host.as_ref().map(|h| h.connected(now)).unwrap_or(false) {
            self.status.clear();
            self.start_match(Mode::Host);
            return;
        }
        if self.back_button() {
            self.leave_network();
            self.status.clear();
            self.screen = Screen::Multi;
        }
    }

    pub(crate) fn online_join(&mut self) {
        self.draw_backdrop();
        let now = self.now;
        text_o("ENTRAR NA SALA ONLINE", 640.0, 80.0, 48.0, hex(0xffd34d));
        let connecting = self.client.is_some();
        if let Some(c) = self.client.as_mut() {
            c.poll(now);
        }
        let (welcomed, has_state, link) = match &self.client {
            Some(c) => (c.welcomed, c.latest.is_some(), c.link()),
            None => (false, false, None),
        };
        if welcomed && has_state {
            self.status.clear();
            self.start_match(Mode::Client);
            if let Some(s) = self.client.as_ref().and_then(|c| c.latest.clone()) {
                self.sim = s;
            }
            return;
        }
        panel(self, 390.0, 110.0, 500.0, 500.0);
        text_o("Código da sala (6 números)", 640.0, 150.0, 24.0, hex(0xcfe3ff));
        // six boxes
        for i in 0..6 {
            let x = 448.0 + i as f32 * 66.0;
            rrect(x, 168.0, 56.0, 70.0, 10.0, hex(0x10162c));
            if let Some(ch) = self.code_text.chars().nth(i) {
                text_o(&ch.to_string(), x + 28.0, 220.0, 46.0, WHITE);
            }
        }
        if !connecting {
            while let Some(ch) = get_char_pressed() {
                if ch.is_ascii_digit() && self.code_text.len() < 6 {
                    self.code_text.push(ch);
                }
            }
            if is_key_pressed(KeyCode::Backspace) {
                self.code_text.pop();
            }
            let keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "<", "0", "OK"];
            for (i, k) in keys.iter().enumerate() {
                let x = 425.0 + (i % 3) as f32 * 148.0;
                let y = 256.0 + (i / 3) as f32 * 64.0;
                let col = if *k == "OK" { hex(0x2f8f4e) } else { hex(0x34405e) };
                if self.button(k, x, y, 134.0, 54.0, col) {
                    match *k {
                        "<" => {
                            self.code_text.pop();
                        }
                        "OK" => {}
                        d => {
                            if self.code_text.len() < 6 {
                                self.code_text.push_str(d);
                            }
                        }
                    }
                    if *k == "OK" {
                        self.join_online_room();
                    }
                }
            }
            if is_key_pressed(KeyCode::Enter) {
                self.join_online_room();
            }
        } else {
            let msg = match &link {
                Some(crate::relay::LinkState::Connecting) => "Conectando ao servidor...".to_string(),
                Some(crate::relay::LinkState::Ready) => {
                    let dots = ".".repeat(((now * 2.0) as usize) % 4);
                    if now - self.connect_t > 10.0 {
                        "A sala não respondeu. Confira o código.".to_string()
                    } else {
                        format!("Procurando a sala{}", dots)
                    }
                }
                Some(crate::relay::LinkState::Failed(e)) => format!("Sem conexão com o servidor ({})", e),
                None => String::new(),
            };
            text_o(&msg, 640.0, 330.0, 26.0, WHITE);
            if self.button("Cancelar", 520.0, 420.0, 240.0, 62.0, hex(0x5a2d3a)) {
                self.leave_network();
            }
        }
        if !self.status.is_empty() {
            text_o(&self.status, 640.0, 590.0, 22.0, hex(0xff8080));
        }
        if self.back_button() {
            self.leave_network();
            self.status.clear();
            self.screen = Screen::Multi;
        }
    }

    fn join_online_room(&mut self) {
        if self.code_text.len() != 6 {
            self.status = "Digite os 6 números do código".into();
            return;
        }
        let mut c = net::Client::new_online(&self.code_text);
        c.pkind = self.pick_power();
        c.look = self.set.look;
        self.client = Some(c);
        self.connect_t = self.now;
        self.status.clear();
    }
}

/// In matches against the computer and on one device both players need different faces.
fn look_eq_fix(s: &mut crate::settings::Settings) -> bool {
    if s.look2 == s.look {
        s.look2 = (s.look + 1) % 6;
        return true;
    }
    false
}
