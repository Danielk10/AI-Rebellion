//! Sintetizador Chiptune y Efectos de Sonido Procedurales en Rust
//! Genera polifonía de 4 canales inspirada en NES Final Mission (Lead, Arp, Bass, Percusión)
//! y efectos de sonido analógicos procedurales en tiempo real (16-bit PCM, 44100 Hz, 0 asignaciones heap).

pub const SAMPLE_RATE: u32 = 44100;
pub const MAX_ACTIVE_SFX: usize = 16;

const BPM: f32 = 148.0;
const SECONDS_PER_BEAT: f32 = 60.0 / BPM;
const SECONDS_PER_STEP: f32 = SECONDS_PER_BEAT / 4.0; // Semicorchea (~0.10135s)
const PATTERN_STEPS: usize = 128; // Estructura de 8 compases (16 pasos por compás)

#[inline(always)]
fn midi_to_freq(note: u8) -> f32 {
    if note == 0 {
        0.0
    } else {
        440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0)
    }
}

// ==============================================================================
// Partituras Chiptune (8 Compases: Melodía Heroica, Acordes y Bajo Galopante)
// ==============================================================================

// Melodía principal (Pulse 1 - 50% duty con staccato y vibrato)
const LEAD_PATTERN: [u8; 128] = [
    // Compás 1: Dm (Apertura contundente)
    74, 74, 74, 74, 77, 77, 76, 76, 74, 74, 74, 74, 69, 69, 72, 72,
    // Compás 2: Bb (Impulso ascendente)
    74, 74, 74, 74, 77, 77, 79, 79, 77, 77, 74, 74, 70, 70, 72, 72,
    // Compás 3: C (Frase rítmica sincopada)
    76, 76, 76, 76, 79, 79, 81, 81, 79, 79, 76, 76, 72, 72, 74, 74,
    // Compás 4: A (Tensión y resolución de estrofa)
    73, 73, 76, 76, 81, 81, 81, 81, 79, 79, 76, 76, 73, 73, 74, 76,
    // Compás 5: F (¡Clímax heroico en registro alto!)
    81, 81, 81, 81, 84, 84, 81, 81, 77, 77, 77, 77, 79, 79, 81, 81,
    // Compás 6: G (Brillo armónico modulado)
    83, 83, 83, 83, 86, 86, 83, 83, 79, 79, 79, 79, 81, 81, 83, 83,
    // Compás 7: Bb (Escalada a la cima emocional)
    86, 86, 86, 86, 84, 84, 82, 82, 81, 81, 79, 79, 81, 81, 82, 82,
    // Compás 8: A7 (Cierre triunfal y vuelta al loop)
    85, 85, 85, 85, 88, 88, 86, 86, 85, 85, 81, 81, 76, 76, 74, 74,
];

// Bajo galopante de 16 semicorcheas por compás (Triangle Channel estilo Natsume)
const BASS_PATTERN: [u8; 128] = [
    // Compás 1: Dm (Galope D2-D3)
    38, 38, 50, 38, 38, 50, 38, 38, 50, 38, 38, 50, 38, 50, 48, 49,
    // Compás 2: Bb
    34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 46, 36, 37,
    // Compás 3: C
    36, 36, 48, 36, 36, 48, 36, 36, 48, 36, 36, 48, 36, 48, 38, 40,
    // Compás 4: A
    33, 33, 45, 33, 33, 45, 33, 33, 45, 33, 45, 33, 34, 35, 36, 37,
    // Compás 5: F
    29, 29, 41, 29, 29, 41, 29, 29, 41, 29, 29, 41, 29, 41, 31, 32,
    // Compás 6: G
    31, 31, 43, 31, 31, 43, 31, 31, 43, 31, 31, 43, 31, 43, 33, 34,
    // Compás 7: Bb
    34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 34, 46, 34, 46, 36, 37,
    // Compás 8: A7
    33, 33, 45, 33, 45, 33, 45, 33, 45, 45, 45, 45, 45, 46, 47, 49,
];

// Acordes de arpegio a 60 Hz (Pulse 2 - 25% duty)
const ARP_CHORDS: [[u8; 4]; 8] = [
    [62, 65, 69, 74], // Dm  (D4, F4, A4, D5)
    [58, 62, 65, 70], // Bb  (Bb3, D4, F4, Bb4)
    [60, 64, 67, 72], // C   (C4, E4, G4, C5)
    [57, 61, 64, 69], // A   (A3, C#4, E4, A4)
    [53, 57, 60, 65], // F   (F3, A3, C4, F4)
    [55, 59, 62, 67], // G   (G3, B3, D4, G4)
    [58, 62, 65, 70], // Bb  (Bb3, D4, F4, Bb4)
    [57, 61, 64, 69], // A7  (A3, C#4, E4, A4)
];

