//! What the player chose: volumes, button size, last match choices. Saved in a small
//! text file (one `name=value` per line) in the app's own folder, so it survives a
//! restart. A missing or broken file just gives the defaults.

use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub sfx: u8,        // sound effects volume, 0..=10
    pub music: u8,      // music volume, 0..=10
    pub btn_scale: u8,  // on-screen button size in percent, 70..=150
    pub btn_alpha: u8,  // on-screen button opacity in percent, 30..=100
    pub touch_mode: u8, // on-screen buttons: 0 automatic, 1 always, 2 never
    pub look: u8,       // your character
    pub look2: u8,      // the opponent's character (computer / second player)
    pub power: u8,      // your power, POWER_KINDS = random
    pub stadium: u8,
    pub len_idx: u8,    // match length (MATCH_LENGTHS index)
    pub level: u8,      // computer level 0..=2
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { sfx: 8, music: 5, btn_scale: 100, btn_alpha: 90, touch_mode: 0, look: 0, look2: 1, power: 6, stadium: 0, len_idx: 1, level: 1 }
    }
}

pub const BTN_SCALE_MIN: u8 = 70;
pub const BTN_SCALE_MAX: u8 = 150;
pub const BTN_ALPHA_MIN: u8 = 30;

impl Settings {
    fn fields(&mut self) -> [(&'static str, &mut u8); 11] {
        [
            ("sfx", &mut self.sfx),
            ("music", &mut self.music),
            ("btn_scale", &mut self.btn_scale),
            ("btn_alpha", &mut self.btn_alpha),
            ("touch_mode", &mut self.touch_mode),
            ("look", &mut self.look),
            ("look2", &mut self.look2),
            ("power", &mut self.power),
            ("stadium", &mut self.stadium),
            ("len_idx", &mut self.len_idx),
            ("level", &mut self.level),
        ]
    }

    /// Keeps every value in its allowed range (a hand-edited file cannot break the game).
    pub fn clamp(&mut self) {
        self.sfx = self.sfx.min(10);
        self.music = self.music.min(10);
        self.btn_scale = self.btn_scale.clamp(BTN_SCALE_MIN, BTN_SCALE_MAX);
        self.btn_alpha = self.btn_alpha.clamp(BTN_ALPHA_MIN, 100);
        self.touch_mode = self.touch_mode.min(2);
        self.look %= 6;
        self.look2 %= 6;
        self.power = self.power.min(6);
        self.stadium %= 3;
        self.len_idx = self.len_idx.min(2);
        self.level = self.level.min(2);
    }

    pub fn load() -> Settings {
        let text = file().and_then(|f| std::fs::read_to_string(f).ok()).unwrap_or_default();
        Settings::from_text(&text)
    }

    pub fn save(&self) {
        let Some(path) = file() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, self.to_text());
    }

    /// Reads `name=value` lines; unknown names and bad values are skipped.
    fn from_text(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if let Ok(v) = v.trim().parse::<u8>() {
                    for (name, field) in s.fields() {
                        if name == k.trim() {
                            *field = v;
                        }
                    }
                }
            }
        }
        s.clamp();
        s
    }

    fn to_text(&self) -> String {
        let mut copy = *self;
        copy.fields().iter().map(|(k, v)| format!("{}={}
", k, v)).collect()
    }
}

/// Where the settings file lives: the app's private folder on Android, the user's
/// application-data folder on the computer.
fn file() -> Option<PathBuf> {
    Some(crate::platform::data_dir()?.join("settings.txt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_settings_come_back() {
        let s = Settings { sfx: 3, music: 0, btn_scale: 120, btn_alpha: 60, touch_mode: 1, look: 4, look2: 2, power: 3, stadium: 2, len_idx: 0, level: 2 };
        assert_eq!(Settings::from_text(&s.to_text()), s);
    }

    #[test]
    fn broken_files_give_safe_values() {
        let s = Settings::from_text("sfx=99
btn_scale=5
look=abc
nonsense
level=7");
        assert_eq!(s.sfx, 10);
        assert_eq!(s.btn_scale, BTN_SCALE_MIN);
        assert_eq!(s.look, Settings::default().look);
        assert_eq!(s.level, 2);
        assert_eq!(Settings::from_text(""), Settings::default());
    }
}
