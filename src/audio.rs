//! Sounds made from simple waves and noise at start-up (no sound files).

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

const RATE: u32 = 22050;

fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&RATE.to_le_bytes());
    v.extend_from_slice(&(RATE * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let x = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        v.extend_from_slice(&x.to_le_bytes());
    }
    v
}

struct Noise(u32);
impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn gen(secs: f32, f: impl Fn(f32, f32, &mut Noise) -> f32) -> Vec<u8> {
    let n = (secs * RATE as f32) as usize;
    let mut nz = Noise(0xdeadbeef);
    let s: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            f(t, t / secs, &mut nz)
        })
        .collect();
    wav(&s)
}

use std::f32::consts::TAU;

/// A few seconds of soft crowd murmur (low-passed noise that swells and fades), made to be looped.
fn crowd_wave() -> Vec<u8> {
    let secs = 4.0f32;
    let n = (secs * RATE as f32) as usize;
    let mut nz = Noise(0x1234abcd);
    let (mut lp1, mut lp2) = (0.0f32, 0.0f32);
    let s: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let k = t / secs;
            lp1 += (nz.next() - lp1) * 0.12;
            lp2 += (lp1 - lp2) * 0.25;
            let swell = 0.75 + 0.25 * (TAU * 0.5 * t).sin();
            let edge = (k / 0.03).min(1.0) * ((1.0 - k) / 0.03).min(1.0);
            lp2 * 2.2 * swell * edge
        })
        .collect();
    wav(&s)
}

pub struct Sounds {
    pub kick: Sound,
    pub head: Sound,
    pub bounce: Sound,
    pub post: Sound,
    pub whistle: Sound,
    pub goal: Sound,
    pub beep: Sound,
    pub power: Sound,
    pub stun: Sound,
    pub jump: Sound,
    pub land: Sound,
    pub crowd: Sound,
    pub ice_cast: Sound,
    pub ice_crack: Sound,
    pub ice_shatter: Sound,
    pub power_pop: Sound,
}