// ==============================================================================
// Efectos de Sonido y Estructuras
// ==============================================================================

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
}

#[derive(Clone, Debug)]
pub struct AudioEngine {
    pub active_sfx: Vec<SfxInstance>,
    pub music_time: f32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
    pub stage_theme: u8,

    // Acumuladores de fase independientes (previenen degradación de coma flotante)
    lead_phase: f32,
    arp_phase: f32,
    bass_phase: f32,
    noise_lfsr: u16,
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
            arp_phase: 0.0,
            bass_phase: 0.0,
            noise_lfsr: 0x7FFF,
        }
    }

    pub fn play_sfx(&mut self, sfx: SoundEffect) {
        if !self.sfx_enabled {
            return;
        }

        let duration = match sfx {
            SoundEffect::Laser => 0.12,
            SoundEffect::SpreadFire => 0.15,
            SoundEffect::Explosion => 0.38,
            SoundEffect::PowerUp => 0.40,
            SoundEffect::BombExplosion => 0.95,
            SoundEffect::PlayerHit => 0.22,
            SoundEffect::BossAlarm => 1.20,
            SoundEffect::SatelliteLock => 0.18,
            SoundEffect::EmpShockwave => 1.10,
            SoundEffect::Ricochet => 0.14,
            SoundEffect::MetalClang => 0.16,
            SoundEffect::ThrusterBurst => 0.18,
        };

        let new_instance = SfxInstance {
            s_type: sfx,
            time: 0.0,
            duration,
        };

        if self.active_sfx.len() < MAX_ACTIVE_SFX {
            self.active_sfx.push(new_instance);
        } else {
            // Robo de voz (Voice Stealing): reutiliza la ranura del SFX más cercano a expirar
            if let Some(oldest) = self.active_sfx.iter_mut().max_by(|a, b| {
                let pa = a.time / a.duration.max(0.001);
                let pb = b.time / b.duration.max(0.001);
                pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
            }) {
                *oldest = new_instance;
            }
        }
    }

    /// Rellena el búfer de audio mono PCM de 16 bits sin asignación de memoria heap (0 runtime allocation).
    pub fn render_samples(&mut self, buffer: &mut [i16]) {
        let dt = 1.0 / (SAMPLE_RATE as f32);

        for sample in buffer.iter_mut() {
            let mut mix = 0.0f32;

            // 1. Sintetizador de Música Chiptune Polifónico (NES Final Mission Style)
            if self.music_enabled {
                self.music_time += dt;

                // Generador LFSR de 15 bits para el canal de ruido blanco (NES 2A03)
                let fb = (self.noise_lfsr & 1) ^ ((self.noise_lfsr >> 1) & 1);
                self.noise_lfsr = (self.noise_lfsr >> 1) | (fb << 14);
                let noise_val = if (self.noise_lfsr & 1) != 0 { 1.0 } else { -1.0 };

                // Temporización musical a 148 BPM
                let total_steps = (self.music_time / SECONDS_PER_STEP) as usize;
                let step_idx = total_steps % PATTERN_STEPS;
                let bar_idx = (step_idx / 16) % 8;
                let step_time = self.music_time - (total_steps as f32 * SECONDS_PER_STEP);
                let step_progress = step_time / SECONDS_PER_STEP;

                // Transposición armónica según stage_theme
                let theme_transpose: i16 = match (self.stage_theme.saturating_sub(1)) % 4 {
                    0 => 0,  // D minor (Stage 1 / 5)
                    1 => 2,  // E minor (Stage 2 / 6)
                    2 => 5,  // G minor (Stage 3 / 7)
                    _ => -2, // C minor (Stage 4 / 8)
                };

                // --- Canal 1: Lead Melody (Pulse Wave 50% con vibrato) ---
                let lead_midi = (LEAD_PATTERN[step_idx] as i16 + theme_transpose).clamp(1, 127) as u8;
                let mut lead_freq = midi_to_freq(lead_midi);
                if step_progress > 0.40 {
                    lead_freq += (self.music_time * 6.0 * std::f32::consts::TAU).sin() * 3.5;
                }
                self.lead_phase = (self.lead_phase + lead_freq * dt) % 1.0;
                let lead_pulse = if self.lead_phase < 0.50 { 1.0 } else { -1.0 };
                let lead_attack = (step_time / 0.015).min(1.0);
                let lead_gate = if step_progress < 0.90 { 1.0 } else { (1.0 - step_progress) / 0.10 };
                let lead_out = lead_pulse * lead_attack * lead_gate * 0.16;

                // --- Canal 2: Arpegios / Acordes (Pulse Wave 25% a 60 Hz) ---
                let sub_tick = ((step_progress * 4.0) as usize).min(3);
                let arp_midi = (ARP_CHORDS[bar_idx][sub_tick] as i16 + theme_transpose).clamp(1, 127) as u8;
                let arp_freq = midi_to_freq(arp_midi);
                self.arp_phase = (self.arp_phase + arp_freq * dt) % 1.0;
                let arp_pulse = if self.arp_phase < 0.25 { 1.0 } else { -1.0 };
                let arp_out = arp_pulse * 0.08;

                // --- Canal 3: Bassline (Triangle Wave, galope a semicorcheas) ---
                let bass_midi = (BASS_PATTERN[step_idx] as i16 + theme_transpose).clamp(1, 127) as u8;
                let bass_freq = midi_to_freq(bass_midi);
                self.bass_phase = (self.bass_phase + bass_freq * dt) % 1.0;
                let tri_val = 2.0 * (2.0 * (self.bass_phase - 0.5).abs() - 0.5);
                let bass_attack = (step_time / 0.010).min(1.0);
                let bass_gate = if step_progress < 0.85 { 1.0 } else { (1.0 - step_progress) / 0.15 };
                let bass_out = tri_val * bass_attack * bass_gate * 0.22;

                // --- Canal 4: Percusión (Canal de Ruido: Kick, Snare, Hi-hat) ---
                let in_bar_step = step_idx % 16;
                let is_fill_bar = bar_idx == 3 || bar_idx == 7;
                let drum_type = if is_fill_bar && in_bar_step >= 12 {
                    2 // Redoble de caja en compases de resolución
                } else {
                    match in_bar_step {
                        0 | 7 | 8 | 10 => 1,  // Kick drum contundente
                        4 | 12 => 2,          // Snare drum seco
                        2 | 6 | 14 | 15 => 3,  // Hi-hat cerrado
                        _ => 0,
                    }
                };

                let drum_out = match drum_type {
                    1 => {
                        let k_freq = 45.0 + 115.0 * (1.0 - (step_time / 0.08).min(1.0)).powi(2);
                        let k_phase = (step_time * k_freq) % 1.0;
                        let k_body = (k_phase * std::f32::consts::TAU).sin();
                        let k_click = if step_time < 0.012 { noise_val * 0.35 } else { 0.0 };
                        let k_env = (1.0 - (step_time / 0.08)).max(0.0);
                        (k_body + k_click) * k_env * 0.22
                    }
                    2 => {
                        let s_env = (1.0 - (step_time / 0.14)).max(0.0).powi(2);
                        let s_body = (step_time * 180.0 * std::f32::consts::TAU).sin() * 0.35;
                        (noise_val * 0.75 + s_body) * s_env * 0.20
                    }
                    3 => {
                        let h_env = (1.0 - (step_time / 0.035)).max(0.0);
                        noise_val * h_env * 0.10
                    }
                    _ => 0.0,
                };

                mix += lead_out + arp_out + bass_out + drum_out;
            }

            // 2. Efectos de Sonido Activos (SFX)
            if self.sfx_enabled && !self.active_sfx.is_empty() {
                for sfx in self.active_sfx.iter_mut() {
                    sfx.time += dt;
                    let progress = sfx.time / sfx.duration;
                    if progress >= 1.0 {
                        continue;
                    }

                    let val = match sfx.s_type {
                        SoundEffect::Laser => {
                            let freq = 960.0 * (1.0 - progress * 0.70);
                            let p = (sfx.time * freq) % 1.0;
                            (if p < 0.5 { 1.0 } else { -1.0 }) * (1.0 - progress) * 0.40
                        }
                        SoundEffect::SpreadFire => {
                            let freq = 700.0 * (1.0 - progress * 0.60);
                            let p = (sfx.time * freq) % 1.0;
                            let p_sub = (sfx.time * (freq * 0.50)) % 1.0;
                            ((if p < 0.3 { 0.7 } else { -0.7 }) + (if p_sub < 0.5 { 0.3 } else { -0.3 }))
                                * (1.0 - progress)
                                * 0.38
                        }
                        SoundEffect::Explosion => {
                            let noise = (sfx.time * 48271.0).sin();
                            let rumble = (sfx.time * 50.0 * std::f32::consts::TAU).sin() * 0.5;
                            (noise + rumble) * (1.0 - progress).powi(2) * 0.55
                        }
                        SoundEffect::BombExplosion => {
                            let sub = (sfx.time * 38.0 * std::f32::consts::TAU).sin();
                            let noise = (sfx.time * 31337.0).sin();
                            let rumble = (sfx.time * 65.0 * std::f32::consts::TAU).sin() * 0.5;
                            (sub * 0.6 + noise * 0.7 + rumble * 0.3).clamp(-1.0, 1.0)
                                * (1.0 - progress).powi(2)
                                * 0.65
                        }
                        SoundEffect::PowerUp => {
                            let note_idx = ((progress * 5.0) as usize).min(4);
                            let freq = [523.25, 659.25, 783.99, 1046.50, 1318.51][note_idx];
                            let p = (sfx.time * freq) % 1.0;
                            let step_env = 1.0 - (progress * 5.0) % 1.0 * 0.3;
                            (if p < 0.5 { 0.8 } else { -0.8 }) * step_env * 0.35
                        }
                        SoundEffect::PlayerHit => {
                            let thud = (sfx.time * 85.0 * (1.0 - progress * 0.5) * std::f32::consts::TAU).sin();
                            let noise = if progress < 0.3 {
                                (sfx.time * 44000.0).sin() * 0.7
                            } else {
                                0.0
                            };
                            (thud + noise) * (1.0 - progress).powi(2) * 0.45
                        }
                        SoundEffect::BossAlarm => {
                            let freq = if (sfx.time * 5.0) as u32 % 2 == 0 { 840.0 } else { 630.0 };
                            let p = (sfx.time * freq) % 1.0;
                            (if p < 0.5 { 1.0 } else { -1.0 }) * (1.0 - progress * 0.3) * 0.42
                        }
                        SoundEffect::SatelliteLock => {
                            let f = if sfx.time < 0.025 {
                                1600.0 + (sfx.time / 0.025) * 1040.0
                            } else {
                                2640.0
                            };
                            let p1 = (sfx.time * f * std::f32::consts::TAU).sin();
                            let p2 = (sfx.time * (f * 1.5) * std::f32::consts::TAU).sin() * 0.45;
                            (p1 + p2) * (1.0 - progress).powi(2) * 0.45
                        }
                        SoundEffect::EmpShockwave => {
                            let sweep = 32.0 + 388.0 * (1.0 - progress).powi(3);
                            let sub = (sfx.time * sweep * std::f32::consts::TAU).sin();
                            let ring = (sfx.time * (sweep * 1.5) * std::f32::consts::TAU).sin() * 0.40;
                            let crackle = if progress < 0.5 {
                                (sfx.time * 28000.0).sin() * (1.0 - progress * 2.0) * 0.35
                            } else {
                                0.0
                            };
                            let rumble = (sfx.time * 40.0 * std::f32::consts::TAU).sin() * 0.5 * (1.0 - progress);
                            (sub + ring + crackle + rumble) * (1.0 - progress * 0.7) * 0.42
                        }
                        SoundEffect::Ricochet => {
                            let freq = 1400.0 + (progress.min(0.5) / 0.5) * 2200.0;
                            let p = (sfx.time * freq) % 1.0;
                            let twang = if p < 0.5 { 0.8 } else { -0.8 };
                            let clang = (sfx.time * 3300.0 * std::f32::consts::TAU).sin() * 0.50;
                            let click = if sfx.time < 0.008 {
                                (sfx.time * 50000.0).sin() * 0.60
                            } else {
                                0.0
                            };
                            (twang * 0.6 + clang + click) * (1.0 - progress).powi(3) * 0.45
                        }
                        SoundEffect::MetalClang => {
                            let m1 = (sfx.time * 1920.0 * std::f32::consts::TAU).sin();
                            let m2 = (sfx.time * 2880.0 * std::f32::consts::TAU).sin() * 0.60;
                            let m3 = (sfx.time * 4150.0 * std::f32::consts::TAU).sin() * 0.30;
                            let click = if sfx.time < 0.006 {
                                (sfx.time * 50000.0).sin() * 0.50
                            } else {
                                0.0
                            };
                            (m1 + m2 + m3 + click) * (1.0 - progress).powi(2) * 0.40
                        }
                        SoundEffect::ThrusterBurst => {
                            let att = (sfx.time / 0.03).min(1.0);
                            let dec = 1.0 - progress;
                            let env = att * dec * 0.22;
                            let noise = (sfx.time * 48271.0).sin();
                            let rumble = (sfx.time * 110.0 * std::f32::consts::TAU).sin() * 0.40;
                            (noise * 0.70 + rumble) * env
                        }
                    };
                    mix += val;
                }
            }

            // Conversión y recorte a PCM 16-bit signed
            let clamped = (mix.clamp(-1.0, 1.0) * 32767.0) as i16;
            *sample = clamped;
        }

        // Limpieza de SFX expirados: se ejecuta UNA SOLA VEZ por búfer.
        if self.sfx_enabled {
            self.active_sfx.retain(|s| s.time < s.duration);
        }
    }
}
