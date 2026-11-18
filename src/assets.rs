//! Embedded 2D game textures (compiled into the binary so Windows and Android builds
//! work without external asset paths).

use macroquad::prelude::*;

pub struct Assets {
    pub stadium_bg: Texture2D,
    pub menu_bg: Texture2D,
    pub logo: Texture2D,
    pub p1_head: Texture2D,
    pub p2_head: Texture2D,
    pub p1_boot: Texture2D,
    pub p2_boot: Texture2D,
    pub ball: Texture2D,
    pub ball_fire: Texture2D,
    pub goal_left: Texture2D,
    pub scoreboard: Texture2D,
    pub btn_left: Texture2D,
    pub btn_kick: Texture2D,
    pub btn_lob: Texture2D,
    pub btn_jump: Texture2D,
    pub btn_power: Texture2D,
}

fn load_png(bytes: &'static [u8]) -> Texture2D {
    let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Linear);
    tex
}

impl Assets {
    pub fn load() -> Assets {
        Assets {
            stadium_bg: load_png(include_bytes!("../assets/stadium_bg.png")),
            menu_bg: load_png(include_bytes!("../assets/menu_bg.png")),
            logo: load_png(include_bytes!("../assets/logo_cabecada.png")),
            p1_head: load_png(include_bytes!("../assets/p1_head.png")),
            p2_head: load_png(include_bytes!("../assets/p2_head.png")),
            p1_boot: load_png(include_bytes!("../assets/p1_boot.png")),
            p2_boot: load_png(include_bytes!("../assets/p2_boot.png")),
            ball: load_png(include_bytes!("../assets/ball.png")),
            ball_fire: load_png(include_bytes!("../assets/ball_fire.png")),
            goal_left: load_png(include_bytes!("../assets/goal_left.png")),
            scoreboard: load_png(include_bytes!("../assets/scoreboard.png")),
            btn_left: load_png(include_bytes!("../assets/btn_left.png")),
            btn_kick: load_png(include_bytes!("../assets/btn_kick.png")),
            btn_lob: load_png(include_bytes!("../assets/btn_lob.png")),
            btn_jump: load_png(include_bytes!("../assets/btn_jump.png")),
            btn_power: load_png(include_bytes!("../assets/btn_power.png")),
        }
    }
}