impl Sounds {
    pub async fn load() -> Option<Sounds> {
        let kick = gen(0.14, |t, k, n| {
            let f = 150.0 - 90.0 * k;
            ((TAU * f * t).sin() * 0.9 + n.next() * 0.35 * (1.0 - k).powi(4)) * (1.0 - k).powi(2)
        });
        let head = gen(0.12, |t, k, n| ((TAU * (260.0 - 120.0 * k) * t).sin() * 0.7 + n.next() * 0.2) * (1.0 - k).powi(2));
        let bounce = gen(0.10, |t, k, _| (TAU * (110.0 - 40.0 * k) * t).sin() * 0.5 * (1.0 - k).powi(2));
        let post = gen(0.5, |t, k, _| {
            ((TAU * 880.0 * t).sin() * 0.5 + (TAU * 1320.0 * t).sin() * 0.3 + (TAU * 2210.0 * t).sin() * 0.15) * (1.0 - k).powi(3)
        });
        let whistle = gen(0.6, |t, k, n| {
            let vib = 1.0 + 0.04 * (TAU * 28.0 * t).sin();
            let env = if k < 0.05 { k / 0.05 } else if k > 0.85 { (1.0 - k) / 0.15 } else { 1.0 };
            ((TAU * 2300.0 * vib * t).sin() * 0.45 + n.next() * 0.05) * env
        });
        let goal = gen(1.8, |t, k, n| {
            // crowd roar: filtered noise swelling and fading
            let env = (k * 4.0).min(1.0) * (1.0 - k).powf(0.7);
            (n.next() * 0.45 + (TAU * 180.0 * t).sin() * 0.05) * env
        });
        let beep = gen(0.12, |t, k, _| (TAU * 880.0 * t).sin() * 0.4 * (1.0 - k));
        let power = gen(0.5, |t, k, n| ((TAU * (300.0 + 900.0 * k) * t).sin() * 0.4 + n.next() * 0.15) * (1.0 - k * 0.6));
        let stun = gen(0.3, |t, k, _| (TAU * (700.0 - 400.0 * k) * t).sin() * 0.35 * (1.0 - k));
        let jump = gen(0.12, |t, k, _| (TAU * (300.0 + 320.0 * k) * t).sin() * 0.4 * (1.0 - k).powi(2));
        let land = gen(0.10, |t, k, n| ((TAU * (95.0 - 30.0 * k) * t).sin() * 0.6 + n.next() * 0.25) * (1.0 - k).powi(3));
        let crowd = crowd_wave();
        let ice_cast = gen(0.48, |t, k, n| {
            // crystalline arctic freeze shimmer + frosty wind blast
            let env = (k / 0.04).min(1.0) * (1.0 - k).powf(1.4);
            let sweep = 1400.0 + 2200.0 * (1.0 - (1.0 - k).powi(2));
            let shimmer = (TAU * sweep * t).sin() * 0.35
                + (TAU * (sweep * 1.498) * t).sin() * 0.25
                + (TAU * (3100.0 + 400.0 * (TAU * 18.0 * t).sin()) * t).sin() * 0.18;
            let frost = n.next() * 0.32 * (1.0 - k).powi(2);
            (shimmer + frost) * env * 0.85
        });
        let ice_crack = gen(0.13, |t, k, n| {
            // sharp brittle fracture snap + high crystalline ping
            let snap = if k < 0.18 { n.next() * (1.0 - k / 0.18) * 0.85 } else { 0.0 };
            let crack2 = if k > 0.22 && k < 0.40 {
                n.next() * (1.0 - (k - 0.22) / 0.18) * 0.55
            } else {
                0.0
            };
            let ping = ((TAU * (2850.0 - 600.0 * k) * t).sin() * 0.45
                + (TAU * 4320.0 * t).sin() * 0.30)
                * (1.0 - k).powi(3);
            (snap + crack2 + ping) * 0.8
        });
        let ice_shatter = gen(0.42, |t, k, n| {
            // crisp shatter burst followed by cascading ice shard tinkles
            let burst = n.next() * 0.7 * (1.0 - k).powi(4);
            let c1 = (TAU * 3400.0 * t).sin() * (1.0 - k).powi(2);
            let c2 = if k > 0.06 {
                let u = (k - 0.06) / 0.94;
                (TAU * 4850.0 * t).sin() * (1.0 - u).powi(3)
            } else {
                0.0
            };
            let c3 = if k > 0.14 {
                let u = (k - 0.14) / 0.86;
                (TAU * 6100.0 * t).sin() * (1.0 - u).powi(3)
            } else {
                0.0
            };
            let c4 = if k > 0.22 {
                let u = (k - 0.22) / 0.78;
                (TAU * 2650.0 * t).sin() * (1.0 - u).powi(2)
            } else {
                0.0
            };
            let tinkle = (c1 * 0.35 + c2 * 0.30 + c3 * 0.22 + c4 * 0.22) * (0.7 + 0.3 * n.next().abs());
            (burst + tinkle) * 0.85
        });
        let power_pop = gen(0.24, |t, k, n| {
            // bouncy arcade pop/sparkle chime for Big / Invis / Clone activation
            let env = (k / 0.03).min(1.0) * (1.0 - k).powf(1.6);
            let f = 240.0 + 740.0 * k.sqrt();
            let pop = (TAU * f * t).sin() * 0.65 + (TAU * (f * 2.0) * t).sin() * 0.25;
            let spark = n.next() * 0.18 * (1.0 - k).powi(4);
            (pop + spark) * env * 0.85
        });
        Some(Sounds {
            kick: load_sound_from_bytes(&kick).await.ok()?,
            head: load_sound_from_bytes(&head).await.ok()?,
            bounce: load_sound_from_bytes(&bounce).await.ok()?,
            post: load_sound_from_bytes(&post).await.ok()?,
            whistle: load_sound_from_bytes(&whistle).await.ok()?,
            goal: load_sound_from_bytes(&goal).await.ok()?,
            beep: load_sound_from_bytes(&beep).await.ok()?,
            power: load_sound_from_bytes(&power).await.ok()?,
            stun: load_sound_from_bytes(&stun).await.ok()?,
            jump: load_sound_from_bytes(&jump).await.ok()?,
            land: load_sound_from_bytes(&land).await.ok()?,
            crowd: load_sound_from_bytes(&crowd).await.ok()?,
            ice_cast: load_sound_from_bytes(&ice_cast).await.ok()?,
            ice_crack: load_sound_from_bytes(&ice_crack).await.ok()?,
            ice_shatter: load_sound_from_bytes(&ice_shatter).await.ok()?,
            power_pop: load_sound_from_bytes(&power_pop).await.ok()?,
        })
    }

    pub fn play(&self, s: &Sound, vol: f32) {
        play_sound(s, PlaySoundParams { looped: false, volume: vol });
    }

    pub fn play_loop(&self, s: &Sound, vol: f32) {
        play_sound(s, PlaySoundParams { looped: true, volume: vol });
    }

    pub fn stop(&self, s: &Sound) {
        stop_sound(s);
    }
}
