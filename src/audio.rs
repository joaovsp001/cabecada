//! Sounds made from simple waves and noise at start-up (no sound files).

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

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
        })
    }

    pub fn play(&self, s: &Sound, vol: f32) {
        play_sound(s, PlaySoundParams { looped: false, volume: vol });
    }
}
