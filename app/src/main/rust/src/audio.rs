//! Motor de Audio 100% Nativo en Rust para Android (AAudio + DSP Shmup Sci-Fi)
//! Renderizado estéreo PCM de 16 bits a 44100 Hz con osciladores PolyBLEP anti-aliased,
//! transitorios punchy, sintetizador Cyberpunk/Synthwave y enlace nativo directo con AAudio (libaaudio.so).

use std::ffi::{c_char, c_int, c_void, CStr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const SAMPLE_RATE: u32 = 44100;
pub const MAX_ACTIVE_SFX: usize = 8;
pub const BUFFER_FRAMES: usize = 512; // ~11.6 ms latencia de hardware
pub const CHANNELS: usize = 2; // Salida Estéreo

const BPM: f32 = 148.0;
const SECONDS_PER_BEAT: f32 = 60.0 / BPM;
const SECONDS_PER_STEP: f32 = SECONDS_PER_BEAT / 4.0; // Semicorchea (~0.10135s)
const PATTERN_STEPS: usize = 128; // 8 compases (16 pasos por compás)
const PATTERN_DURATION: f32 = (PATTERN_STEPS as f32) * SECONDS_PER_STEP;

#[inline(always)]
fn midi_to_freq(note: u8) -> f32 {
    if note == 0 {
        0.0
    } else {
        440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }
}

// ------------------------------------------------------------------------------
// DSP: Algoritmos PolyBLEP (Band-Limited Step) y Saturación Suave
// ------------------------------------------------------------------------------

#[inline(always)]
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

#[inline(always)]
fn antialiased_pulse(phase: f32, duty: f32, dt: f32) -> f32 {
    let raw = if phase < duty { 1.0 } else { -1.0 };
    let blep1 = poly_blep(phase, dt);
    let blep2 = poly_blep((phase + 1.0 - duty) % 1.0, dt);
    raw + blep1 - blep2
}

#[inline(always)]
fn antialiased_saw(phase: f32, dt: f32) -> f32 {
    let raw = 2.0 * phase - 1.0;
    raw - poly_blep(phase, dt)
}

#[inline(always)]
fn soft_limit(x: f32) -> f32 {
    x / (1.0 + x * x).sqrt()
}

// ------------------------------------------------------------------------------
// Partituras Musicales (8 Compases: Melodía Heroica, Acordes y Galope)
// ------------------------------------------------------------------------------

const LEAD_PATTERN: [u8; 128] = [
    74, 74, 74, 74, 77, 77, 76, 76, 74, 74, 74, 74, 69, 69, 72, 72,
    74, 74, 74, 74, 77, 77, 79, 79, 77, 77, 74, 74, 70, 70, 72, 72,
    76, 76, 76, 76, 79, 79, 81, 81, 79, 79, 76, 76, 72, 72, 74, 74,
    73, 73, 76, 76, 81, 81, 81, 81, 79, 79, 76, 76, 73, 73, 74, 76,
    81, 81, 81, 81, 84, 84, 81, 81, 77, 77, 77, 77, 79, 79, 81, 81,
    83, 83, 83, 83, 86, 86, 83, 83, 79, 79, 79, 79, 81, 81, 83, 83,
    86, 86, 86, 86, 84, 84, 82, 82, 81, 81, 79, 79, 81, 81, 82, 82,
    85, 85, 85, 85, 88, 88, 86, 86, 85, 85, 81, 81, 76, 76, 74, 74,
];

const BASS_PATTERN: [u8; 128] = [
    38, 38, 50, 38, 38, 50, 38, 38, 50, 38, 38, 50, 38, 50, 48, 49,
    34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 46, 36, 37,
    36, 36, 48, 36, 36, 48, 36, 36, 48, 36, 36, 48, 36, 48, 38, 40,
    33, 33, 45, 33, 33, 45, 33, 33, 45, 33, 45, 33, 34, 35, 36, 37,
    29, 29, 41, 29, 29, 41, 29, 29, 41, 29, 29, 41, 29, 41, 31, 32,
    31, 31, 43, 31, 31, 43, 31, 31, 43, 31, 31, 43, 31, 43, 33, 34,
    34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 46, 36, 37,
    33, 33, 45, 33, 45, 33, 45, 33, 45, 45, 45, 45, 45, 46, 47, 49,
];

const ARP_CHORDS: [[u8; 4]; 8] = [
    [62, 65, 69, 74],
    [58, 62, 65, 70],
    [60, 64, 67, 72],
    [57, 61, 64, 69],
    [53, 57, 60, 65],
    [55, 59, 62, 67],
    [58, 62, 65, 70],
    [57, 61, 64, 69],
];

// ------------------------------------------------------------------------------
// Efectos de Sonido
// ------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SoundEffect {
    Laser,
    SpreadFire,
    Explosion,
    PowerUp,
    BombExplosion,
    PlayerHit,
    BossAlarm,
    SatelliteLock,
    EmpShockwave,
    Ricochet,
    MetalClang,
    ThrusterBurst,
}

