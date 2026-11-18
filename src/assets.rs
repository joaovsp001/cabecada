//! Embedded 2D game textures and font (compiled into the binary so Windows and Android builds
//! work without external asset paths).

use macroquad::prelude::*;

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/game.ttf");

pub struct Assets {
    pub stadium_bgs: [Texture2D; 3],
    pub menu_bg: Texture2D,
    pub logo: Texture2D,
    pub trophy: Texture2D,
    pub heads: [Texture2D; 6],
    pub boots: [Texture2D; 6],
    pub ball: Texture2D,
    pub ball_fire: Texture2D,
    pub ball_ice: Texture2D,
    pub ball_bomb: Texture2D,
    pub ice_block: Texture2D,
    pub boom: [Texture2D; 8],
    pub fx_dust: Texture2D,
    pub fx_kick: Texture2D,
    pub confetti: Texture2D,
    pub goal_left: Texture2D,
    pub scoreboard: Texture2D,
    pub btn_left: Texture2D,
    pub btn_kick: Texture2D,
    pub btn_lob: Texture2D,
    pub btn_jump: Texture2D,
    pub btn_powers: [Texture2D; 6],
    pub btn_menu_green: Texture2D,
    pub btn_menu_blue: Texture2D,
    pub btn_menu_orange: Texture2D,
    pub btn_menu_purple: Texture2D,
    pub btn_menu_gold: Texture2D,
    pub btn_menu_red: Texture2D,
    pub btn_menu_gray: Texture2D,
}

fn load_png(bytes: &'static [u8]) -> Texture2D {
    let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
    tex.set_filter(FilterMode::Linear);
    tex
}

impl Assets {
    pub fn load() -> Assets {
        Assets {
            stadium_bgs: [
                load_png(include_bytes!("../assets/stadium_bg.png")),
                load_png(include_bytes!("../assets/stadium_bg2.png")),
                load_png(include_bytes!("../assets/stadium_bg3.png")),
            ],
            menu_bg: load_png(include_bytes!("../assets/menu_bg.png")),
            logo: load_png(include_bytes!("../assets/logo_cabecada.png")),
            trophy: load_png(include_bytes!("../assets/trophy.png")),
            heads: [
                load_png(include_bytes!("../assets/p1_head.png")),
                load_png(include_bytes!("../assets/p2_head.png")),
                load_png(include_bytes!("../assets/p3_head.png")),
                load_png(include_bytes!("../assets/p4_head.png")),
                load_png(include_bytes!("../assets/p5_head.png")),
                load_png(include_bytes!("../assets/p6_head.png")),
            ],
            boots: [
                load_png(include_bytes!("../assets/p1_boot.png")),
                load_png(include_bytes!("../assets/p2_boot.png")),
                load_png(include_bytes!("../assets/p3_boot.png")),
                load_png(include_bytes!("../assets/p4_boot.png")),
                load_png(include_bytes!("../assets/p5_boot.png")),
                load_png(include_bytes!("../assets/p6_boot.png")),
            ],
            ball: load_png(include_bytes!("../assets/ball.png")),
            ball_fire: load_png(include_bytes!("../assets/ball_fire.png")),
            ball_ice: load_png(include_bytes!("../assets/ball_ice.png")),
            ball_bomb: load_png(include_bytes!("../assets/ball_bomb.png")),
            ice_block: load_png(include_bytes!("../assets/ice_block.png")),
            boom: [
                load_png(include_bytes!("../assets/boom_00.png")),
                load_png(include_bytes!("../assets/boom_01.png")),
                load_png(include_bytes!("../assets/boom_02.png")),
                load_png(include_bytes!("../assets/boom_03.png")),
                load_png(include_bytes!("../assets/boom_04.png")),
                load_png(include_bytes!("../assets/boom_05.png")),
                load_png(include_bytes!("../assets/boom_06.png")),
                load_png(include_bytes!("../assets/boom_07.png")),
            ],
            fx_dust: load_png(include_bytes!("../assets/fx_dust.png")),
            fx_kick: load_png(include_bytes!("../assets/fx_kick.png")),
            confetti: load_png(include_bytes!("../assets/confetti.png")),
            goal_left: load_png(include_bytes!("../assets/goal_left.png")),
            scoreboard: load_png(include_bytes!("../assets/scoreboard.png")),
            btn_left: load_png(include_bytes!("../assets/btn_left.png")),
            btn_kick: load_png(include_bytes!("../assets/btn_kick.png")),
            btn_lob: load_png(include_bytes!("../assets/btn_lob.png")),
            btn_jump: load_png(include_bytes!("../assets/btn_jump.png")),
            btn_powers: [
                load_png(include_bytes!("../assets/btn_power_fire.png")),
                load_png(include_bytes!("../assets/btn_power_big.png")),
                load_png(include_bytes!("../assets/btn_power_ice.png")),
                load_png(include_bytes!("../assets/btn_power_invis.png")),
                load_png(include_bytes!("../assets/btn_power_bomb.png")),
                load_png(include_bytes!("../assets/btn_power_clone.png")),
            ],
            btn_menu_green: load_png(include_bytes!("../assets/btn_menu_green.png")),
            btn_menu_blue: load_png(include_bytes!("../assets/btn_menu_blue.png")),
            btn_menu_orange: load_png(include_bytes!("../assets/btn_menu_orange.png")),
            btn_menu_purple: load_png(include_bytes!("../assets/btn_menu_purple.png")),
            btn_menu_gold: load_png(include_bytes!("../assets/btn_menu_gold.png")),
            btn_menu_red: load_png(include_bytes!("../assets/btn_menu_red.png")),
            btn_menu_gray: load_png(include_bytes!("../assets/btn_menu_gray.png")),
        }
    }
}
