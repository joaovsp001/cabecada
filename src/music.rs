//! Background music for the menus. If `assets/music/menu.ogg` (or `.wav`) exists it is
//! used; otherwise a short cheerful loop is made by code at start-up (bass, plucked
//! chords and drums, 16 seconds, loops seamlessly).

const RATE: u32 = 22050;
const BPM: f32 = 120.0;

/// WAV bytes of the menu music loop.
pub fn menu_track() -> Vec<u8> {
    if let Some(bytes) = crate::assets::music_bytes("menu") {
        return bytes.to_vec();
    }
    wav(&synth_loop())
}

fn note_hz(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0)
}

/// 8 bars of 4 beats: C - G - Am - F - C - G - F - G.
fn synth_loop() -> Vec<f32> {
    let beat = 60.0 / BPM;
    let bars = 8;
    let total = (bars as f32 * 4.0 * beat * RATE as f32) as usize;
    let mut out = vec![0.0f32; total];
    // chord roots (MIDI) and triads
    let chords: [(i32, [i32; 3]); 8] = [
        (48, [60, 64, 67]), // C
        (43, [59, 62, 67]), // G
        (45, [60, 64, 69]), // Am
        (41, [60, 65, 69]), // F
        (48, [60, 64, 67]), // C
        (43, [59, 62, 67]), // G
        (41, [60, 65, 69]), // F
        (43, [59, 62, 67]), // G
    ];
    let mut noise = 0x2468ace1u32;
    let mut rnd = move || {
        noise ^= noise << 13;
        noise ^= noise >> 17;
        noise ^= noise << 5;
        (noise as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    let add = |out: &mut Vec<f32>, start: f32, len: f32, f: &mut dyn FnMut(f32, f32) -> f32| {
        let s0 = (start * RATE as f32) as usize;
        let n = (len * RATE as f32) as usize;
        for i in 0..n {
            let idx = (s0 + i) % out.len(); // wraps, so the loop has no seam
            let t = i as f32 / RATE as f32;
            out[idx] += f(t, i as f32 / n as f32);
        }
    };
    use std::f32::consts::TAU;
    for (bar, (root, triad)) in chords.iter().enumerate() {
        let bar_t = bar as f32 * 4.0 * beat;
        // bass: eighth notes, root and octave
        for e in 0..8 {
            let midi = if e % 4 == 2 { root + 12 } else { *root };
            let hz = note_hz(midi);
            add(&mut out, bar_t + e as f32 * beat * 0.5, beat * 0.45, &mut |t, k| {
                let tri = 2.0 * (2.0 * (t * hz).fract() - 1.0).abs() - 1.0;
                tri * 0.22 * (1.0 - k).powf(0.6)
            });
        }
        // plucked arpeggio: sixteenth notes over the chord
        let pattern = [0usize, 1, 2, 1, 0, 2, 1, 2];
        for s in 0..16 {
            let midi = triad[pattern[s % 8]] + if s >= 8 { 12 } else { 0 };
            let hz = note_hz(midi);
            add(&mut out, bar_t + s as f32 * beat * 0.25, beat * 0.5, &mut |t, k| {
                let sq = if (t * hz).fract() < 0.5 { 1.0 } else { -1.0 };
                let sine = (TAU * hz * t).sin();
                (sq * 0.35 + sine * 0.65) * 0.07 * (1.0 - k).powi(3)
            });
        }
        // drums: kick on 1 and 3, snare on 2 and 4, hi-hat on eighths
        for b in 0..4 {
            let bt = bar_t + b as f32 * beat;
            if b % 2 == 0 {
                add(&mut out, bt, 0.16, &mut |t, k| (TAU * (110.0 - 70.0 * k) * t).sin() * 0.45 * (1.0 - k).powi(2));
            } else {
                add(&mut out, bt, 0.14, &mut |t, k| (rnd() * 0.6 + (TAU * 190.0 * t).sin() * 0.3) * 0.22 * (1.0 - k).powi(3));
            }
            for h in 0..2 {
                add(&mut out, bt + h as f32 * beat * 0.5, 0.04, &mut |_, k| rnd() * 0.06 * (1.0 - k));
            }
        }
    }
    // gentle limiter
    let peak = out.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(0.001);
    let gain = 0.85 / peak;
    out.iter().map(|v| (v * gain).tanh() * 0.9).collect()
}

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
