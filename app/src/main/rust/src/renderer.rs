//! Rasterizador Gráfico de Alto Rendimiento para IA Rebellion
//! Experiencia visual inspirada en Final Mission (Famicom/NES Japón) de Natsume
//! y la Rebelión de la IA:
//! - Soldado humano permanente con armadura, visor HUD, jetpack de plasma y satélites bivalvos.
//! - Stage 1: Nueva York en ruinas (paleta auténtica de NES: cielo negro, nubes púrpuras,
//!   rascacielos con lamas, tuberías cromadas, vigas de celosía naranja y escombros).
//! - Splash Screen de inicio: "DIAMON BLACK - POWERED BY RUST" estilo NVIDIA bumper.
//! - HUD limpio superior SIN BOTONES VIRTUALES.

use crate::boss::{Boss, BossId};
use crate::bullet::{Bullet, BulletType};
use crate::enemy::{Enemy, EnemyType};
use crate::player::Player;

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: u32,
    pub size: f32,
}

#[derive(Clone, Debug)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub size: usize,
    pub brightness: u8,
}

#[derive(Clone, Debug)]
pub struct Renderer {
    pub stars: Vec<Star>,
    pub particles: Vec<Particle>,
    pub width: usize,
    pub height: usize,
    pub anim_time: f32,
    pub scroll_x: f32,
}