#[derive(Clone, Debug)]
pub struct SfxInstance {
    pub s_type: SoundEffect,
    pub time: f32,
    pub duration: f32,
    pub pan: f32, // -1.0 (Izquierda) a 1.0 (Derecha)
}

#[derive(Clone, Debug)]
pub struct AudioEngine {
    pub active_sfx: Vec<SfxInstance>,
    pub music_time: f32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
    pub stage_theme: u8,

    // Acumuladores de fase
    lead_phase: f32,
    lead_detune_phase: f32,
    arp_phase: f32,
    bass_phase: f32,
    noise_seed: u32,
    laser_toggle: bool,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            active_sfx: Vec::with_capacity(MAX_ACTIVE_SFX),
            music_time: 0.0,
            music_enabled: true,
            sfx_enabled: true,
            stage_theme: 1,
            lead_phase: 0.0,
            lead_detune_phase: 0.0,
            arp_phase: 0.0,
            bass_phase: 0.0,
            noise_seed: 0x12345678,
            laser_toggle: false,
        }
    }

    #[inline(always)]
    fn next_noise(&mut self) -> f32 {
        self.noise_seed = self.noise_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        ((self.noise_seed >> 16) as f32 / 32768.0) - 1.0
    }

    pub fn reset_for_new_game(&mut self, stage: u8) {
        self.active_sfx.clear();
        self.music_time = 0.0;
        self.stage_theme = stage;
        self.lead_phase = 0.0;
        self.lead_detune_phase = 0.0;
        self.arp_phase = 0.0;
        self.bass_phase = 0.0;
        self.noise_seed = 0x12345678;
        self.laser_toggle = false;
    }

    pub fn play_sfx(&mut self, sfx: SoundEffect) {
        if !self.sfx_enabled {
            return;
        }

        let duration = match sfx {
            SoundEffect::Laser => 0.09,
            SoundEffect::SpreadFire => 0.12,
            SoundEffect::Explosion => 0.35,
            SoundEffect::PowerUp => 0.40,
            SoundEffect::BombExplosion => 0.95,
            SoundEffect::PlayerHit => 0.22,
            SoundEffect::BossAlarm => 1.05,
            SoundEffect::SatelliteLock => 0.16,
            SoundEffect::EmpShockwave => 0.95,
            SoundEffect::Ricochet => 0.12,
            SoundEffect::MetalClang => 0.14,
            SoundEffect::ThrusterBurst => 0.15,
        };

        let pan = match sfx {
            SoundEffect::Laser => {
                self.laser_toggle = !self.laser_toggle;
                if self.laser_toggle { -0.15 } else { 0.15 }
            }
            SoundEffect::Ricochet => -0.25,
            SoundEffect::SatelliteLock => 0.25,
            _ => 0.0,
        };

        // Deduplicación de voces para evitar sobrecarga y retrasos en audio:
        // Disparos rápidos reinician la instancia existente en lugar de apilar voces concurrentes
        match sfx {
            SoundEffect::Laser | SoundEffect::SpreadFire | SoundEffect::Ricochet | SoundEffect::MetalClang => {
                if let Some(existing) = self.active_sfx.iter_mut().find(|s| s.s_type == sfx) {
                    existing.time = 0.0;
                    existing.pan = pan;
                    return;
                }
            }
            SoundEffect::Explosion => {
                let exp_count = self.active_sfx.iter().filter(|s| s.s_type == SoundEffect::Explosion).count();
                if exp_count >= 2 {
                    if let Some(oldest) = self.active_sfx.iter_mut().filter(|s| s.s_type == SoundEffect::Explosion).max_by(|a, b| {
                        a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                        oldest.time = 0.0;
                        oldest.pan = pan;
                        return;
                    }
                }
            }
            _ => {}
        }

        let new_instance = SfxInstance {
            s_type: sfx,
            time: 0.0,
            duration,
            pan,
        };

        if self.active_sfx.len() < MAX_ACTIVE_SFX {
            self.active_sfx.push(new_instance);
        } else if let Some(oldest) = self.active_sfx.iter_mut().max_by(|a, b| {
            let pa = a.time / a.duration.max(0.001);
            let pb = b.time / b.duration.max(0.001);
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            *oldest = new_instance;
        }
    }

    /// Renderiza muestras estéreo entrelazadas [L0, R0, L1, R1, ...] a 44100 Hz.
    pub fn render_samples_stereo(&mut self, buffer: &mut [i16]) {
        let dt = 1.0 / (SAMPLE_RATE as f32);
        let num_frames = buffer.len() / 2;

        for frame_idx in 0..num_frames {
            let mut mix_l = 0.0f32;
            let mut mix_r = 0.0f32;

            // 1. Música Cyberpunk / Synthwave Shmup
            if self.music_enabled {
                self.music_time += dt;
                if self.music_time >= PATTERN_DURATION {
                    self.music_time -= PATTERN_DURATION;
                }
                let noise_val = self.next_noise();

                let total_steps = (self.music_time / SECONDS_PER_STEP) as usize;
                let step_idx = total_steps % PATTERN_STEPS;
                let bar_idx = (step_idx / 16) % 8;
                let step_time = self.music_time - (total_steps as f32 * SECONDS_PER_STEP);
                let step_progress = step_time / SECONDS_PER_STEP;

                let theme_transpose: i16 = match (self.stage_theme.saturating_sub(1)) % 4 {
                    0 => 0,  // D minor (Stage 1 / 5)
                    1 => 2,  // E minor (Stage 2 / 6)
                    2 => 5,  // G minor (Stage 3 / 7)
                    _ => -2, // C minor (Stage 4 / 8)
                };

                // --- Lead con Unison Detune y PolyBLEP ---
                let lead_midi = (LEAD_PATTERN[step_idx] as i16 + theme_transpose).clamp(1, 127) as u8;
                let mut lead_freq = midi_to_freq(lead_midi);
                if step_progress > 0.40 {
                    lead_freq += (self.music_time * 5.8 * std::f32::consts::TAU).sin() * 3.8;
                }
                self.lead_phase = (self.lead_phase + lead_freq * dt) % 1.0;
                self.lead_detune_phase = (self.lead_detune_phase + lead_freq * 1.0028 * dt) % 1.0;

                let lead_osc1 = antialiased_pulse(self.lead_phase, 0.50, lead_freq * dt);
                let lead_osc2 = antialiased_saw(self.lead_detune_phase, lead_freq * 1.0028 * dt);
                let lead_attack = (step_time / 0.012).min(1.0);
                let lead_gate = if step_progress < 0.88 { 1.0 } else { (1.0 - step_progress) / 0.12 };
                let lead_tone = (lead_osc1 * 0.65 + lead_osc2 * 0.35) * lead_attack * lead_gate * 0.16;

                mix_l += lead_tone * 0.90;
                mix_r += lead_tone * 1.10;

                // --- Arpegios / Acordes Espaciales en Estéreo ---
                let sub_tick = ((step_progress * 4.0) as usize).min(3);
                let arp_midi = (ARP_CHORDS[bar_idx][sub_tick] as i16 + theme_transpose).clamp(1, 127) as u8;
                let arp_freq = midi_to_freq(arp_midi);
                self.arp_phase = (self.arp_phase + arp_freq * dt) % 1.0;
                let arp_pulse = antialiased_pulse(self.arp_phase, 0.35, arp_freq * dt);
                let arp_tone = arp_pulse * 0.08;
                let pan_arp = if sub_tick % 2 == 0 { 0.25 } else { -0.25 };
                mix_l += arp_tone * (1.0 - pan_arp);
                mix_r += arp_tone * (1.0 + pan_arp);

                // --- Driving Cyberpunk Bassline (Doble Capa: Sub-Bass + Grit) ---
                let bass_midi = (BASS_PATTERN[step_idx] as i16 + theme_transpose).clamp(1, 127) as u8;
                let bass_freq = midi_to_freq(bass_midi);
                self.bass_phase = (self.bass_phase + bass_freq * dt) % 1.0;

                let sub_bass = (self.bass_phase * std::f32::consts::TAU).sin();
                let grit_bass = antialiased_saw(self.bass_phase, bass_freq * dt);
                let bass_attack = (step_time / 0.008).min(1.0);
                let bass_gate = if step_progress < 0.85 { 1.0 } else { (1.0 - step_progress) / 0.15 };
                let bass_tone = (sub_bass * 0.65 + grit_bass * 0.35) * bass_attack * bass_gate * 0.24;
                mix_l += bass_tone;
                mix_r += bass_tone;

                // --- Batería Punchy (Kick de caída exponencial, Snare estéreo, Hi-Hat) ---
                let in_bar_step = step_idx % 16;
                let is_fill_bar = bar_idx == 3 || bar_idx == 7;
                let drum_type = if is_fill_bar && in_bar_step >= 12 {
                    2
                } else {
                    match in_bar_step {
                        0 | 7 | 8 | 10 => 1,
                        4 | 12 => 2,
                        2 | 6 | 14 | 15 => 3,
                        _ => 0,
                    }
                };

                match drum_type {
                    1 => {
                        let k_env = (1.0 - (step_time / 0.075)).max(0.0);
                        let k_freq = 48.0 + 132.0 * k_env.powi(3);
                        let k_phase = (step_time * k_freq) % 1.0;
                        let k_sine = (k_phase * std::f32::consts::TAU).sin();
                        let k_click = if step_time < 0.004 { noise_val * 0.40 } else { 0.0 };
                        let k_val = soft_limit((k_sine + k_click) * 1.5) * k_env * 0.26;
                        mix_l += k_val;
                        mix_r += k_val;
                    }
                    2 => {
                        let s_env = (1.0 - (step_time / 0.13)).max(0.0).powi(2);
                        let s_body = (step_time * 190.0 * std::f32::consts::TAU).sin() * 0.35;
                        let s_noise = noise_val * 0.70;
                        let s_tone = (s_body + s_noise) * s_env * 0.22;
                        mix_l += s_tone * 0.95;
                        mix_r += s_tone * 1.05;
                    }
                    3 => {
                        let h_env = (1.0 - (step_time / 0.030)).max(0.0);
                        let h_val = noise_val * h_env * 0.10;
                        mix_l += h_val * 0.8;
                        mix_r += h_val * 1.2;
                    }
                    _ => {}
                }
            }

            // 2. Efectos de Sonido Procedurales Modernos
            if self.sfx_enabled && !self.active_sfx.is_empty() {
                let mut seed = self.noise_seed;
                for sfx in self.active_sfx.iter_mut() {
                    sfx.time += dt;
                    let progress = sfx.time / sfx.duration;
                    if progress >= 1.0 {
                        continue;
                    }

                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise_val = ((seed >> 16) as f32 / 32768.0) - 1.0;

                    let val = match sfx.s_type {
                        SoundEffect::Laser => {
                            let freq = 340.0 + 2460.0 * (1.0 - progress).powi(2);
                            let mod_idx = (1.0 - progress) * 2.5;
                            let phase_carrier = (sfx.time * freq) % 1.0;
                            let phase_mod = (sfx.time * (freq * 2.0)) % 1.0;
                            let modulator = (phase_mod * std::f32::consts::TAU).sin() * mod_idx;
                            let carrier = ((phase_carrier + modulator) * std::f32::consts::TAU).sin();
                            let click = if sfx.time < 0.002 { noise_val * 0.5 } else { 0.0 };
                            (carrier + click) * (1.0 - progress).powi(2) * 0.40
                        }
                        SoundEffect::SpreadFire => {
                            let freq = 260.0 + 1540.0 * (1.0 - progress).powi(2);
                            let p1 = (sfx.time * freq * std::f32::consts::TAU).sin();
                            let p2 = (sfx.time * (freq * 1.04) * std::f32::consts::TAU).sin() * 0.6;
                            (p1 + p2) * (1.0 - progress).powi(2) * 0.38
                        }
                        SoundEffect::Explosion => {
                            let impact = if sfx.time < 0.02 {
                                (sfx.time * 95.0 * std::f32::consts::TAU).sin() * 0.7
                            } else {
                                0.0
                            };
                            let sub_drop = 38.0 + 35.0 * (1.0 - (sfx.time / 0.25).min(1.0));
                            let sub = (sfx.time * sub_drop * std::f32::consts::TAU).sin() * 0.5;
                            let fireball = noise_val * (1.0 - progress).powi(2);
                            soft_limit(impact + sub + fireball) * 0.58
                        }
                        SoundEffect::BombExplosion => {
                            let sub = (sfx.time * 28.0 * std::f32::consts::TAU).sin() * 0.7;
                            let blast = noise_val * 0.8;
                            let rumble = (sfx.time * 55.0 * std::f32::consts::TAU).sin() * 0.4;
                            soft_limit(sub + blast + rumble) * (1.0 - progress).powi(2) * 0.70
                        }
                        SoundEffect::EmpShockwave => {
                            let sweep = 24.0 + 360.0 * (1.0 - progress).powi(3);
                            let tone = (sfx.time * sweep * std::f32::consts::TAU).sin();
                            let sizzle = noise_val * (1.0 - progress * 1.5).max(0.0) * 0.4;
                            (tone + sizzle) * (1.0 - progress * 0.6) * 0.44
                        }
                        SoundEffect::PowerUp => {
                            let note_idx = ((progress * 4.0) as usize).min(3);
                            let freq = [659.25, 830.61, 987.77, 1318.51][note_idx];
                            let p = (sfx.time * freq * std::f32::consts::TAU).sin();
                            let harm = (sfx.time * freq * 2.76 * std::f32::consts::TAU).sin() * 0.35;
                            (p + harm) * (1.0 - (progress * 4.0) % 1.0 * 0.35) * 0.38
                        }
                        SoundEffect::PlayerHit => {
                            let thud = (sfx.time * 75.0 * std::f32::consts::TAU).sin();
                            let crunch = noise_val * (1.0 - progress * 2.0).max(0.0) * 0.6;
                            soft_limit(thud + crunch) * (1.0 - progress).powi(2) * 0.48
                        }
                        SoundEffect::BossAlarm => {
                            let freq = if (sfx.time * 5.5) as u32 % 2 == 0 { 880.0 } else { 660.0 };
                            let p = (sfx.time * freq) % 1.0;
                            antialiased_saw(p, freq * dt) * 0.42
                        }
                        SoundEffect::SatelliteLock => {
                            let f = 2400.0 + (progress * 1200.0);
                            (sfx.time * f * std::f32::consts::TAU).sin() * (1.0 - progress).powi(2) * 0.40
                        }
                        SoundEffect::Ricochet => {
                            let f = 1200.0 + progress * 2400.0;
                            let p = (sfx.time * f * std::f32::consts::TAU).sin();
                            let ping = (sfx.time * 4200.0 * std::f32::consts::TAU).sin() * 0.4;
                            (p + ping) * (1.0 - progress).powi(3) * 0.45
                        }
                        SoundEffect::MetalClang => {
                            let m1 = (sfx.time * 1920.0 * std::f32::consts::TAU).sin();
                            let m2 = (sfx.time * 2880.0 * std::f32::consts::TAU).sin() * 0.6;
                            (m1 + m2) * (1.0 - progress).powi(2) * 0.40
                        }
                        SoundEffect::ThrusterBurst => {
                            let env = (sfx.time / 0.02).min(1.0) * (1.0 - progress);
                            (noise_val * 0.6 + (sfx.time * 90.0 * std::f32::consts::TAU).sin() * 0.4) * env * 0.32
                        }
                    };

                    let pan_l = (1.0 - sfx.pan).clamp(0.0, 2.0) * 0.5;
                    let pan_r = (1.0 + sfx.pan).clamp(0.0, 2.0) * 0.5;
                    mix_l += val * pan_l;
                    mix_r += val * pan_r;
                }
                self.noise_seed = seed;
            }

            // 3. Master Soft Limiting y Conversión a PCM 16-bit
            let out_l = (soft_limit(mix_l) * 32760.0) as i16;
            let out_r = (soft_limit(mix_r) * 32760.0) as i16;

            buffer[frame_idx * 2] = out_l;
            buffer[frame_idx * 2 + 1] = out_r;
        }

        if self.sfx_enabled {
            self.active_sfx.retain(|s| s.time < s.duration);
        }
    }

    /// Renderiza muestras mono para compatibilidad previa con arrays de JNI
    pub fn render_samples(&mut self, buffer: &mut [i16]) {
        let mut stereo_temp = vec![0i16; buffer.len() * 2];
        self.render_samples_stereo(&mut stereo_temp);
        for (i, sample) in buffer.iter_mut().enumerate() {
            let l = stereo_temp[i * 2] as i32;
            let r = stereo_temp[i * 2 + 1] as i32;
            *sample = ((l + r) / 2) as i16;
        }
    }
}

// ==============================================================================
// Enlace Dinámico con Android NDK AAudio (libaaudio.so)
// ==============================================================================

const RTLD_NOW: c_int = 2;

extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
const AAUDIO_FORMAT_PCM_I16: i32 = 1;
const AAUDIO_PERFORMANCE_MODE_LOW_LATENCY: i32 = 12;
const AAUDIO_SHARING_MODE_SHARED: i32 = 1;

struct AAudioApi {
    handle: *mut c_void,
    create_stream_builder: unsafe extern "C" fn(*mut *mut c_void) -> i32,
    builder_set_direction: unsafe extern "C" fn(*mut c_void, i32),
    builder_set_sample_rate: unsafe extern "C" fn(*mut c_void, i32),
    builder_set_channel_count: unsafe extern "C" fn(*mut c_void, i32),
    builder_set_format: unsafe extern "C" fn(*mut c_void, i32),
    builder_set_performance_mode: unsafe extern "C" fn(*mut c_void, i32),
    builder_set_sharing_mode: unsafe extern "C" fn(*mut c_void, i32),
    builder_open_stream: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32,
    builder_delete: unsafe extern "C" fn(*mut c_void) -> i32,
    stream_request_start: unsafe extern "C" fn(*mut c_void) -> i32,
    stream_request_pause: unsafe extern "C" fn(*mut c_void) -> i32,
    stream_request_stop: unsafe extern "C" fn(*mut c_void) -> i32,
    stream_close: unsafe extern "C" fn(*mut c_void) -> i32,
    stream_write: unsafe extern "C" fn(*mut c_void, *const c_void, i32, i64) -> i32,
}

impl AAudioApi {
    unsafe fn load() -> Option<Self> {
        let lib_name = CStr::from_bytes_with_nul(b"libaaudio.so\0").ok()?;
        let handle = dlopen(lib_name.as_ptr(), RTLD_NOW);
        if handle.is_null() {
            return None;
        }

        macro_rules! get_sym {
            ($sym:literal, $ty:ty) => {{
                let sym_name = CStr::from_bytes_with_nul($sym).ok()?;
                let sym_ptr = dlsym(handle, sym_name.as_ptr());
                if sym_ptr.is_null() {
                    dlclose(handle);
                    return None;
                }
                std::mem::transmute::<*mut c_void, $ty>(sym_ptr)
            }};
        }

        Some(Self {
            handle,
            create_stream_builder: get_sym!(b"AAudio_createStreamBuilder\0", unsafe extern "C" fn(*mut *mut c_void) -> i32),
            builder_set_direction: get_sym!(b"AAudioStreamBuilder_setDirection\0", unsafe extern "C" fn(*mut c_void, i32)),
            builder_set_sample_rate: get_sym!(b"AAudioStreamBuilder_setSampleRate\0", unsafe extern "C" fn(*mut c_void, i32)),
            builder_set_channel_count: get_sym!(b"AAudioStreamBuilder_setChannelCount\0", unsafe extern "C" fn(*mut c_void, i32)),
            builder_set_format: get_sym!(b"AAudioStreamBuilder_setFormat\0", unsafe extern "C" fn(*mut c_void, i32)),
            builder_set_performance_mode: get_sym!(b"AAudioStreamBuilder_setPerformanceMode\0", unsafe extern "C" fn(*mut c_void, i32)),
            builder_set_sharing_mode: get_sym!(b"AAudioStreamBuilder_setSharingMode\0", unsafe extern "C" fn(*mut c_void, i32)),
            builder_open_stream: get_sym!(b"AAudioStreamBuilder_openStream\0", unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> i32),
            builder_delete: get_sym!(b"AAudioStreamBuilder_delete\0", unsafe extern "C" fn(*mut c_void) -> i32),
            stream_request_start: get_sym!(b"AAudioStream_requestStart\0", unsafe extern "C" fn(*mut c_void) -> i32),
            stream_request_pause: get_sym!(b"AAudioStream_requestPause\0", unsafe extern "C" fn(*mut c_void) -> i32),
            stream_request_stop: get_sym!(b"AAudioStream_requestStop\0", unsafe extern "C" fn(*mut c_void) -> i32),
            stream_close: get_sym!(b"AAudioStream_close\0", unsafe extern "C" fn(*mut c_void) -> i32),
            stream_write: get_sym!(b"AAudioStream_write\0", unsafe extern "C" fn(*mut c_void, *const c_void, i32, i64) -> i32),
        })
    }
}

impl Drop for AAudioApi {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { dlclose(self.handle); }
        }
    }
}

