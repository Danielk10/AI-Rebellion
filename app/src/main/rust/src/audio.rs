//! Sintetizador Chiptune y Efectos de Sonido Procedurales en Rust
//! Genera ondas cuadradas, triangulares y ruido blanco en tiempo real (16-bit PCM, 44100 Hz)

pub const SAMPLE_RATE: u32 = 44100;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SoundEffect {
    Laser,
    SpreadFire,
    Explosion,
    PowerUp,
    BombExplosion,
    PlayerHit,
    BossAlarm,
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
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            active_sfx: Vec::new(),
            music_time: 0.0,
            music_enabled: true,
            sfx_enabled: true,
            stage_theme: 1,
        }
    }

    pub fn play_sfx(&mut self, sfx: SoundEffect) {
        if !self.sfx_enabled {
            return;
        }
        let duration = match sfx {
            SoundEffect::Laser => 0.12,
            SoundEffect::SpreadFire => 0.15,
            SoundEffect::Explosion => 0.35,
            SoundEffect::PowerUp => 0.40,
            SoundEffect::BombExplosion => 0.90,
            SoundEffect::PlayerHit => 0.25,
            SoundEffect::BossAlarm => 1.20,
        };
        self.active_sfx.push(SfxInstance {
            s_type: sfx,
            time: 0.0,
            duration,
        });
    }

    /// Rellena un búfer con muestras de audio mono PCM de 16-bit
    pub fn render_samples(&mut self, buffer: &mut [i16]) {
        let dt = 1.0 / (SAMPLE_RATE as f32);

        for sample in buffer.iter_mut() {
            let mut mix = 0.0f32;

            // 1. Sintetizador de Música Chiptune (Basado en el tema del nivel)
            if self.music_enabled {
                self.music_time += dt;
                let beat = (self.music_time * 4.0) as u32;
                let base_freq = match (self.stage_theme + (beat % 4) as u8) % 6 {
                    0 => 130.81, // C3
                    1 => 146.83, // D3
                    2 => 164.81, // E3
                    3 => 174.61, // F3
                    4 => 196.00, // G3
                    _ => 220.00, // A3
                };

                // Onda cuadrada (Lead & Bassline retro)
                let phase = (self.music_time * base_freq) % 1.0;
                let lead = if phase < 0.5 { 0.10 } else { -0.10 };

                // Percusión de ruido periódica (Hi-hat / Snare retro)
                let noise = if (self.music_time * 8.0) % 1.0 < 0.08 {
                    ((self.music_time * 9999.0).sin()) * 0.08
                } else {
                    0.0
                };

                mix += lead + noise;
            }

            // 2. Efectos de sonido activos
            for sfx in self.active_sfx.iter_mut() {
                sfx.time += dt;
                let progress = sfx.time / sfx.duration;
                if progress >= 1.0 {
                    continue;
                }

                let vol = (1.0 - progress) * 0.45;
                let val = match sfx.s_type {
                    SoundEffect::Laser => {
                        let freq = 880.0 * (1.0 - progress * 0.6);
                        let p = (sfx.time * freq) % 1.0;
                        (if p < 0.5 { 1.0 } else { -1.0 }) * vol
                    }
                    SoundEffect::SpreadFire => {
                        let freq = 550.0 * (1.0 - progress * 0.4);
                        let p = (sfx.time * freq) % 1.0;
                        (if p < 0.3 { 1.0 } else { -1.0 }) * vol
                    }
                    SoundEffect::Explosion | SoundEffect::BombExplosion => {
                        // Ruido pseudo-aleatorio para explosiones
                        ((sfx.time * 48271.0).sin()) * vol * 1.2
                    }
                    SoundEffect::PowerUp => {
                        let freq = 440.0 + (progress * 880.0);
                        let p = (sfx.time * freq) % 1.0;
                        (if p < 0.5 { 0.8 } else { -0.8 }) * vol
                    }
                    SoundEffect::PlayerHit => {
                        ((sfx.time * 200.0).sin()) * vol
                    }
                    SoundEffect::BossAlarm => {
                        let freq = if (sfx.time * 5.0) as u32 % 2 == 0 { 800.0 } else { 600.0 };
                        let p = (sfx.time * freq) % 1.0;
                        (if p < 0.5 { 1.0 } else { -1.0 }) * vol
                    }
                };
                mix += val;
            }

            // Limpieza de SFX terminados
            self.active_sfx.retain(|s| s.time < s.duration);

            // Clamp a 16-bit signed
            let clamped = (mix.clamp(-1.0, 1.0) * 32767.0) as i16;
            *sample = clamped;
        }
    }
}