impl Renderer {
    pub fn new(w: usize, h: usize) -> Self {
        let mut stars = Vec::with_capacity(140);
        for i in 0..140 {
            let speed = match i % 4 {
                0 => 30.0,
                1 => 70.0,
                2 => 140.0,
                _ => 220.0,
            };
            stars.push(Star {
                x: ((i * 37) % w) as f32,
                y: ((i * 47) % h) as f32,
                speed,
                size: if speed > 130.0 { 2 } else { 1 },
                brightness: ((i * 19) % 155 + 100) as u8,
            });
        }

        Self {
            stars,
            particles: Vec::with_capacity(384),
            width: w,
            height: h,
            anim_time: 0.0,
            scroll_x: 0.0,
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.width = w;
        self.height = h;
    }

    pub fn add_explosion(&mut self, x: f32, y: f32, count: usize, base_color: u32) {
        for i in 0..count {
            let angle = (i as f32) * (std::f32::consts::PI * 2.0 / count as f32);
            let speed = 80.0 + ((i * 17) % 220) as f32;
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.65,
                max_life: 0.65,
                color: base_color,
                size: 3.5,
            });
        }
    }

    pub fn update_fx(&mut self, dt: f32) {
        self.anim_time += dt;
        self.scroll_x += 90.0 * dt;
        let w_f = self.width as f32;

        for s in self.stars.iter_mut() {
            s.x -= s.speed * dt;
            if s.x < 0.0 {
                s.x += w_f;
            }
        }

        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    /// Renderiza la pantalla de presentación Splash de Diamon Black & Powered by Rust
    pub fn render_splash_screen(&self, buffer: &mut [u32], timer: f32) {
        let w = self.width as isize;
        let h = self.height as isize;

        // Fondo oscuro de fibra de carbono y circuitos
        buffer.fill(0xFF07090E);

        // Trazas de circuitos cibernéticos de fondo
        for line_y in (40..h).step_by(60) {
            self.draw_rect(buffer, 0, line_y, w as usize, 1, 0xFF0E1422);
        }
        for line_x in (60..w).step_by(80) {
            self.draw_rect(buffer, line_x, 0, 1, h as usize, 0xFF0E1422);
        }

        let cx = w / 2;
        let cy = h / 2 - 20;

        // Destello de haz anamórfico central (cian a la izquierda, naranja a la derecha)
        let beam_alpha = ((timer * 2.5).sin().abs() * 0.6).min(0.6);
        self.draw_rect(buffer, 40, cy - 65, (w - 80) as usize, 2, 0xFF00E5FF);
        self.draw_point_light(buffer, cx - 180, cy - 65, 120, 0xFF00E5FF, beam_alpha);
        self.draw_point_light(buffer, cx + 180, cy + 50, 120, 0xFFFF5500, beam_alpha);

        // --- 1. TITULAR: DIAMON BLACK ---
        // Caja metálica tallada con biseles
        let t_box_w = 420;
        let t_box_h = 56;
        let t_x = cx - t_box_w / 2;
        let t_y = cy - 90;

        self.draw_rect(buffer, t_x, t_y, t_box_w as usize, t_box_h as usize, 0xFF181C26);
        self.draw_rect(buffer, t_x + 2, t_y + 2, (t_box_w - 4) as usize, 2, 0xFF4A5568);
        self.draw_rect(buffer, t_x + 2, t_y + (t_box_h - 4), (t_box_w - 4) as usize, 2, 0xFF0A0D14);

        // Letras en bloques "DIAMON BLACK"
        self.draw_simple_text(buffer, "DIAMON BLACK", cx - 130, cy - 76, 0xFFE2E8F0, 3);

        // --- 2. LOGO BADGE: POWERED BY RUST ---
        let badge_w = 360;
        let badge_h = 68;
        let b_x = cx - badge_w / 2;
        let b_y = cy + 10;

        // Placa de titanio cepillado con cantos redondeados simulados
        self.draw_rect(buffer, b_x, b_y, badge_w as usize, badge_h as usize, 0xFF1E232E);
        self.draw_rect(buffer, b_x + 3, b_y + 3, (badge_w - 6) as usize, (badge_h - 6) as usize, 0xFF2A3140);
        self.draw_rect(buffer, b_x + 4, b_y + 4, (badge_w - 8) as usize, 2, 0xFF5A667E);

        // Engranaje icónico de Rust en plasma naranja ardiente
        let gear_x = b_x + 48;
        let gear_y = b_y + 34;

        // Dientes del engranaje (8 dientes exteriores)
        for d in 0..8 {
            let rad = (d as f32) * (std::f32::consts::PI / 4.0) + timer * 0.8;
            let tx = gear_x + (rad.cos() * 22.0) as isize;
            let ty = gear_y + (rad.sin() * 22.0) as isize;
            self.draw_circle(buffer, tx, ty, 5, 0xFFFF4500);
            self.draw_circle(buffer, tx, ty, 3, 0xFFFFCC00);
        }

        // Anillo de engranaje exterior
        self.draw_circle(buffer, gear_x, gear_y, 20, 0xFFFF4500);
        self.draw_circle(buffer, gear_x, gear_y, 16, 0xFF1E232E);
        // Letra 'R' interior
        self.draw_rect(buffer, gear_x - 5, gear_y - 8, 4, 16, 0xFFFFD700);
        self.draw_rect(buffer, gear_x - 5, gear_y - 8, 10, 4, 0xFFFFD700);
        self.draw_rect(buffer, gear_x + 1, gear_y - 8, 4, 8, 0xFFFFD700);
        self.draw_rect(buffer, gear_x - 5, gear_y - 1, 10, 3, 0xFFFFD700);
        self.draw_rect(buffer, gear_x + 1, gear_y + 2, 4, 6, 0xFFFFD700);

        // Resplandor cálido del engranaje de Rust
        self.draw_point_light(buffer, gear_x, gear_y, 45, 0xFFFF4500, 0.7);

        // Textos del badge
        self.draw_simple_text(buffer, "POWERED BY", b_x + 95, b_y + 14, 0xFFA0AEC0, 1);
        self.draw_simple_text(buffer, "R U S T", b_x + 95, b_y + 32, 0xFFFF7700, 3);

        // Aviso inferior de inicio
        let tap_pulse = ((timer * 4.0).sin().abs() * 200.0) as u32;
        let tap_col = 0xFF000000 | (tap_pulse << 16) | (tap_pulse << 8) | tap_pulse;
        self.draw_simple_text(buffer, "TOCA LA PANTALLA PARA INICIAR", cx - 170, cy + 120, tap_col, 2);
    }

    pub fn render_frame(
        &mut self,
        buffer: &mut [u32],
        _bg_color: u32,
        players: &[Player],
        enemies: &[Enemy],
        boss: &Option<Boss>,
        bullets: &[Bullet],
        stage_num: u8,
        stage_name: &str,
    ) {
        let w = self.width;
        let h = self.height;

        // 1. Escenario enriquecido según nivel (Stage 1 = Ruined NYC estilo Final Mission)
        self.render_stage_environment(buffer, stage_num);

        // 2. Campo de estrellas / partículas de fondo
        for s in self.stars.iter() {
            let px = s.x as usize;
            let py = s.y as usize;
            if px < w && py < h {
                let col = 0xFF000000 | ((s.brightness as u32) << 16) | ((s.brightness as u32) << 8) | (s.brightness as u32);
                buffer[py * w + px] = col;
                if s.size > 1 && px + 1 < w {
                    buffer[py * w + px + 1] = col;
                }
            }
        }

        // 3. Proyectiles y ráfagas de energía
        for b in bullets.iter() {
            if !b.active {
                continue;
            }
            let bx = b.x as isize;
            let by = b.y as isize;
            let r = b.radius as usize;

            match b.b_type {
                BulletType::LaserBeam => {
                    self.draw_rect(buffer, bx - 22, by - 4, 44, 8, b.color);
                    self.draw_rect(buffer, bx - 18, by - 2, 36, 4, 0xFFFFFFFF);
                    self.draw_point_light(buffer, bx, by, 28, b.color, 0.4);
                }
                BulletType::SpreadWave => {
                    self.draw_circle(buffer, bx, by, r + 1, b.color);
                    self.draw_circle(buffer, bx, by, r.saturating_sub(1), 0xFFFFFFFF);
                    self.draw_point_light(buffer, bx, by, r + 14, b.color, 0.35);
                }
                BulletType::HomingMissile => {
                    self.draw_rect(buffer, bx - 10, by - 4, 20, 8, b.color);
                    self.draw_rect(buffer, bx - 14, by - 2, 4, 4, 0xFFFF6600);
                    self.draw_point_light(buffer, bx - 14, by, 16, 0xFFFF6600, 0.5);
                }
                BulletType::BioAcid => {
                    self.draw_circle(buffer, bx, by, r + 2, 0xFF39FF14);
                    self.draw_circle(buffer, bx, by, r, 0xFF88FF00);
                    self.draw_point_light(buffer, bx, by, r + 18, 0xFF39FF14, 0.45);
                }
                _ => {
                    self.draw_circle(buffer, bx, by, r, b.color);
                    self.draw_circle(buffer, bx, by, 2, 0xFFFFFFFF);
                    self.draw_point_light(buffer, bx, by, r + 10, b.color, 0.3);
                }
            }
        }

        // 4. Enemigos comunes detallados y orientables
        for e in enemies.iter() {
            if !e.active {
                continue;
            }
            self.draw_enemy(buffer, e);
        }

        // 5. Jefe de fase (Boss)
        if let Some(b) = boss.as_ref() {
            if !b.defeated {
                self.draw_boss(buffer, b);
            }
        }

        // 6. Efectos de partículas / explosiones con resplandor aditivo
        for p in self.particles.iter() {
            let px = p.x as isize;
            let py = p.y as isize;
            let sz = (p.size * (p.life / p.max_life)).max(1.0) as usize;
            self.draw_circle(buffer, px, py, sz, p.color);
            self.draw_point_light(buffer, px, py, sz + 12, p.color, 0.4);
        }

        // 7. Personaje: SIEMPRE el Comando Humano con Jetpack en todos los niveles (Estilo Final Mission)
        for player in players.iter() {
            if !player.active {
                continue;
            }
            self.draw_human_cyber_soldier(buffer, player);
        }

        // 8. HUD Limpio Superior SIN BOTONES VIRTUALES
        self.render_clean_hud(buffer, players, stage_num, stage_name);
    }

    /// Renderiza escenarios temáticos basados en Final Mission NES y el lore de Rebelión de IA
    fn render_stage_environment(&self, buffer: &mut [u32], stage: u8) {
        let w = self.width;
        let h = self.height;
        let s = self.scroll_x;
        let t = self.anim_time;

        match stage {
            // Nivel 1: RUINED NEW YORK CITY (Paleta NES 14 colores de Final Mission)
            1 => {
                // Cielo negro profundo de medianoche
                buffer.fill(0xFF000000);

                // A. Capa de Nubes de Tormenta Púrpuras en el Techo (y: 0..40)
                for nx in 0..(w / 16 + 2) {
                    let rx = (nx * 16) as isize;
                    let cloud_h = 24 + (((nx as f32 * 0.4 + t * 0.5).sin() * 8.0) as usize);
                    self.draw_rect(buffer, rx, 0, 16, cloud_h, 0xFF44009B); // Púrpura oscuro
                    self.draw_rect(buffer, rx, 0, 16, cloud_h.saturating_sub(6), 0xFF7F00EF); // Violeta eléctrico
                }

                // B. Rascacielos Lejanos en Paralaje Lento (s * 0.25)
                let bld_offset1 = (s * 0.25) as usize % 96;
                for i in 0..(w / 96 + 2) {
                    let bx = (i * 96) as isize - bld_offset1 as isize;
                    let bld_h = 160 + ((i * 47) % 110);
                    let by = h as isize - 70 - bld_h as isize;
                    // Silueta pizarra/azulada lejana
                    self.draw_rect(buffer, bx, by, 76, bld_h, 0xFF183C5C);
                    // Antena en la azotea
                    self.draw_rect(buffer, bx + 36, by - 18, 3, 18, 0xFF737373);
                    // Puntos de luz lejanos
                    for wy in 0..(bld_h / 28) {
                        self.draw_rect(buffer, bx + 20, by + 16 + (wy * 28) as isize, 4, 4, 0xFFFBFBFB);
                    }
                }

                // C. Rascacielos Medios con Ventanas en Código de Barras (s * 0.50, Paleta NES #7F00EF)
                let bld_offset2 = (s * 0.50) as usize % 120;
                for i in 0..(w / 120 + 2) {
                    let bx = (i * 120) as isize - bld_offset2 as isize;
                    let bld_h = 210 + ((i * 31) % 95);
                    let by = h as isize - 65 - bld_h as isize;
                    // Fachada púrpura
                    self.draw_rect(buffer, bx, by, 90, bld_h, 0xFF44009B);
                    // Lamas verticales de ventanas (alternando púrpura y negro)
                    for wx in 0..7 {
                        self.draw_rect(buffer, bx + 10 + (wx * 11) as isize, by + 12, 5, bld_h - 24, 0xFF7F00EF);
                    }
                }

                // D. Tuberías Industriales de Cromo en Techo y Plataforma Inferior (s * 0.85)
                let pipe_scroll = (s * 0.85) as usize % 64;
                let top_pipe_y = 50;
                let bot_pipe_y = h as isize - 64;

                // Tubería continua superior
                self.draw_rect(buffer, 0, top_pipe_y, w, 14, 0xFF737373);
                self.draw_rect(buffer, 0, top_pipe_y + 2, w, 5, 0xFFBBBBBB);
                self.draw_rect(buffer, 0, top_pipe_y + 3, w, 2, 0xFFFBFBFB); // Brillo cromo puro

                // Tubería continua inferior
                self.draw_rect(buffer, 0, bot_pipe_y, w, 16, 0xFF737373);
                self.draw_rect(buffer, 0, bot_pipe_y + 3, w, 6, 0xFFBBBBBB);
                self.draw_rect(buffer, 0, bot_pipe_y + 4, w, 2, 0xFFFBFBFB);

                // Bridas y juntas de tuberías cada 64 px
                for f in 0..(w / 64 + 2) {
                    let fx = (f * 64) as isize - pipe_scroll as isize;
                    self.draw_rect(buffer, fx, top_pipe_y - 2, 6, 18, 0xFFBBBBBB);
                    self.draw_rect(buffer, fx, bot_pipe_y - 2, 6, 20, 0xFFBBBBBB);
                }

                // E. Vigas de Celosía (Trusses) de Acero Naranja/Óxido conectando plataformas
                let girder_scroll = (s * 0.85) as usize % 80;
                for g in 0..(w / 80 + 2) {
                    let gx = (g * 80) as isize - girder_scroll as isize;
                    let gy = bot_pipe_y - 18;
                    // Viga Warren con crucetas naranja #C74C0C
                    self.draw_rect(buffer, gx, gy, 74, 16, 0xFFC74C0C);
                    self.draw_rect(buffer, gx, gy, 74, 2, 0xFFA30000);
                    self.draw_rect(buffer, gx, gy + 14, 74, 2, 0xFFA30000);
                    // Cruces en X
                    for x_i in 0..3 {
                        self.draw_rect(buffer, gx + 8 + (x_i * 20) as isize, gy + 3, 10, 10, 0xFF181C26);
                    }
                }

                // F. Suelo de Escombros y Concreto Ocre (#877000) en la base inferior
                let ground_y = h as isize - 38;
                self.draw_rect(buffer, 0, ground_y, w, 38, 0xFF877000);
                self.draw_rect(buffer, 0, ground_y, w, 3, 0xFFFBD7A7); // Aristas iluminadas
                // Detalle estocástico de cascotes
                for deb in (0..w).step_by(18) {
                    let dy = ground_y + 6 + ((deb * 13) % 24) as isize;
                    self.draw_rect(buffer, deb as isize, dy, 5, 4, 0xFF44009B);
                    self.draw_rect(buffer, deb as isize + 2, dy + 1, 2, 2, 0xFFFBFBFB);
                }
            }
            // Nivel 2: Fábrica de Drones
            2 => {
                buffer.fill(0xFF140D07);
                let beam_offset = (s * 0.4) as usize % 120;
                for bx in 0..(w / 120 + 2) {
                    let rx = (bx * 120) as isize - beam_offset as isize;
                    self.draw_rect(buffer, rx, 0, 14, 90, 0xFF2E2218);
                    self.draw_rect(buffer, rx, h as isize - 90, 14, 90, 0xFF2E2218);
                    let flash = (t * 4.0).sin() > 0.0;
                    let light_col = if flash { 0xFFFF4500 } else { 0xFF661100 };
                    self.draw_circle(buffer, rx + 7, 85, 4, light_col);
                    self.draw_circle(buffer, rx + 7, h as isize - 85, 4, light_col);
                }
                self.draw_rect(buffer, 0, h as isize - 24, w, 24, 0xFFFF5500);
                self.draw_rect(buffer, 0, h as isize - 12, w, 12, 0xFFFFCC00);
            }
            // Nivel 3: Megaciudad Desolada
            3 => {
                buffer.fill(0xFF0D0B18);
                let city_scroll = (s * 0.35) as usize % 90;
                for bld in 0..(w / 90 + 2) {
                    let bx = (bld * 90) as isize - city_scroll as isize;
                    let bld_h = 160 + ((bld * 43) % 180);
                    let by = h as isize - bld_h as isize;
                    self.draw_rect(buffer, bx, by, 75, bld_h, 0xFF17142B);
                    for wy in 0..(bld_h / 24) {
                        for wx in 0..3 {
                            let win_col = if (bld + wx + wy) % 3 == 0 { 0xFF00F0FF } else { 0xFFFF007F };
                            self.draw_rect(buffer, bx + 12 + (wx * 20) as isize, by + 18 + (wy * 24) as isize, 8, 10, win_col);
                        }
                    }
                }
            }
            // Nivel 4: Colmena Bio-Orgánica (Abadox)
            4 => {
                buffer.fill(0xFF1F0812);
                let wave_step = (s * 0.6) as f32;
                for x_col in (0..w).step_by(8) {
                    let wave_top = (35.0 + ((x_col as f32 + wave_step) * 0.04).sin() * 22.0) as isize;
                    let wave_bot = (35.0 + ((x_col as f32 - wave_step) * 0.04).cos() * 22.0) as isize;
                    self.draw_rect(buffer, x_col as isize, 0, 8, wave_top as usize, 0xFF4A1024);
                    self.draw_rect(buffer, x_col as isize, h as isize - wave_bot, 8, wave_bot as usize, 0xFF4A1024);
                    let acid_y = wave_top - 4;
                    self.draw_rect(buffer, x_col as isize, acid_y, 8, 4, 0xFF39FF14);
                }
            }
            // Niveles 5 a 8: Fondos cósmicos y estaciones orbitales
            _ => {
                buffer.fill(0xFF04060C);
                let grid_offset = (s * 0.5) as usize % 60;
                for gx in 0..(w / 60 + 2) {
                    let rx = (gx * 60) as isize - grid_offset as isize;
                    self.draw_rect(buffer, rx, 0, 1, h, 0x2200D2FF);
                }
            }
        }
    }

    /// Renderiza un Soldado Humano Cibernético estilo Final Mission con animaciones completas
    fn draw_human_cyber_soldier(&self, buffer: &mut [u32], player: &Player) {
        let px = player.x as isize;
        let py = player.y as isize;
        let dir = if player.facing_right { 1 } else { -1 };
        let t = player.anim_timer;

        // Efecto parpadeo de invulnerabilidad
        if player.invulnerable_timer > 0.0 && ((player.invulnerable_timer * 18.0) as usize % 2 == 0) {
            return;
        }

        // Inclinación dinámica al moverlo en pantalla:
        // vy negativo (subiendo) -> inclinación hacia arriba (-10°)
        // vy positivo (bajando) -> inclinación hacia abajo (+10°)
        let tilt_y = (player.vy * 0.035).clamp(-12.0, 12.0) as isize;

        // Flotación / oscilación natural de vuelo (hovering)
        let hover_y = (t * 6.0).sin() * 2.5;
        let cy = py + hover_y as isize;

        // 1. Llama de Plasma del Jetpack (animada, reactiva a la aceleración)
        let thrust_bonus = if player.jetpack_active { 8 } else { 0 };
        let flame_osc = ((t * 32.0).sin().abs() * (8.0 + thrust_bonus as f32)) as isize;
        let jet_x = px - dir * 16;
        let jet_y = cy - 2 - (tilt_y / 2);

        // Llamarada naranja fuego exterior
        self.draw_rect(buffer, jet_x - dir * (12 + flame_osc), jet_y - 3, (12 + flame_osc) as usize, 8, 0xFFFF4500);
        // Núcleo de plasma cian/blanco interior
        self.draw_rect(buffer, jet_x - dir * (6 + flame_osc / 2), jet_y - 1, (6 + flame_osc / 2) as usize, 4, 0xFF00FFFF);

        // Chispas de propulsión al volar
        if player.jetpack_active && ((t * 20.0) as usize % 2 == 0) {
            let spark_x = jet_x - dir * (14 + flame_osc);
            self.draw_circle(buffer, spark_x, jet_y + 1, 2, 0xFFFFD700);
        }

        // 2. Mochila Propulsora Jetpack (en la espalda del soldado)
        self.draw_rect(buffer, px - dir * 14, cy - 8 + tilt_y, 8, 18, 0xFF3D4452); // Cilindro metálico
        self.draw_rect(buffer, px - dir * 16, cy + 8 + tilt_y, 6, 6, 0xFF6C7A89);  // Tobera propulsora
        self.draw_circle(buffer, px - dir * 10, cy - 4 + tilt_y, 2, 0xFF00FF66);   // LED verde

        // 3. Piernas y Botas Gravitacionales con balanceo al volar
        let leg_sway = if player.jetpack_active { ((t * 14.0).sin() * 4.0) as isize } else { 0 };
        self.draw_rect(buffer, px - dir * 4 + leg_sway, cy + 10 + tilt_y, 6, 12, 0xFF2038EB); // Azul cobalto
        self.draw_rect(buffer, px + dir * 2 - leg_sway, cy + 8 + tilt_y, 6, 14, 0xFF2B303A);
        self.draw_rect(buffer, px + dir * 2 - leg_sway, cy + 20 + tilt_y, 8, 4, player.color); // Bota

        // 4. Torso y Peto de Combate Blindado (Armadura Exo-esqueleto)
        self.draw_rect(buffer, px - dir * 6, cy - 6 + tilt_y, 14, 16, player.color);
        self.draw_circle(buffer, px + dir * 2, cy - 1 + tilt_y, 3, 0xFFFFFFFF);
        self.draw_rect(buffer, px - dir * 8, cy - 8 + tilt_y, 10, 6, 0xFFD8D8E0);

        // 5. Cabeza y Casco Táctico con Visor Cibernético
        self.draw_circle(buffer, px + dir * 2, cy - 14 + tilt_y, 7, 0xFF2F3542);
        let visor_col = if player.id == 0 { 0xFF00FFFF } else { 0xFFFFD700 };
        self.draw_rect(buffer, px + dir * 4, cy - 16 + tilt_y, 6, 4, visor_col);
        self.draw_rect(buffer, px + dir * 6, cy - 16 + tilt_y, 2, 2, 0xFFFFFFFF);

        // Luz proyectada hacia adelante por el visor
        self.draw_point_light(buffer, px + dir * 14, cy - 14 + tilt_y, 22, visor_col, 0.4);

        // 6. Brazo y Rifle de Asalto de Plasma Pesado con retroceso (recoil)
        let recoil = if player.fire_timer > 0.06 { 3 } else { 0 };
        let gun_x = px + dir * (10 - recoil);
        let gun_y = cy - 2 + tilt_y;
        self.draw_rect(buffer, gun_x, gun_y, 18, 6, 0xFF737373);
        self.draw_rect(buffer, gun_x + dir * 14, gun_y + 1, 6, 4, 0xFFBBBBBB);
        self.draw_rect(buffer, gun_x + 4, gun_y - 3, 5, 3, 0xFFFF0055);

        // Fogonazo de disparo animado (Muzzle Flash)
        if player.fire_timer > 0.05 {
            let flash_x = gun_x + dir * 20;
            self.draw_circle(buffer, flash_x, gun_y + 3, 6, 0xFFFFFF00);
            self.draw_circle(buffer, flash_x, gun_y + 3, 3, 0xFFFFFFFF);
        }

        // 7. Satélites Orbitales de Apoyo Bivalvos (Final Mission Pods)
        for sat in player.satellites.iter() {
            let sat_x = (player.x + sat.angle.cos() * sat.distance) as isize;
            let sat_y = (player.y + sat.angle.sin() * sat.distance) as isize;
            let sat_color = if sat.is_locked { 0xFFFF0055 } else { 0xFF00E5FF };

            // Cápsula bivalva con armadura de titanio
            self.draw_circle(buffer, sat_x, sat_y, 9, 0xFF3A4250);
            self.draw_circle(buffer, sat_x, sat_y, 7, sat_color);
            self.draw_circle(buffer, sat_x, sat_y, 3, 0xFFFFFFFF);

            // Haz de mira láser direccional en 360 grados si está fijado (Satellite Lock)
            if sat.is_locked {
                for step in 1..=3 {
                    let d = step as f32 * 10.0;
                    let lx = sat_x + (sat.aim_angle.cos() * d) as isize;
                    let ly = sat_y + (sat.aim_angle.sin() * d) as isize;
                    self.draw_circle(buffer, lx, ly, if step == 3 { 3 } else { 1 }, 0xFFFF0055);
                }
            }
        }
    }

    /// Renderiza la interfaz de juego limpia (HUD) SIN BOTONES EN PANTALLA
    fn render_clean_hud(
        &self,
        buffer: &mut [u32],
        players: &[Player],
        stage_num: u8,
        _stage_name: &str,
    ) {
        if let Some(p1) = players.first() {
            // 1. Barra de Vida de P1
            let max_hp_w = 180;
            let hp_w = ((p1.health / p1.max_health).clamp(0.0, 1.0) * max_hp_w as f32) as usize;
            self.draw_rect(buffer, 24, 20, max_hp_w + 8, 16, 0xFF1B1B26);
            self.draw_rect(buffer, 28, 24, max_hp_w, 8, 0xFF3D3D4E);
            let hp_col = if p1.health > 40.0 { 0xFF00FF77 } else { 0xFFFF3344 };
            self.draw_rect(buffer, 28, 24, hp_w, 8, hp_col);

            // 2. Bombas EMP disponibles
            for b in 0..p1.bombs.min(6) {
                let bx = 28 + (b as isize * 18);
                self.draw_circle(buffer, bx, 44, 5, 0xFF00E5FF);
                self.draw_circle(buffer, bx, 44, 2, 0xFFFFFFFF);
            }

            // 3. Vidas restantes
            for v in 0..p1.lives.max(0).min(6) {
                let vx = 140 + (v as isize * 14);
                self.draw_circle(buffer, vx, 44, 4, 0xFF00D2FF);
            }

            // 4. Indicador del Nivel actual (1 a 8)
            let st_bar_x = self.width as isize - 160;
            self.draw_rect(buffer, st_bar_x, 20, 130, 20, 0x99111122);
            for dot in 1..=8 {
                let dot_col = if dot == stage_num {
                    0xFFFFFF00
                } else if dot < stage_num {
                    0xFF00FF77
                } else {
                    0xFF555566
                };
                self.draw_circle(buffer, st_bar_x + (dot as isize * 14), 30, 4, dot_col);
            }
        }
    }

    pub fn draw_rect(&self, buffer: &mut [u32], x: isize, y: isize, rw: usize, rh: usize, color: u32) {
        let w = self.width as isize;
        let h = self.height as isize;

        for ry in 0..rh as isize {
            let cy = y + ry;
            if cy < 0 || cy >= h {
                continue;
            }
            for rx in 0..rw as isize {
                let cx = x + rx;
                if cx >= 0 && cx < w {
                    buffer[(cy * w + cx) as usize] = color;
                }
            }
        }
    }

    pub fn draw_circle(&self, buffer: &mut [u32], cx: isize, cy: isize, radius: usize, color: u32) {
        let r_i = radius as isize;
        let r_sq = (radius * radius) as isize;
        let w = self.width as isize;
        let h = self.height as isize;

        for dy in -r_i..=r_i {
            let py = cy + dy;
            if py < 0 || py >= h {
                continue;
            }
            for dx in -r_i..=r_i {
                let px = cx + dx;
                if px >= 0 && px < w && (dx * dx + dy * dy) <= r_sq {
                    buffer[(py * w + px) as usize] = color;
                }
            }
        }
    }

    pub fn blend_additive(dst: u32, src: u32, intensity: f32) -> u32 {
        let factor = intensity.clamp(0.0, 1.0);
        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let sr = ((src >> 16) & 0xFF) as f32 * factor;
        let sg = ((src >> 8) & 0xFF) as f32 * factor;
        let sb = (src & 0xFF) as f32 * factor;

        let r = (dr as f32 + sr).min(255.0) as u32;
        let g = (dg as f32 + sg).min(255.0) as u32;
        let b = (db as f32 + sb).min(255.0) as u32;

        0xFF000000 | (r << 16) | (g << 8) | b
    }

    pub fn draw_point_light(&self, buffer: &mut [u32], lx: isize, ly: isize, radius: usize, color: u32, intensity: f32) {
        let r_i = radius as isize;
        let r_f = radius as f32;
        let w = self.width as isize;
        let h = self.height as isize;

        for dy in -r_i..=r_i {
            let py = ly + dy;
            if py < 0 || py >= h {
                continue;
            }
            let dy_sq = (dy * dy) as f32;
            for dx in -r_i..=r_i {
                let px = lx + dx;
                if px >= 0 && px < w {
                    let dist_sq = dx as f32 * dx as f32 + dy_sq;
                    let dist = dist_sq.sqrt();
                    if dist <= r_f {
                        let falloff = (1.0 - (dist / r_f)).powi(2) * intensity;
                        let idx = (py * w + px) as usize;
                        buffer[idx] = Self::blend_additive(buffer[idx], color, falloff);
                    }
                }
            }
        }
    }

    /// Renderiza naves y drones enemigos con torretas orientables
    pub fn draw_enemy(&self, buffer: &mut [u32], e: &Enemy) {
        let ex = e.x as isize;
        let ey = e.y as isize;
        let r = e.radius as usize;
        let t = self.anim_time;

        match e.enemy_type {
            EnemyType::PatrolDrone => {
                // Drone cruciforme centinela (+) de la IA
                self.draw_rect(buffer, ex - 16, ey - 4, 32, 8, 0xFF3E4354);
                self.draw_rect(buffer, ex - 4, ey - 16, 8, 32, 0xFF3E4354);
                // Ojo óptico rojo de escaneo central
                self.draw_circle(buffer, ex, ey, 5, 0xFFFF0033);
                self.draw_circle(buffer, ex, ey, 2, 0xFFFFFFFF);
                self.draw_point_light(buffer, ex, ey, 18, 0xFFFF0033, 0.5);
            }
            EnemyType::KamikazeWasp => {
                // Cápsula bivalva (almeja) en vuelo senoidal
                self.draw_rect(buffer, ex - 12, ey - 7, 24, 14, 0xFFC74C0C);
                self.draw_rect(buffer, ex - 8, ey - 4, 16, 8, 0xFF111111);
                self.draw_circle(buffer, ex - 12, ey, 4, 0xFFFFCC00);
                self.draw_point_light(buffer, ex - 12, ey, 14, 0xFFFF3300, 0.4);
            }
            EnemyType::LaserTurret => {
                // Torreta S-400 orientable montada en plataforma/tubería
                let base_y = if e.is_ceiling { ey - 10 } else { ey + 6 };
                self.draw_rect(buffer, ex - 16, base_y, 32, 10, 0xFF737373);
                self.draw_circle(buffer, ex, ey, 12, 0xFF4A4E69);

                // Cañón doble orientado hacia el jugador según aim_angle
                let barrel_len = 16.0;
                let bx = ex + (e.aim_angle.cos() * barrel_len) as isize;
                let by = ey + (e.aim_angle.sin() * barrel_len) as isize;
                self.draw_circle(buffer, bx, by, 4, 0xFFBBBBBB);
                self.draw_circle(buffer, bx, by, 2, 0xFFFF2020);
                self.draw_point_light(buffer, bx, by, 16, 0xFFFF2020, 0.4);
            }
            EnemyType::CyberCrab => {
                self.draw_circle(buffer, ex, ey, r, 0xFF5C677D);
                self.draw_circle(buffer, ex, ey, r.saturating_sub(4), 0xFF1F2430);
                self.draw_rect(buffer, ex - 22, ey - 14, 10, 6, 0xFF7D8597);
                self.draw_rect(buffer, ex - 22, ey + 8, 10, 6, 0xFF7D8597);
                self.draw_circle(buffer, ex, ey, 6, 0xFF00F5D4);
                self.draw_point_light(buffer, ex, ey, 24, 0xFF00F5D4, 0.5);
            }
            EnemyType::AsteroidLeech => {
                self.draw_circle(buffer, ex, ey, r, 0xFF7B2CBF);
                self.draw_circle(buffer, ex, ey, r.saturating_sub(5), 0xFF3C096C);
                let pulse = ((t * 6.0).sin() * 2.0) as usize;
                self.draw_circle(buffer, ex, ey, 5 + pulse, 0xFFFF0055);
                self.draw_circle(buffer, ex, ey, 2, 0xFFFFFFFF);
            }
            EnemyType::StealthStriker => {
                self.draw_rect(buffer, ex - 16, ey - 8, 32, 16, 0xFF14171E);
                self.draw_rect(buffer, ex - 12, ey - 4, 24, 8, 0xFF0B0D12);
                self.draw_rect(buffer, ex - 18, ey - 10, 4, 20, 0xFF00FFFF);
                self.draw_circle(buffer, ex, ey, 4, 0xFF00FFFF);
            }
        }
    }

    /// Renderiza jefes colosales (Boss)
    pub fn draw_boss(&self, buffer: &mut [u32], b: &Boss) {
        let bx = b.x as isize;
        let by = b.y as isize;
        let r = b.radius as usize;
        let t = self.anim_time;
        let w = self.width;

        match b.id {
            BossId::Stage1TitanWarcrawler => {
                // Fortaleza móvil de asedio mecánico (Titan Warcrawler)
                // Orugas y chasis inferior pesado
                self.draw_rect(buffer, bx - 60, by - 55, 120, 110, 0xFF2A2E3B);
                self.draw_rect(buffer, bx - 70, by + 40, 140, 22, 0xFF181C24); // Orugas
                self.draw_rect(buffer, bx - 65, by + 45, 130, 4, 0xFF737373);  // Eslabones

                // Placas de blindaje con remaches y franjas de advertencia
                self.draw_rect(buffer, bx - 55, by - 48, 100, 20, 0xFFC74C0C);
                self.draw_rect(buffer, bx - 55, by - 48, 100, 4, 0xFF111111);

                // Torreta superior gemela orientable
                self.draw_rect(buffer, bx - 30, by - 70, 40, 18, 0xFF4A5568);
                self.draw_circle(buffer, bx - 35, by - 62, 7, 0xFFFF2020);

                // Núcleo cuántico central de IA (expuesto según fase)
                let core_color = if b.phase >= 2 { 0xFFFF0055 } else { 0xFFFF5500 };
                let core_r = 16 + ((t * 6.0).sin().abs() * 4.0) as usize;
                self.draw_circle(buffer, bx, by, core_r, core_color);
                self.draw_circle(buffer, bx, by, 8, 0xFFFFFFFF);
                self.draw_point_light(buffer, bx, by, 65, core_color, 0.8);

                // Chispas de daño si tiene poca vida
                if b.health < b.max_health * 0.5 && ((t * 18.0) as usize % 2 == 0) {
                    self.draw_circle(buffer, bx - 40, by - 20, 3, 0xFFFFD700);
                    self.draw_circle(buffer, bx + 20, by + 10, 4, 0xFFFF4500);
                }
            }
            _ => {
                // Casco blindado masivo
                self.draw_circle(buffer, bx, by, r, 0xFF2B2D42);
                self.draw_circle(buffer, bx, by, r.saturating_sub(6), 0xFF1A1B29);
                self.draw_rect(buffer, bx - (r as isize / 2), by - (r as isize / 2), r, r, 0xFF3D405B);
                self.draw_rect(buffer, bx - r as isize - 16, by - 24, 22, 10, 0xFF495057);
                self.draw_rect(buffer, bx - r as isize - 16, by + 14, 22, 10, 0xFF495057);

                let core_color = if (t * 4.0).sin() > 0.0 { 0xFFFF0055 } else { 0xFFFF5500 };
                self.draw_circle(buffer, bx, by, 22, core_color);
                self.draw_circle(buffer, bx, by, 12, 0xFFFFFFFF);
                self.draw_point_light(buffer, bx, by, r + 70, core_color, 0.85);
            }
        }

        // Barra de salud superior del Jefe
        let max_w = 520;
        let cur_w = ((b.health / b.max_health).clamp(0.0, 1.0) * max_w as f32) as usize;
        let bar_x = (w as isize - max_w as isize) / 2;
        self.draw_rect(buffer, bar_x - 6, 22, max_w + 12, 22, 0xFF550000);
        self.draw_rect(buffer, bar_x - 2, 24, max_w + 4, 18, 0xFF1A0000);
        self.draw_rect(buffer, bar_x, 26, cur_w, 14, 0xFFFF2244);
        self.draw_rect(buffer, bar_x, 26, cur_w, 3, 0xFFFF99AA);
    }

    /// Renderizador simple de texto en píxeles (fuente matricial 5x7)
    fn draw_simple_text(&self, buffer: &mut [u32], text: &str, start_x: isize, start_y: isize, color: u32, scale: usize) {
        let mut cur_x = start_x;
        for ch in text.chars() {
            let glyph = Self::get_char_glyph(ch);
            for row in 0..7 {
                for col in 0..5 {
                    if (glyph[row] & (1 << (4 - col))) != 0 {
                        self.draw_rect(
                            buffer,
                            cur_x + (col * scale) as isize,
                            start_y + (row * scale) as isize,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
            cur_x += (6 * scale) as isize;
        }
    }

    fn get_char_glyph(ch: char) -> [u8; 7] {
        match ch.to_ascii_uppercase() {
            'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
            'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'G' => [0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001],
            'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
            'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
            'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
            '0'..='9' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
            '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
            _ => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        }
    }
}