/// Bucle de reproducción nativa de baja latencia con AAudio en un hilo independiente.
pub fn native_playback_loop(
    running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    audio_engine: Arc<Mutex<AudioEngine>>,
) {
    let api = unsafe { AAudioApi::load() };
    if api.is_none() {
        while running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }
        return;
    }
    let api = api.unwrap();

    let mut stream: *mut c_void = std::ptr::null_mut();
    unsafe {
        let mut builder: *mut c_void = std::ptr::null_mut();
        if (api.create_stream_builder)(&mut builder) != 0 || builder.is_null() {
            return;
        }

        (api.builder_set_direction)(builder, AAUDIO_DIRECTION_OUTPUT);
        (api.builder_set_sample_rate)(builder, SAMPLE_RATE as i32);
        (api.builder_set_channel_count)(builder, CHANNELS as i32);
        (api.builder_set_format)(builder, AAUDIO_FORMAT_PCM_I16);
        (api.builder_set_performance_mode)(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);
        (api.builder_set_sharing_mode)(builder, AAUDIO_SHARING_MODE_SHARED);

        let res = (api.builder_open_stream)(builder, &mut stream);
        (api.builder_delete)(builder);

        if res != 0 || stream.is_null() {
            return;
        }

        (api.stream_request_start)(stream);
    }

    let mut pcm_buffer = vec![0i16; BUFFER_FRAMES * CHANNELS];
    let timeout_ns: i64 = 100_000_000; // 100 ms timeout de escritura
    let mut was_paused = false;

    while running.load(Ordering::SeqCst) {
        let paused = is_paused.load(Ordering::SeqCst);
        if paused {
            if !was_paused {
                unsafe { (api.stream_request_pause)(stream); }
                was_paused = true;
            }
            thread::sleep(Duration::from_millis(50));
            continue;
        } else if was_paused {
            unsafe { (api.stream_request_start)(stream); }
            was_paused = false;
        }

        // Generar muestras estéreo
        if let Ok(mut engine) = audio_engine.lock() {
            engine.render_samples_stereo(&mut pcm_buffer);
        } else {
            pcm_buffer.fill(0);
        }

        // Escritura bloqueante sobre AAudio: marca el paso del reloj de hardware
        unsafe {
            let written = (api.stream_write)(
                stream,
                pcm_buffer.as_ptr() as *const c_void,
                BUFFER_FRAMES as i32,
                timeout_ns,
            );
            if written < 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    unsafe {
        (api.stream_request_stop)(stream);
        (api.stream_close)(stream);
    }
}
