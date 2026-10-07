//! Rasterizador Gráfico de Alto Rendimiento para IA Rebellion
//! Experiencia visual inspirada en Final Mission (Natsume) y Abadox:
//! - Motor Gráfico Vectorial Moderno con Anti-Aliasing Subpixel (SDF),
//!   luces volumétricas cuadráticas, plumas de plasma multicapa y retículas holográficas HUD.
//! - Trayecto Multidireccional Dinámico: Vuelo horizontal, ascenso vertical (torres/rascacielos),
//!   descenso vertical (fundiciones subterráneas/magma) y autopista aérea de asedio.
//! - Arquetipos de IA Rebelde: Sintéticos bípedos (katana/rifle), ciber-plantas biomecánicas
//!   (zarcillos Abadox, torretas de esporas pulsantes, flores navaja), drones de asalto y mechas leviatanes.
//! - Intro Splash Cinemática: Diamon Black - Powered by Rust.
//! - HUD Limpio Superior SIN BOTONES VIRTUALES.

use crate::boss::{Boss, BossId};
use crate::bullet::{Bullet, BulletType};
use crate::enemy::{Enemy, EnemyType};
use crate::level::StagePhase;
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
    pub scroll_y: f32,
    pub scroll_vx: f32,
    pub scroll_vy: f32,
    pub stage_phase: StagePhase,
    pub stage_progress: f32,
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
            scroll_y: 0.0,
            scroll_vx: 160.0,
            scroll_vy: 0.0,
            stage_phase: StagePhase::HorizontalRight,
            stage_progress: 0.0,
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
        self.scroll_x += self.scroll_vx * dt;
        self.scroll_y += self.scroll_vy * dt;
        let w_f = self.width as f32;
        let h_f = self.height as f32;

        let rel_vx = if self.scroll_vx.abs() > 10.0 { -self.scroll_vx / 160.0 } else { -0.15 };
        let rel_vy = if self.scroll_vy.abs() > 10.0 { -self.scroll_vy / 160.0 } else { 0.0 };

        for s in self.stars.iter_mut() {
            s.x += rel_vx * s.speed * dt;
            s.y += rel_vy * s.speed * dt;

            if s.x < 0.0 { s.x += w_f; }
            else if s.x >= w_f { s.x -= w_f; }

            if s.y < 0.0 { s.y += h_f; }
            else if s.y >= h_f { s.y -= h_f; }
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
        let t_box_w = 420;
        let t_box_h = 56;
        let t_x = cx - t_box_w / 2;
        let t_y = cy - 90;

        self.draw_rect(buffer, t_x, t_y, t_box_w as usize, t_box_h as usize, 0xFF181C26);
        self.draw_rect(buffer, t_x + 2, t_y + 2, (t_box_w - 4) as usize, 2, 0xFF4A5568);
        self.draw_rect(buffer, t_x + 2, t_y + (t_box_h - 4), (t_box_w - 4) as usize, 2, 0xFF0A0D14);

        self.draw_simple_text(buffer, "DIAMON BLACK", cx - 130, cy - 76, 0xFFE2E8F0, 3);

        // --- 2. LOGO BADGE: POWERED BY RUST ---
        let badge_w = 360;
        let badge_h = 68;
        let b_x = cx - badge_w / 2;
        let b_y = cy + 10;

        self.draw_rect(buffer, b_x, b_y, badge_w as usize, badge_h as usize, 0xFF1E232E);
        self.draw_rect(buffer, b_x + 3, b_y + 3, (badge_w - 6) as usize, (badge_h - 6) as usize, 0xFF2A3140);
        self.draw_rect(buffer, b_x + 4, b_y + 4, (badge_w - 8) as usize, 2, 0xFF5A667E);

        let gear_x = b_x + 48;
        let gear_y = b_y + 34;

        for d in 0..8 {
            let rad = (d as f32) * (std::f32::consts::PI / 4.0) + timer * 0.8;
            let tx = gear_x + (rad.cos() * 22.0) as isize;
            let ty = gear_y + (rad.sin() * 22.0) as isize;
            self.draw_circle(buffer, tx, ty, 5, 0xFFFF4500);
            self.draw_circle(buffer, tx, ty, 3, 0xFFFFCC00);
        }

        self.draw_circle(buffer, gear_x, gear_y, 20, 0xFFFF4500);
        self.draw_circle(buffer, gear_x, gear_y, 16, 0xFF1E232E);
        self.draw_rect(buffer, gear_x - 5, gear_y - 8, 4, 16, 0xFFFFD700);
        self.draw_rect(buffer, gear_x - 5, gear_y - 8, 10, 4, 0xFFFFD700);
        self.draw_rect(buffer, gear_x + 1, gear_y - 8, 4, 8, 0xFFFFD700);
        self.draw_rect(buffer, gear_x - 5, gear_y - 1, 10, 3, 0xFFFFD700);
        self.draw_rect(buffer, gear_x + 1, gear_y + 2, 4, 6, 0xFFFFD700);

        self.draw_point_light(buffer, gear_x, gear_y, 45, 0xFFFF4500, 0.7);

        self.draw_simple_text(buffer, "POWERED BY", b_x + 95, b_y + 14, 0xFFA0AEC0, 1);
        self.draw_simple_text(buffer, "R U S T", b_x + 95, b_y + 32, 0xFFFF7700, 3);

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

        // 1. Escenario multidireccional enriquecido
        self.render_stage_environment(buffer, stage_num);

        // 2. Campo de estrellas reactivo
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

        // 7. Personaje: Comando Cibernético Moderno en todos los niveles
        for player in players.iter() {
            if !player.active {
                continue;
            }
            self.draw_modern_commando(buffer, player);
        }

        // 8. HUD Limpio Superior SIN BOTONES VIRTUALES
        self.render_clean_hud(buffer, players, stage_num, stage_name);
    }

    /// Renderiza escenarios temáticos basados en Final Mission NES y trayectos multidireccionales
    fn render_stage_environment(&self, buffer: &mut [u32], stage: u8) {
        let w = self.width;
        let h = self.height;
        let s_x = self.scroll_x;
        let s_y = self.scroll_y;
        let t = self.anim_time;
        let progress = self.stage_progress.clamp(0.0, 1.0);

        match stage {
            1 => {
                buffer.fill(0xFF000000);

                match self.stage_phase {
                    StagePhase::HorizontalRight => {
                        if progress < 0.28 {
                            // === SECCIÓN 1: Lower Ruined Street & Pipe Platforms ===
                            for nx in 0..(w / 16 + 2) {
                                let rx = (nx * 16) as isize;
                                let cloud_h = 24 + (((nx as f32 * 0.4 + t * 0.5).sin() * 8.0) as usize);
                                self.draw_rect(buffer, rx, 0, 16, cloud_h, 0xFF44009B);
                                self.draw_rect(buffer, rx, 0, 16, cloud_h.saturating_sub(6), 0xFF7F00EF);
                            }

                            let bld_offset1 = (s_x * 0.25) as usize % 96;
                            for i in 0..(w / 96 + 2) {
                                let bx = (i * 96) as isize - bld_offset1 as isize;
                                let bld_h = 160 + ((i * 47) % 110);
                                let by = h as isize - 70 - bld_h as isize;
                                self.draw_rect(buffer, bx, by, 76, bld_h, 0xFF183C5C);
                                self.draw_rect(buffer, bx + 36, by - 18, 3, 18, 0xFF737373);
                                for wy in 0..(bld_h / 28) {
                                    self.draw_rect(buffer, bx + 20, by + 16 + (wy * 28) as isize, 4, 4, 0xFFFBFBFB);
                                }
                            }

                            let bld_offset2 = (s_x * 0.50) as usize % 120;
                            for i in 0..(w / 120 + 2) {
                                let bx = (i * 120) as isize - bld_offset2 as isize;
                                let bld_h = 210 + ((i * 31) % 95);
                                let by = h as isize - 65 - bld_h as isize;
                                self.draw_rect(buffer, bx, by, 90, bld_h, 0xFF44009B);
                                for wx in 0..7 {
                                    self.draw_rect(buffer, bx + 10 + (wx * 11) as isize, by + 12, 5, bld_h - 24, 0xFF7F00EF);
                                }
                            }

                            let pipe_scroll = (s_x * 0.85) as usize % 64;
                            let top_pipe_y = 50;
                            let bot_pipe_y = h as isize - 64;

                            self.draw_rect(buffer, 0, top_pipe_y, w, 14, 0xFF737373);
                            self.draw_rect(buffer, 0, top_pipe_y + 2, w, 5, 0xFFBBBBBB);
                            self.draw_rect(buffer, 0, top_pipe_y + 3, w, 2, 0xFFFBFBFB);

                            self.draw_rect(buffer, 0, bot_pipe_y, w, 16, 0xFF737373);
                            self.draw_rect(buffer, 0, bot_pipe_y + 3, w, 6, 0xFFBBBBBB);
                            self.draw_rect(buffer, 0, bot_pipe_y + 4, w, 2, 0xFFFBFBFB);

                            for f in 0..(w / 64 + 2) {
                                let fx = (f * 64) as isize - pipe_scroll as isize;
                                self.draw_rect(buffer, fx, top_pipe_y - 2, 6, 18, 0xFFBBBBBB);
                                self.draw_rect(buffer, fx, bot_pipe_y - 2, 6, 20, 0xFFBBBBBB);
                            }

                            let girder_scroll = (s_x * 0.85) as usize % 80;
                            for g in 0..(w / 80 + 2) {
                                let gx = (g * 80) as isize - girder_scroll as isize;
                                let gy = bot_pipe_y - 18;
                                self.draw_rect(buffer, gx, gy, 74, 16, 0xFFC74C0C);
                                self.draw_rect(buffer, gx, gy, 74, 2, 0xFFA30000);
                                self.draw_rect(buffer, gx, gy + 14, 74, 2, 0xFFA30000);
                                for x_i in 0..3 {
                                    self.draw_rect(buffer, gx + 8 + (x_i * 20) as isize, gy + 3, 10, 10, 0xFF181C26);
                                }
                            }

                            let ground_y = h as isize - 38;
                            self.draw_rect(buffer, 0, ground_y, w, 38, 0xFF877000);
                            self.draw_rect(buffer, 0, ground_y, w, 3, 0xFFFBD7A7);
                            for deb in (0..w).step_by(18) {
                                let dy = ground_y + 6 + ((deb * 13) % 24) as isize;
                                self.draw_rect(buffer, deb as isize, dy, 5, 4, 0xFF44009B);
                                self.draw_rect(buffer, deb as isize + 2, dy + 1, 2, 2, 0xFFFBFBFB);
                            }
                        } else {
                            // === SECCIÓN 4: Elevated Sky-Highway Leading to TITAN-01 Warcrawler ===
                            let dist_offset = (s_x * 0.20) as usize % 110;
                            for d_i in 0..(w / 110 + 2) {
                                let dx = (d_i * 110) as isize - dist_offset as isize;
                                self.draw_rect(buffer, dx, (h / 2) as isize, 90, (h / 2) as usize, 0xFF0F1726);
                            }

                            let storm_flash = (t * 5.0).sin() > 0.85;
                            if storm_flash {
                                self.draw_rect(buffer, 0, 0, w, 60, 0x339400D3);
                            }

                            let hw_y = h as isize - 75;
                            self.draw_rect(buffer, 0, hw_y, w, 55, 0xFF2B303A);
                            self.draw_rect(buffer, 0, hw_y, w, 4, 0xFF737373);
                            self.draw_rect(buffer, 0, hw_y + 4, w, 2, 0xFFFBFBFB);

                            let lane_scroll = (s_x * 1.10) as usize % 48;
                            for l_i in 0..(w / 48 + 2) {
                                let lx = (l_i * 48) as isize - lane_scroll as isize;
                                self.draw_rect(buffer, lx, hw_y + 24, 24, 3, 0xFFFFD700);
                                self.draw_rect(buffer, lx, hw_y + 42, 18, 2, 0xFFE2E8F0);
                            }

                            let rail_scroll = (s_x * 1.10) as usize % 40;
                            self.draw_rect(buffer, 0, hw_y - 12, w, 12, 0xFFC74C0C);
                            for r_i in 0..(w / 40 + 2) {
                                let rx = (r_i * 40) as isize - rail_scroll as isize;
                                self.draw_rect(buffer, rx, hw_y - 12, 14, 12, 0xFF111111);
                            }

                            for post_i in 0..(w / 120 + 2) {
                                let px = (post_i * 120) as isize - rail_scroll as isize;
                                self.draw_rect(buffer, px, hw_y - 30, 4, 18, 0xFFBBBBBB);
                                let alarm_on = ((t * 8.0) as usize + post_i) % 2 == 0;
                                let al_col = if alarm_on { 0xFFFF0033 } else { 0xFF440011 };
                                self.draw_circle(buffer, px + 2, hw_y - 32, 4, al_col);
                            }
                        }
                    }
                    StagePhase::AscendUp => {
                        // === SECCIÓN 2: Vertical ASCENT (scrolling UP) through Skyscraper Shaft ===
                        buffer.fill(0xFF0F081D);

                        let shaft_bg_scroll = (s_y.abs() * 0.40) as usize % 80;
                        for y_b in 0..(h / 80 + 2) {
                            let by = (y_b * 80) as isize + shaft_bg_scroll as isize - 80;
                            self.draw_rect(buffer, 50, by, w - 100, 32, 0xFF1B0B33);
                            for win_x in (60..(w - 60)).step_by(28) {
                                self.draw_rect(buffer, win_x as isize, by + 8, 8, 16, 0xFF6500B8);
                            }
                        }

                        let cable_scroll = (s_y.abs() * 0.85) as usize % 40;
                        let cable1_x = (w as f32 * 0.32) as isize;
                        let cable2_x = (w as f32 * 0.68) as isize;
                        self.draw_rect(buffer, cable1_x, 0, 3, h, 0xFF4A5568);
                        self.draw_rect(buffer, cable2_x, 0, 3, h, 0xFF4A5568);

                        for y_beam in 0..(h / 120 + 2) {
                            let beam_y = (y_beam * 120) as isize + cable_scroll as isize - 40;
                            self.draw_rect(buffer, 0, beam_y, 80, 10, 0xFFC74C0C);
                            self.draw_rect(buffer, 0, beam_y + 2, 80, 2, 0xFFFBD7A7);
                            self.draw_rect(buffer, w as isize - 80, beam_y, 80, 10, 0xFFC74C0C);
                            self.draw_rect(buffer, w as isize - 80, beam_y + 2, 80, 2, 0xFFFBD7A7);
                        }

                        self.draw_rect(buffer, 0, 0, 48, h, 0xFF2B0A4E);
                        self.draw_rect(buffer, 44, 0, 4, h, 0xFF737373);
                        self.draw_rect(buffer, 46, 0, 2, h, 0xFFBBBBBB);

                        self.draw_rect(buffer, w as isize - 48, 0, 48, h, 0xFF2B0A4E);
                        self.draw_rect(buffer, w as isize - 48, 0, 4, h, 0xFF737373);
                        self.draw_rect(buffer, w as isize - 46, 0, 2, h, 0xFFBBBBBB);

                        let bracket_scroll = (s_y.abs() * 0.90) as usize % 70;
                        for b_i in 0..(h / 70 + 2) {
                            let by = (b_i * 70) as isize + bracket_scroll as isize - 70;
                            self.draw_rect(buffer, 36, by, 12, 16, 0xFFBBBBBB);
                            self.draw_circle(buffer, 42, by + 8, 3, 0xFFFF2020);
                            self.draw_rect(buffer, w as isize - 48, by, 12, 16, 0xFFBBBBBB);
                            self.draw_circle(buffer, w as isize - 42, by + 8, 3, 0xFFFF2020);
                        }
                    }
                    StagePhase::DescendDown => {
                        // === SECCIÓN 3: Vertical DESCENT (scrolling DOWN) into Subterranean Foundry ===
                        buffer.fill(0xFF140803);

                        let magma_pulse = ((t * 4.0).sin().abs() * 30.0) as usize;
                        let abyss_h = 90 + magma_pulse;
                        self.draw_rect(buffer, 0, h as isize - abyss_h as isize, w, abyss_h, 0x44FF3300);
                        self.draw_rect(buffer, 0, h as isize - 40, w, 40, 0x77FF6600);
                        self.draw_rect(buffer, 0, h as isize - 15, w, 15, 0xAAFFAA00);

                        let foundry_scroll = (s_y.abs() * 0.80) as usize % 90;
                        for y_f in 0..(h / 90 + 2) {
                            let fy = (y_f * 90) as isize - foundry_scroll as isize;
                            self.draw_rect(buffer, 54, fy, w - 108, 14, 0xFF2E1A11);
                            self.draw_rect(buffer, 54, fy + 4, w - 108, 4, 0xFFFF4500);
                        }

                        self.draw_rect(buffer, 0, 0, 52, h, 0xFF24140D);
                        self.draw_rect(buffer, 48, 0, 4, h, 0xFFFF4500);
                        self.draw_rect(buffer, w as isize - 52, 0, 52, h, 0xFF24140D);
                        self.draw_rect(buffer, w as isize - 52, 0, 4, h, 0xFFFF4500);

                        for v_i in 0..(h / 65 + 2) {
                            let vy = (v_i * 65) as isize - foundry_scroll as isize;
                            self.draw_rect(buffer, 38, vy, 14, 18, 0xFF73503C);
                            self.draw_circle(buffer, 45, vy + 9, 3, 0xFFFFCC00);
                            self.draw_rect(buffer, w as isize - 52, vy, 14, 18, 0xFF73503C);
                            self.draw_circle(buffer, w as isize - 45, vy + 9, 3, 0xFFFFCC00);
                        }
                    }
                    StagePhase::BossEncounter => {
                        // === ARENA DE JEFE: TITAN-01 Warcrawler Arrival ===
                        let storm_flash = (t * 8.0).sin() > 0.70;
                        let sky_col = if storm_flash { 0xFF220033 } else { 0xFF0A0410 };
                        buffer.fill(sky_col);

                        let hw_y = h as isize - 75;
                        self.draw_rect(buffer, 0, hw_y, w, 55, 0xFF1E232E);
                        self.draw_rect(buffer, 0, hw_y, w, 4, 0xFFFF0055);
                        for grid_x in (0..w).step_by(50) {
                            self.draw_rect(buffer, grid_x as isize, hw_y, 2, 55, 0x44FF0055);
                        }
                    }
                }
            }
            // Nivel 2: Fábrica de Drones (Earth)
            2 => {
                buffer.fill(0xFF140D07);
                let beam_offset = (s_x * 0.4) as usize % 120;
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
            // Nivel 3: Mars Cyber-Foundry (Mars)
            3 => {
                buffer.fill(0xFF1A0802);
                let canyon_scroll = (s_x * 0.35) as usize % 100;
                for c in 0..(w / 100 + 2) {
                    let cx = (c * 100) as isize - canyon_scroll as isize;
                    let c_h = 140 + ((c * 43) % 120);
                    let cy = h as isize - c_h as isize;
                    self.draw_rect(buffer, cx, cy, 85, c_h, 0xFF541B08);
                    self.draw_rect(buffer, cx + 10, cy + 15, 65, 4, 0xFFFF3300);
                }
            }
            // Nivel 4: Europa Sub-Glacial Network (Jupiter)
            4 => {
                buffer.fill(0xFF03141C);
                let ice_scroll = (s_x * 0.5) as usize % 80;
                for ic in 0..(w / 80 + 2) {
                    let ix = (ic * 80) as isize - ice_scroll as isize;
                    let stalac = 50 + ((ic * 29) % 60);
                    self.draw_rect(buffer, ix, 0, 24, stalac, 0xFF0A3C52);
                    self.draw_rect(buffer, ix + 4, 0, 8, stalac - 6, 0xFF00E5FF);
                }
            }
            // Nivel 5: Hephaestus Solar Bastion (Mercury / Solar Orbit)
            5 => {
                buffer.fill(0xFF1F0F00);
                let flare = ((t * 2.0).sin().abs() * 30.0) as isize;
                self.draw_rect(buffer, 0, 0, w, 40 + flare as usize, 0xFFFF7700);
                self.draw_rect(buffer, 0, 0, w, 20 + (flare / 2) as usize, 0xFFFFDD00);
            }
            // Nivel 6: Titan Methane Spire (Saturn)
            6 => {
                buffer.fill(0xFF191004);
                self.draw_rect(buffer, 0, 30, w, 8, 0xFFC29B38);
                self.draw_rect(buffer, 0, 34, w, 2, 0xFFF7E294);
            }
            // Nivel 7: Nemesis Mothership Fleet (Deep Space)
            7 => {
                buffer.fill(0xFF0D0614);
                let hull_offset = (s_x * 0.4) as usize % 140;
                for h_i in 0..(w / 140 + 2) {
                    let hx = (h_i * 140) as isize - hull_offset as isize;
                    self.draw_rect(buffer, hx, h as isize - 100, 120, 100, 0xFF1E172E);
                    self.draw_rect(buffer, hx + 20, h as isize - 80, 80, 14, 0xFFFF007F);
                }
            }
            // Nivel 8: Quantum Singularity Core (The AI Overmind)
            _ => {
                buffer.fill(0xFF000000);
                let grid_offset = (s_x * 0.6) as usize % 50;
                for gx in 0..(w / 50 + 2) {
                    let rx = (gx * 50) as isize - grid_offset as isize;
                    self.draw_rect(buffer, rx, 0, 1, h, 0x44FF0033);
                }
                let vortex_r = 50 + ((t * 4.0).sin().abs() * 14.0) as usize;
                self.draw_point_light(buffer, (w / 2) as isize, (h / 2) as isize, vortex_r, 0xFFFF0055, 0.7);
            }
        }
    }

    // ==============================================================================
    // Primitivas Vectoriales Modernas (SDF Anti-Aliasing, Iluminación Volumétrica)
    // ==============================================================================

    /// Mezclado Alpha lineal suave: src sobre dst
    #[inline(always)]
    pub fn blend_alpha(dst: u32, src: u32, alpha: f32) -> u32 {
        let a = (alpha.clamp(0.0, 1.0) * 256.0) as u32;
        if a == 0 { return dst; }
        if a >= 256 { return src; }
        let inv_a = 256 - a;

        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let sr = (src >> 16) & 0xFF;
        let sg = (src >> 8) & 0xFF;
        let sb = src & 0xFF;

        let r = ((sr * a + dr * inv_a) >> 8).min(255);
        let g = ((sg * a + dg * inv_a) >> 8).min(255);
        let b = ((sb * a + db * inv_a) >> 8).min(255);

        0xFF000000 | (r << 16) | (g << 8) | b
    }

    /// Mezclado aditivo de alta luminosidad con saturación para Bloom y Plasma
    #[inline(always)]
    pub fn blend_additive(dst: u32, src: u32, intensity: f32) -> u32 {
        let factor = intensity.clamp(0.0, 1.0);
        if factor <= 0.001 { return dst; }
        let dr = (dst >> 16) & 0xFF;
        let dg = (dst >> 8) & 0xFF;
        let db = dst & 0xFF;

        let sr = (((src >> 16) & 0xFF) as f32 * factor) as u32;
        let sg = (((src >> 8) & 0xFF) as f32 * factor) as u32;
        let sb = ((src & 0xFF) as f32 * factor) as u32;

        let r = (dr + sr).min(255);
        let g = (dg + sg).min(255);
        let b = (db + sb).min(255);

        0xFF000000 | (r << 16) | (g << 8) | b
    }

    /// Interpolación lineal de colores ARGB
    #[inline(always)]
    pub fn lerp_color(c1: u32, c2: u32, t: f32) -> u32 {
        let factor = t.clamp(0.0, 1.0);
        let r1 = ((c1 >> 16) & 0xFF) as f32;
        let g1 = ((c1 >> 8) & 0xFF) as f32;
        let b1 = (c1 & 0xFF) as f32;

        let r2 = ((c2 >> 16) & 0xFF) as f32;
        let g2 = ((c2 >> 8) & 0xFF) as f32;
        let b2 = (c2 & 0xFF) as f32;

        let r = (r1 + (r2 - r1) * factor) as u32;
        let g = (g1 + (g2 - g1) * factor) as u32;
        let b = (b1 + (b2 - b1) * factor) as u32;

        0xFF000000 | (r << 16) | (g << 8) | b
    }

    /// Círculo vectorial con suavizado de bordes subpixel (SDF Anti-Aliasing)
    pub fn draw_aa_circle(
        &self,
        buffer: &mut [u32],
        cx: f32,
        cy: f32,
        radius: f32,
        fill_color: u32,
        border_color: u32,
        border_width: f32,
    ) {
        let w = self.width as isize;
        let h = self.height as isize;
        let min_x = (cx - radius - 1.0).floor().max(0.0) as isize;
        let max_x = (cx + radius + 1.0).ceil().min(w as f32 - 1.0) as isize;
        let min_y = (cy - radius - 1.0).floor().max(0.0) as isize;
        let max_y = (cy + radius + 1.0).ceil().min(h as f32 - 1.0) as isize;

        for y in min_y..=max_y {
            let dy = y as f32 - cy;
            let dy2 = dy * dy;
            let row_idx = y * w;
            for x in min_x..=max_x {
                let dx = x as f32 - cx;
                let dist = (dx * dx + dy2).sqrt();
                let edge_dist = radius - dist;

                if edge_dist >= 0.5 {
                    let pixel_idx = (row_idx + x) as usize;
                    if border_width > 0.0 && edge_dist <= border_width {
                        let border_factor = ((border_width - edge_dist) + 0.5).clamp(0.0, 1.0);
                        buffer[pixel_idx] = Self::lerp_color(fill_color, border_color, border_factor);
                    } else {
                        buffer[pixel_idx] = fill_color;
                    }
                } else if edge_dist > -0.5 {
                    let alpha = edge_dist + 0.5;
                    let col = if border_width > 0.0 { border_color } else { fill_color };
                    let pixel_idx = (row_idx + x) as usize;
                    buffer[pixel_idx] = Self::blend_alpha(buffer[pixel_idx], col, alpha);
                }
            }
        }
    }

    /// Cápsula / Segmento curvo anti-aliased (extremidades, tentáculos, katanas, cañones)
    pub fn draw_aa_capsule(
        &self,
        buffer: &mut [u32],
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        radius: f32,
        color: u32,
    ) {
        let w = self.width as isize;
        let h = self.height as isize;
        let min_x = (x0.min(x1) - radius - 1.0).floor().max(0.0) as isize;
        let max_x = (x0.max(x1) + radius + 1.0).ceil().min(w as f32 - 1.0) as isize;
        let min_y = (y0.min(y1) - radius - 1.0).floor().max(0.0) as isize;
        let max_y = (y0.max(y1) + radius + 1.0).ceil().min(h as f32 - 1.0) as isize;

        let seg_dx = x1 - x0;
        let seg_dy = y1 - y0;
        let seg_len_sq = seg_dx * seg_dx + seg_dy * seg_dy;

        for y in min_y..=max_y {
            let py = y as f32;
            let row_idx = y * w;
            for x in min_x..=max_x {
                let px = x as f32;
                let t = if seg_len_sq > 0.0001 {
                    (((px - x0) * seg_dx + (py - y0) * seg_dy) / seg_len_sq).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let close_x = x0 + t * seg_dx;
                let close_y = y0 + t * seg_dy;
                let dist = ((px - close_x) * (px - close_x) + (py - close_y) * (py - close_y)).sqrt();
                let edge_dist = radius - dist;

                if edge_dist >= 0.5 {
                    buffer[(row_idx + x) as usize] = color;
                } else if edge_dist > -0.5 {
                    let alpha = edge_dist + 0.5;
                    let pixel_idx = (row_idx + x) as usize;
                    buffer[pixel_idx] = Self::blend_alpha(buffer[pixel_idx], color, alpha);
                }
            }
        }
    }

    /// Luz puntual volumétrica cuadrática sin sqrt() dentro del bucle interno
    pub fn draw_volumetric_light(
        &self,
        buffer: &mut [u32],
        lx: f32,
        ly: f32,
        radius: f32,
        color: u32,
        intensity: f32,
    ) {
        let w = self.width as isize;
        let h = self.height as isize;
        let r2 = radius * radius;
        let inv_r2 = 1.0 / r2;

        let min_x = (lx - radius).floor().max(0.0) as isize;
        let max_x = (lx + radius).ceil().min(w as f32 - 1.0) as isize;
        let min_y = (ly - radius).floor().max(0.0) as isize;
        let max_y = (ly + radius).ceil().min(h as f32 - 1.0) as isize;

        for y in min_y..=max_y {
            let dy = y as f32 - ly;
            let dy2 = dy * dy;
            let row_idx = y * w;
            for x in min_x..=max_x {
                let dx = x as f32 - lx;
                let dist2 = dx * dx + dy2;
                if dist2 <= r2 {
                    let factor = (1.0 - dist2 * inv_r2).powi(2) * intensity;
                    let pixel_idx = (row_idx + x) as usize;
                    buffer[pixel_idx] = Self::blend_additive(buffer[pixel_idx], color, factor);
                }
            }
        }
    }

    /// Pluma de plasma de doble capa para propulsores jetpack y postquemadores
    pub fn draw_plasma_plume(
        &self,
        buffer: &mut [u32],
        start_x: f32,
        start_y: f32,
        dir_x: f32,
        dir_y: f32,
        length: f32,
        max_width: f32,
        core_color: u32,
        outer_color: u32,
    ) {
        let steps = 12;
        let perp_x = -dir_y;
        let perp_y = dir_x;

        for i in 0..steps {
            let t_ratio = i as f32 / steps as f32;
            let seg_x = start_x + dir_x * (t_ratio * length);
            let seg_y = start_y + dir_y * (t_ratio * length);

            let shock = (self.anim_time * 36.0 + i as f32 * 1.5).sin() * 0.25;
            let w_outer = (max_width * (1.0 - t_ratio) * (1.0 + shock)).max(0.5);
            let w_core = (w_outer * 0.45).max(0.5);

            let p1_out_x = seg_x + perp_x * w_outer;
            let p1_out_y = seg_y + perp_y * w_outer;
            let p2_out_x = seg_x - perp_x * w_outer;
            let p2_out_y = seg_y - perp_y * w_outer;

            let p1_core_x = seg_x + perp_x * w_core;
            let p1_core_y = seg_y + perp_y * w_core;
            let p2_core_x = seg_x - perp_x * w_core;
            let p2_core_y = seg_y - perp_y * w_core;

            self.draw_aa_capsule(buffer, p1_out_x, p1_out_y, p2_out_x, p2_out_y, 1.8, outer_color);
            self.draw_aa_capsule(buffer, p1_core_x, p1_core_y, p2_core_x, p2_core_y, 1.2, core_color);
        }

        self.draw_volumetric_light(buffer, start_x, start_y, max_width * 3.2, outer_color, 0.7);
    }

    /// Retícula holográfica de puntería HUD proyectada en el espacio de juego
    pub fn draw_holographic_reticle(
        &self,
        buffer: &mut [u32],
        cx: f32,
        cy: f32,
        radius: f32,
        color: u32,
        is_locked: bool,
    ) {
        let t = self.anim_time;
        let rot = t * 2.5;

        for i in 0..4 {
            let base_a = rot + (i as f32) * (std::f32::consts::PI * 0.5);
            let a1 = base_a;
            let a2 = base_a + 0.4;

            let x1 = cx + a1.cos() * radius;
            let y1 = cy + a1.sin() * radius;
            let x2 = cx + a2.cos() * radius;
            let y2 = cy + a2.sin() * radius;
            self.draw_aa_capsule(buffer, x1, y1, x2, y2, 1.2, color);
        }

        let inner_r = radius * 0.55;
        let b_len = 5.0;
        self.draw_aa_capsule(buffer, cx, cy - inner_r, cx, cy - inner_r + b_len, 1.0, color);
        self.draw_aa_capsule(buffer, cx, cy + inner_r, cx, cy + inner_r - b_len, 1.0, color);
        self.draw_aa_capsule(buffer, cx - inner_r, cy, cx - inner_r + b_len, cy, 1.0, color);
        self.draw_aa_capsule(buffer, cx + inner_r, cy, cx + inner_r - b_len, cy, 1.0, color);

        let dot_col = if is_locked { 0xFFFF0055 } else { 0xFFFFFFFF };
        self.draw_aa_circle(buffer, cx, cy, if is_locked { 2.5 } else { 1.5 }, dot_col, color, 0.5);
    }

    // ==============================================================================
    // Renderizado de Arquetipos de Enemigos Avanzados
    // ==============================================================================

    /// 1. Sintéticos Humanoides Modernos (Porcelana Blanca / Obsidiana con Katana o Rifle)
    pub fn draw_modern_humanoid_synth(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        is_obsidian: bool,
        has_katana: bool,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let t = self.anim_time;
        let aim = enemy.aim_angle;

        let (plate_col, joint_col, glow_col, highlight_col) = if is_obsidian {
            (0xFF0F172A, 0xFF020617, 0xFF00F0FF, 0xFF38BDF8)
        } else {
            (0xFFF1F5F9, 0xFF1E293B, 0xFFFF0055, 0xFFFFFFFF)
        };

        let hover_y = (t * 5.0 + ex * 0.05).sin() * 3.5;
        let cy = ey + hover_y;

        let leg_swing = (t * 7.0).sin();
        let left_thigh_end = (ex + leg_swing * 6.0, cy + 14.0);
        let right_thigh_end = (ex - leg_swing * 6.0, cy + 14.0);
        let left_foot = (left_thigh_end.0 + 4.0, cy + 28.0);
        let right_foot = (right_thigh_end.0 - 4.0, cy + 28.0);

        self.draw_aa_capsule(buffer, ex, cy, left_thigh_end.0, left_thigh_end.1, 4.0, joint_col);
        self.draw_aa_capsule(buffer, left_thigh_end.0, left_thigh_end.1, left_foot.0, left_foot.1, 3.2, joint_col);
        self.draw_aa_capsule(buffer, ex, cy, right_thigh_end.0, right_thigh_end.1, 4.0, joint_col);
        self.draw_aa_capsule(buffer, right_thigh_end.0, right_thigh_end.1, right_foot.0, right_foot.1, 3.2, joint_col);

        self.draw_aa_capsule(buffer, ex - 1.0, cy + 2.0, left_thigh_end.0, left_thigh_end.1, 3.0, plate_col);
        self.draw_aa_capsule(buffer, ex + 1.0, cy + 2.0, right_thigh_end.0, right_thigh_end.1, 3.0, plate_col);
        self.draw_aa_capsule(buffer, left_foot.0 - 4.0, left_foot.1, left_foot.0 + 5.0, left_foot.1, 2.2, highlight_col);
        self.draw_aa_capsule(buffer, right_foot.0 - 4.0, right_foot.1, right_foot.0 + 5.0, right_foot.1, 2.2, highlight_col);

        self.draw_aa_capsule(buffer, ex, cy - 2.0, ex, cy + 10.0, 7.5, joint_col);
        self.draw_aa_capsule(buffer, ex, cy - 4.0, ex, cy + 6.0, 6.5, plate_col);
        self.draw_aa_capsule(buffer, ex, cy - 2.0, ex, cy + 5.0, 1.8, glow_col);
        self.draw_volumetric_light(buffer, ex, cy + 1.0, 14.0, glow_col, 0.45);

        let head_y = cy - 14.0;
        self.draw_aa_circle(buffer, ex, head_y, 7.0, plate_col, joint_col, 1.2);
        self.draw_aa_capsule(buffer, ex - 3.0, head_y - 4.5, ex + 2.0, head_y - 4.5, 1.5, highlight_col);

        let eye_offset_x = (aim.cos() * 4.5).clamp(-5.0, 5.0);
        let eye_offset_y = (aim.sin() * 2.5).clamp(-3.0, 3.0);
        let eye_x = ex + eye_offset_x;
        let eye_y = head_y + eye_offset_y;

        self.draw_aa_capsule(buffer, ex - 4.5, head_y, ex + 4.5, head_y, 2.0, 0xFF020617);
        self.draw_aa_circle(buffer, eye_x, eye_y, 2.0, 0xFFFFFFFF, glow_col, 0.8);
        self.draw_aa_capsule(buffer, eye_x - 7.0, eye_y, eye_x + 7.0, eye_y, 0.9, glow_col);
        self.draw_volumetric_light(buffer, eye_x, eye_y, 22.0, glow_col, 0.65);

        if has_katana {
            let shoulder_x = ex + 4.0;
            let shoulder_y = cy - 5.0;
            let hand_x = ex + aim.cos() * 12.0;
            let hand_y = cy + aim.sin() * 12.0;
            self.draw_aa_capsule(buffer, shoulder_x, shoulder_y, hand_x, hand_y, 3.0, plate_col);

            let hilt_len = 8.0;
            let blade_dir_x = aim.cos();
            let blade_dir_y = aim.sin();
            let hilt_tip_x = hand_x + blade_dir_x * hilt_len;
            let hilt_tip_y = hand_y + blade_dir_y * hilt_len;
            self.draw_aa_capsule(buffer, hand_x, hand_y, hilt_tip_x, hilt_tip_y, 2.2, 0xFF1E293B);

            let blade_len = 36.0;
            let blade_tip_x = hilt_tip_x + blade_dir_x * blade_len;
            let blade_tip_y = hilt_tip_y + blade_dir_y * blade_len;
            self.draw_aa_capsule(buffer, hilt_tip_x, hilt_tip_y, blade_tip_x, blade_tip_y, 5.0, glow_col);
            self.draw_aa_capsule(buffer, hilt_tip_x, hilt_tip_y, blade_tip_x, blade_tip_y, 2.0, 0xFFFFFFFF);
            self.draw_volumetric_light(buffer, (hilt_tip_x + blade_tip_x) * 0.5, (hilt_tip_y + blade_tip_y) * 0.5, 30.0, glow_col, 0.55);
        } else {
            let shoulder_x = ex + 3.0;
            let shoulder_y = cy - 4.0;
            let hand_x = ex + aim.cos() * 14.0;
            let hand_y = cy + aim.sin() * 14.0;
            self.draw_aa_capsule(buffer, shoulder_x, shoulder_y, hand_x, hand_y, 3.2, plate_col);

            let barrel_dir_x = aim.cos();
            let barrel_dir_y = aim.sin();
            let r_body_x = hand_x + barrel_dir_x * 10.0;
            let r_body_y = hand_y + barrel_dir_y * 10.0;
            self.draw_aa_capsule(buffer, hand_x, hand_y, r_body_x, r_body_y, 4.2, 0xFF0F172A);

            let muzzle_x = r_body_x + barrel_dir_x * 16.0;
            let muzzle_y = r_body_y + barrel_dir_y * 16.0;
            self.draw_aa_capsule(buffer, r_body_x, r_body_y, muzzle_x, muzzle_y, 2.4, 0xFF64748B);
            self.draw_aa_circle(buffer, r_body_x, r_body_y, 2.0, glow_col, 0xFFFFFFFF, 0.5);

            let sight_len = 80.0;
            let sight_x = muzzle_x + barrel_dir_x * sight_len;
            let sight_y = muzzle_y + barrel_dir_y * sight_len;
            self.draw_aa_capsule(buffer, muzzle_x, muzzle_y, sight_x, sight_y, 0.8, glow_col);
        }
    }

    /// 2. Ciber-Plantas Biomecánicas (Zarcillos Bioluminiscentes estilo Abadox)
    pub fn draw_cyber_vine(
        &self,
        buffer: &mut [u32],
        start_x: f32,
        start_y: f32,
        length: f32,
        base_angle: f32,
        anim_time: f32,
        vine_color: u32,
        lum_color: u32,
    ) {
        let segments = 8;
        let seg_len = length / segments as f32;
        let mut prev_x = start_x;
        let mut prev_y = start_y;

        for i in 1..=segments {
            let seg_f = i as f32;
            let wave = (anim_time * 3.5 + seg_f * 0.7).sin() * 12.0 * (seg_f / segments as f32);
            let curr_angle = base_angle + (wave * 0.03);
            let curr_x = prev_x + curr_angle.cos() * seg_len;
            let curr_y = prev_y + curr_angle.sin() * seg_len + wave * 0.4;
            let radius = (6.0 * (1.0 - (seg_f / (segments as f32 + 2.0)))).max(1.8);

            self.draw_aa_capsule(buffer, prev_x, prev_y, curr_x, curr_y, radius, vine_color);

            let pulse = ((anim_time * 6.0 - seg_f * 0.8).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            let sap_radius = (radius * 0.5).max(1.0);
            let pulsed_sap = Self::lerp_color(0xFF022C22, lum_color, pulse);
            self.draw_aa_capsule(buffer, prev_x, prev_y, curr_x, curr_y, sap_radius, pulsed_sap);

            if i % 2 == 0 {
                let thorn_angle = curr_angle + std::f32::consts::FRAC_PI_2;
                let thorn_len = radius * 2.2;
                let tx = curr_x + thorn_angle.cos() * thorn_len;
                let ty = curr_y + thorn_angle.sin() * thorn_len;
                self.draw_aa_capsule(buffer, curr_x, curr_y, tx, ty, 1.2, lum_color);
                self.draw_volumetric_light(buffer, curr_x, curr_y, radius * 3.5, lum_color, 0.35);
            }

            prev_x = curr_x;
            prev_y = curr_y;
        }
    }

    /// 3. Torretas de Esporas Pulsantes con Dilatación de Respiración
    pub fn draw_spore_pod_turret(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        anim_time: f32,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let is_ceiling = enemy.is_ceiling;
        let anchor_y = if is_ceiling { ey - 18.0 } else { ey + 18.0 };

        self.draw_aa_capsule(buffer, ex - 18.0, anchor_y, ex + 18.0, anchor_y, 4.0, 0xFF1E293B);
        self.draw_aa_capsule(buffer, ex - 12.0, anchor_y, ex - 6.0, ey, 3.5, 0xFF0F172A);
        self.draw_aa_capsule(buffer, ex + 12.0, anchor_y, ex + 6.0, ey, 3.5, 0xFF0F172A);

        let breathe = (anim_time * 4.5).sin();
        let bulb_r = 16.0 + breathe * 2.8;

        self.draw_aa_circle(buffer, ex, ey, bulb_r, 0xFF2E1065, 0xFF581C87, 2.5);

        let sac_r = bulb_r * 0.65;
        let sac_col = 0xFF10B981;
        let sac_pulse = (breathe * 0.3 + 0.7).clamp(0.0, 1.0);
        let inner_col = Self::lerp_color(0xFF064E3B, 0xFF6EE7B7, sac_pulse);
        self.draw_aa_circle(buffer, ex, ey, sac_r, inner_col, sac_col, 1.5);
        self.draw_volumetric_light(buffer, ex, ey, bulb_r + 20.0, sac_col, 0.65 * sac_pulse);

        let aim_x = ex + enemy.aim_angle.cos() * (bulb_r + 2.0);
        let aim_y = ey + enemy.aim_angle.sin() * (bulb_r + 2.0);
        self.draw_aa_capsule(buffer, ex, ey, aim_x, aim_y, 4.0, 0xFF020617);
        self.draw_aa_circle(buffer, aim_x, aim_y, 3.0, 0xFFFFFFFF, sac_col, 1.0);
    }

    /// 4. Flores Mecánicas Trampa con Pétalos Navaja y Estambre Láser
    pub fn draw_flower_trap(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        anim_time: f32,
        open_factor: f32,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let petal_count = 6;
        let base_radius = 20.0;

        let stem_base_y = if enemy.is_ceiling { ey - 22.0 } else { ey + 22.0 };
        self.draw_aa_capsule(buffer, ex, stem_base_y, ex, ey, 5.0, 0xFF14532D);
        self.draw_aa_capsule(buffer, ex, stem_base_y, ex, ey, 2.0, 0xFF22C55E);

        let stamen_glow = 0xFF00F0FF;
        let charge_pulse = (anim_time * 12.0).sin().abs();
        let stamen_r = 6.0 + open_factor * 3.0;

        if open_factor > 0.15 {
            self.draw_aa_circle(buffer, ex, ey, stamen_r, 0xFFFFFFFF, stamen_glow, 2.0);
            self.draw_volumetric_light(buffer, ex, ey, 32.0 * open_factor, stamen_glow, 0.75 * charge_pulse);

            for i in 0..4 {
                let f_angle = (i as f32) * (std::f32::consts::PI * 0.5) + anim_time * 2.0;
                let fx = ex + f_angle.cos() * (stamen_r + 5.0);
                let fy = ey + f_angle.sin() * (stamen_r + 5.0);
                self.draw_aa_capsule(buffer, ex, ey, fx, fy, 1.2, 0xFFFFFFFF);
                self.draw_aa_circle(buffer, fx, fy, 1.8, stamen_glow, 0xFFFFFFFF, 0.5);
            }
        }

        for p in 0..petal_count {
            let angle = (p as f32) * (std::f32::consts::PI * 2.0 / petal_count as f32);
            let blossom_spread = open_factor * 16.0;
            let petal_dist = base_radius + blossom_spread;
            let px = ex + angle.cos() * petal_dist;
            let py = ey + angle.sin() * petal_dist;

            let side_angle1 = angle + 0.35 * (1.0 - open_factor * 0.3);
            let side_angle2 = angle - 0.35 * (1.0 - open_factor * 0.3);
            let sx1 = ex + side_angle1.cos() * (base_radius * 0.7);
            let sy1 = ey + side_angle1.sin() * (base_radius * 0.7);
            let sx2 = ex + side_angle2.cos() * (base_radius * 0.7);
            let sy2 = ey + side_angle2.sin() * (base_radius * 0.7);

            self.draw_aa_capsule(buffer, sx1, sy1, px, py, 2.2, 0xFF94A3B8);
            self.draw_aa_capsule(buffer, sx2, sy2, px, py, 2.2, 0xFF475569);
            self.draw_aa_capsule(buffer, ex, ey, px, py, 2.8, 0xFF1E293B);
            self.draw_aa_circle(buffer, px, py, 2.5, 0xFFFFFFFF, 0xFF38BDF8, 0.8);
        }
    }

    /// 5. Dron Cazador Predatorio de Alas en Flecha Invertida
    pub fn draw_predatory_drone(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        anim_time: f32,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let tilt_y = (enemy.vy * 0.05).clamp(-8.0, 8.0);

        let engine_y1 = ey - 8.0 + tilt_y;
        let engine_y2 = ey + 8.0 + tilt_y;
        let eng_x = ex + 14.0;
        self.draw_aa_capsule(buffer, eng_x, engine_y1, eng_x - 12.0, engine_y1, 4.0, 0xFF1E293B);
        self.draw_aa_capsule(buffer, eng_x, engine_y2, eng_x - 12.0, engine_y2, 4.0, 0xFF1E293B);

        let exhaust_len = 16.0 + (anim_time * 30.0).sin().abs() * 8.0;
        self.draw_plasma_plume(buffer, eng_x, engine_y1, 1.0, 0.0, exhaust_len, 5.0, 0xFFFFFFFF, 0xFFFF4500);
        self.draw_plasma_plume(buffer, eng_x, engine_y2, 1.0, 0.0, exhaust_len, 5.0, 0xFFFFFFFF, 0xFFFF4500);

        self.draw_aa_capsule(buffer, ex + 4.0, ey - 4.0, ex - 16.0, ey - 22.0, 4.5, 0xFF0F172A);
        self.draw_aa_capsule(buffer, ex - 16.0, ey - 22.0, ex - 24.0, ey - 18.0, 3.2, 0xFF38BDF8);
        self.draw_aa_capsule(buffer, ex + 4.0, ey + 4.0, ex - 16.0, ey + 22.0, 4.5, 0xFF0F172A);
        self.draw_aa_capsule(buffer, ex - 16.0, ey + 22.0, ex - 24.0, ey + 18.0, 3.2, 0xFF38BDF8);

        self.draw_aa_capsule(buffer, ex + 10.0, ey, ex - 18.0, ey, 7.5, 0xFF1E293B);
        self.draw_aa_capsule(buffer, ex + 6.0, ey - 2.0, ex - 14.0, ey - 2.0, 5.5, 0xFF475569);

        let sensor_x = ex - 16.0;
        self.draw_aa_circle(buffer, sensor_x, ey, 4.5, 0xFFFF0033, 0xFFFFFFFF, 1.2);
        self.draw_volumetric_light(buffer, sensor_x, ey, 24.0, 0xFFFF0033, 0.7);
    }

    /// 6. Leviatán Mecánico Biomecánico de Asalto (Mecha Leviathan)
    pub fn draw_biomech_leviathan(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        anim_time: f32,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let r = enemy.radius;

        self.draw_aa_circle(buffer, ex, ey, r, 0xFF0F172A, 0xFF334155, 3.0);
        self.draw_aa_circle(buffer, ex, ey, r - 5.0, 0xFF1E293B, 0xFF475569, 2.0);
        self.draw_aa_capsule(buffer, ex - r * 0.6, ey - r * 0.5, ex + r * 0.6, ey - r * 0.5, 2.0, 0xFF94A3B8);

        let pincer_cycle = (anim_time * 4.0).sin();
        let claw_open = (pincer_cycle * 8.0).max(0.0);

        let u_joint_x = ex - r * 0.8;
        let u_joint_y = ey - r * 0.7;
        let u_tip_x = u_joint_x - 18.0;
        let u_tip_y = u_joint_y - 10.0 - claw_open;
        self.draw_aa_capsule(buffer, ex - r * 0.3, ey - r * 0.4, u_joint_x, u_joint_y, 5.0, 0xFF334155);
        self.draw_aa_capsule(buffer, u_joint_x, u_joint_y, u_tip_x, u_tip_y, 4.0, 0xFF64748B);
        self.draw_aa_capsule(buffer, u_tip_x, u_tip_y, u_tip_x + 6.0, u_tip_y + 14.0, 2.5, 0xFF00F0FF);

        let l_joint_x = ex - r * 0.8;
        let l_joint_y = ey + r * 0.7;
        let l_tip_x = l_joint_x - 18.0;
        let l_tip_y = l_joint_y + 10.0 + claw_open;
        self.draw_aa_capsule(buffer, ex - r * 0.3, ey + r * 0.4, l_joint_x, l_joint_y, 5.0, 0xFF334155);
        self.draw_aa_capsule(buffer, l_joint_x, l_joint_y, l_tip_x, l_tip_y, 4.0, 0xFF64748B);
        self.draw_aa_capsule(buffer, l_tip_x, l_tip_y, l_tip_x + 6.0, l_tip_y - 14.0, 2.5, 0xFF00F0FF);

        let core_pulse = (anim_time * 6.0).sin().abs();
        let core_r = 10.0 + core_pulse * 2.5;
        let core_col = 0xFF00F0FF;
        self.draw_aa_circle(buffer, ex, ey, core_r, 0xFFFFFFFF, core_col, 2.5);
        self.draw_volumetric_light(buffer, ex, ey, 45.0, core_col, 0.85);
    }

    /// Renderiza naves, sintéticos y bio-plantas enemigas con despacho al arquetipo adecuado
    pub fn draw_enemy(&self, buffer: &mut [u32], e: &Enemy) {
        match e.enemy_type {
            EnemyType::SynthKatana => {
                self.draw_modern_humanoid_synth(buffer, e, false, true);
            }
            EnemyType::SynthRifle => {
                self.draw_modern_humanoid_synth(buffer, e, true, false);
            }
            EnemyType::BioPlantVine => {
                self.draw_cyber_vine(buffer, e.x, e.y, e.radius * 2.5, e.aim_angle, self.anim_time, 0xFF14532D, 0xFF22C55E);
            }
            EnemyType::BioPlantSporePod => {
                self.draw_spore_pod_turret(buffer, e, self.anim_time);
            }
            EnemyType::BioPlantFlowerTrap => {
                let open_factor = ((self.anim_time * 2.5).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                self.draw_flower_trap(buffer, e, self.anim_time, open_factor);
            }
            EnemyType::PredatoryDrone => {
                self.draw_predatory_drone(buffer, e, self.anim_time);
            }
            EnemyType::BioMechLeviathan => {
                self.draw_biomech_leviathan(buffer, e, self.anim_time);
            }
            // Mapeos retrocompatibles
            EnemyType::PatrolDrone => {
                self.draw_predatory_drone(buffer, e, self.anim_time);
            }
            EnemyType::KamikazeWasp => {
                self.draw_predatory_drone(buffer, e, self.anim_time);
            }
            EnemyType::LaserTurret => {
                self.draw_spore_pod_turret(buffer, e, self.anim_time);
            }
            EnemyType::CyberCrab => {
                self.draw_biomech_leviathan(buffer, e, self.anim_time);
            }
            EnemyType::AsteroidLeech => {
                self.draw_cyber_vine(buffer, e.x, e.y, e.radius * 2.5, e.aim_angle, self.anim_time, 0xFF2E1065, 0xFF9333EA);
            }
            EnemyType::StealthStriker => {
                self.draw_modern_humanoid_synth(buffer, e, true, true);
            }
        }
    }

    // ==============================================================================
    // Renderizado del Comando Humano Arnold & Sigourney y Satélites Bivalvos
    // ==============================================================================

    pub fn draw_modern_commando(&self, buffer: &mut [u32], player: &Player) {
        let px = player.x;
        let py = player.y;
        let dir = if player.facing_right { 1.0 } else { -1.0 };
        let t = player.anim_timer;

        if player.invulnerable_timer > 0.0 && ((player.invulnerable_timer * 18.0) as usize % 2 == 0) {
            return;
        }

        let tilt_y = (player.bank_angle * 0.8).clamp(-12.0, 12.0);
        let hover_y = (t * 6.0).sin() * 2.2;
        let cy = py + hover_y;

        let (armor_base, armor_highlight, visor_glow, plume_outer, plume_core) = match player.id {
            0 => (0xFF0F2B48, 0xFFE0E7FF, 0xFF00F0FF, 0xFF0099FF, 0xFFE0FFFF), // Arnold (P1): Cobalto / Platino / Cian
            1 => (0xFF590D1C, 0xFFFFD700, 0xFFFFB703, 0xFFFF3300, 0xFFFFFBEB), // Sigourney (P2): Carmesí / Oro / Ámbar
            2 => (0xFF0B3D20, 0xFF86EFAC, 0xFF00FF77, 0xFF059669, 0xFFD1FAE5), // Jax (P3): Esmeralda / Plasma Verde
            _ => (0xFF451A03, 0xFFFDE68A, 0xFFFFD700, 0xFFD97706, 0xFFFEF3C7), // Orion (P4): Ámbar / Oro Solar
        };

        let recoil_x = player.recoil_anim * 5.0;

        // 1. Propulsor Jetpack y Pluma de Plasma de Doble Capa
        let jet_x = px - dir * 16.0;
        let jet_y = cy - 2.0 - (tilt_y * 0.5);
        let plume_len = if player.jetpack_active { 28.0 } else { 14.0 };
        self.draw_plasma_plume(buffer, jet_x, jet_y, -dir, 0.0, plume_len, 6.5, plume_core, plume_outer);
        self.draw_aa_capsule(buffer, jet_x + dir * 6.0, jet_y - 7.0 + tilt_y, jet_x, jet_y + 9.0 + tilt_y, 4.5, 0xFF1E293B);
        self.draw_aa_capsule(buffer, jet_x + dir * 4.0, jet_y - 6.0 + tilt_y, jet_x, jet_y - 6.0 + tilt_y, 2.5, armor_highlight);

        // 2. Piernas articuladas y botas estabilizadoras
        let leg_sway = if player.jetpack_active { (t * 15.0).sin() * 3.5 } else { 0.0 };
        self.draw_aa_capsule(buffer, px - dir * 6.0, cy + 8.0 + tilt_y, px - dir * 8.0 + leg_sway, cy + 22.0 + tilt_y, 3.8, 0xFF0F172A);
        self.draw_aa_capsule(buffer, px + dir * 2.0, cy + 8.0 + tilt_y, px + dir * 4.0 - leg_sway, cy + 24.0 + tilt_y, 4.2, armor_base);
        self.draw_aa_capsule(buffer, px + dir * 2.0 - leg_sway, cy + 24.0 + tilt_y, px + dir * 8.0 - leg_sway, cy + 24.0 + tilt_y, 2.0, armor_highlight);

        // 3. Coraza torácica y reactor arc central
        self.draw_aa_capsule(buffer, px - dir * 5.0, cy - 6.0 + tilt_y, px + dir * 5.0, cy + 6.0 + tilt_y, 7.5, 0xFF020617);
        self.draw_aa_capsule(buffer, px - dir * 4.0, cy - 7.0 + tilt_y, px + dir * 4.0, cy + 5.0 + tilt_y, 6.2, armor_base);
        self.draw_aa_capsule(buffer, px - dir * 3.0, cy - 8.0 + tilt_y, px + dir * 4.0, cy - 4.0 + tilt_y, 1.8, armor_highlight);
        self.draw_aa_circle(buffer, px + dir * 2.0, cy - 1.0 + tilt_y, 2.8, 0xFFFFFFFF, visor_glow, 0.8);
        self.draw_volumetric_light(buffer, px + dir * 2.0, cy - 1.0 + tilt_y, 16.0, visor_glow, 0.55);

        // 4. Casco aerodinámico y visor HUD panorámico
        let head_x = px + dir * 2.0;
        let head_y = cy - 15.0 + tilt_y;
        self.draw_aa_circle(buffer, head_x, head_y, 6.8, armor_base, 0xFF020617, 1.2);
        self.draw_aa_capsule(buffer, head_x - dir * 2.0, head_y - 5.0, head_x + dir * 2.0, head_y - 5.0, 1.4, armor_highlight);

        let visor_x = head_x + dir * 3.0;
        let visor_y = head_y;
        self.draw_aa_capsule(buffer, visor_x, visor_y - 1.5, visor_x + dir * 3.5, visor_y, 2.5, visor_glow);
        self.draw_aa_capsule(buffer, visor_x + dir * 1.0, visor_y - 1.5, visor_x + dir * 3.5, visor_y - 0.5, 1.2, 0xFFFFFFFF);
        self.draw_volumetric_light(buffer, visor_x + dir * 8.0, visor_y, 24.0, visor_glow, 0.6);

        // 5. Fusil de plasma pesado con amortiguación de retroceso
        let gun_x = px + dir * (12.0 - recoil_x);
        let gun_y = cy - 2.0 + tilt_y;
        self.draw_aa_capsule(buffer, gun_x, gun_y, gun_x + dir * 16.0, gun_y, 4.0, 0xFF1E293B);
        self.draw_aa_capsule(buffer, gun_x + dir * 12.0, gun_y - 2.0, gun_x + dir * 22.0, gun_y - 2.0, 1.6, 0xFF94A3B8);
        self.draw_aa_capsule(buffer, gun_x + dir * 12.0, gun_y + 2.0, gun_x + dir * 22.0, gun_y + 2.0, 1.6, 0xFF94A3B8);

        if player.fire_timer > 0.04 || player.recoil_anim > 0.5 {
            let flash_x = gun_x + dir * 25.0;
            self.draw_aa_circle(buffer, flash_x, gun_y, 5.0, 0xFFFFFFFF, visor_glow, 1.5);
            self.draw_volumetric_light(buffer, flash_x, gun_y, 28.0, visor_glow, 0.85);
        }

        // 6. Retícula HUD holográfica en pantalla
        let reticle_dist = 65.0;
        let reticle_x = px + dir * reticle_dist;
        let reticle_y = cy + tilt_y;
        self.draw_holographic_reticle(buffer, reticle_x, reticle_y, 14.0, visor_glow, false);

        // 7. Satélites Orbitales Bivalvos con Escudos Rotacionales
        for sat in player.satellites.iter() {
            self.draw_modern_satellite(buffer, sat, px, py, visor_glow, t);
        }
    }

    /// Satélite orbital bivalvo con proyección de barrera de plasma
    pub fn draw_modern_satellite(
        &self,
        buffer: &mut [u32],
        sat: &crate::player::Satellite,
        player_x: f32,
        player_y: f32,
        accent_color: u32,
        anim_time: f32,
    ) {
        let sx = player_x + sat.angle.cos() * sat.distance;
        let sy = player_y + sat.angle.sin() * sat.distance;
        let shell_rot = anim_time * 4.0;
        let is_locked = sat.is_locked;

        let shield_radius = 16.0;
        let shield_col = if is_locked { 0xFFFF0055 } else { accent_color };
        let shield_facing = sat.aim_angle;

        for a in -3..=3 {
            let arc_a = shield_facing + (a as f32) * 0.18;
            let arc_x = sx + arc_a.cos() * shield_radius;
            let arc_y = sy + arc_a.sin() * shield_radius;
            self.draw_aa_circle(buffer, arc_x, arc_y, 1.8, 0xFFFFFFFF, shield_col, 0.8);
        }
        self.draw_volumetric_light(buffer, sx + shield_facing.cos() * shield_radius, sy + shield_facing.sin() * shield_radius, 22.0, shield_col, 0.45);

        let h1_x = sx + shell_rot.cos() * 5.0;
        let h1_y = sy + shell_rot.sin() * 5.0;
        let h2_x = sx - shell_rot.cos() * 5.0;
        let h2_y = sy - shell_rot.sin() * 5.0;

        self.draw_aa_capsule(buffer, h1_x - 3.0, h1_y - 2.0, h1_x + 3.0, h1_y + 2.0, 3.5, 0xFF1E293B);
        self.draw_aa_capsule(buffer, h2_x - 3.0, h2_y - 2.0, h2_x + 3.0, h2_y + 2.0, 3.5, 0xFF1E293B);
        self.draw_aa_circle(buffer, h1_x, h1_y, 2.0, 0xFFFFFFFF, shield_col, 0.5);
        self.draw_aa_circle(buffer, h2_x, h2_y, 2.0, 0xFFFFFFFF, shield_col, 0.5);

        self.draw_aa_circle(buffer, sx, sy, 3.8, 0xFFFFFFFF, shield_col, 1.0);
        self.draw_volumetric_light(buffer, sx, sy, 18.0, shield_col, 0.6);

        if is_locked {
            let trace_len = 50.0;
            let tx = sx + sat.aim_angle.cos() * trace_len;
            let ty = sy + sat.aim_angle.sin() * trace_len;
            self.draw_aa_capsule(buffer, sx, sy, tx, ty, 0.9, 0xFFFF0055);
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
                self.draw_rect(buffer, bx - 60, by - 55, 120, 110, 0xFF2A2E3B);
                self.draw_rect(buffer, bx - 70, by + 40, 140, 22, 0xFF181C24);
                self.draw_rect(buffer, bx - 65, by + 45, 130, 4, 0xFF737373);

                self.draw_rect(buffer, bx - 55, by - 48, 100, 20, 0xFFC74C0C);
                self.draw_rect(buffer, bx - 55, by - 48, 100, 4, 0xFF111111);

                self.draw_rect(buffer, bx - 30, by - 70, 40, 18, 0xFF4A5568);
                self.draw_circle(buffer, bx - 35, by - 62, 7, 0xFFFF2020);

                let core_color = if b.phase >= 2 { 0xFFFF0055 } else { 0xFFFF5500 };
                let core_r = 16 + ((t * 6.0).sin().abs() * 4.0) as usize;
                self.draw_circle(buffer, bx, by, core_r, core_color);
                self.draw_circle(buffer, bx, by, 8, 0xFFFFFFFF);
                self.draw_point_light(buffer, bx, by, 65, core_color, 0.8);

                if b.health < b.max_health * 0.5 && ((t * 18.0) as usize % 2 == 0) {
                    self.draw_circle(buffer, bx - 40, by - 20, 3, 0xFFFFD700);
                    self.draw_circle(buffer, bx + 20, by + 10, 4, 0xFFFF4500);
                }
            }
            _ => {
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

        let max_w = 520;
        let cur_w = ((b.health / b.max_health).clamp(0.0, 1.0) * max_w as f32) as usize;
        let bar_x = (w as isize - max_w as isize) / 2;
        self.draw_rect(buffer, bar_x - 6, 22, max_w + 12, 22, 0xFF550000);
        self.draw_rect(buffer, bar_x - 2, 24, max_w + 4, 18, 0xFF1A0000);
        self.draw_rect(buffer, bar_x, 26, cur_w, 14, 0xFFFF2244);
        self.draw_rect(buffer, bar_x, 26, cur_w, 3, 0xFFFF99AA);
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
            let max_hp_w = 180;
            let hp_w = ((p1.health / p1.max_health).clamp(0.0, 1.0) * max_hp_w as f32) as usize;
            self.draw_rect(buffer, 24, 20, max_hp_w + 8, 16, 0xFF1B1B26);
            self.draw_rect(buffer, 28, 24, max_hp_w, 8, 0xFF3D3D4E);
            let hp_col = if p1.health > 40.0 { 0xFF00FF77 } else { 0xFFFF3344 };
            self.draw_rect(buffer, 28, 24, hp_w, 8, hp_col);

            for b in 0..p1.bombs.min(6) {
                let bx = 28 + (b as isize * 18);
                self.draw_circle(buffer, bx, 44, 5, 0xFF00E5FF);
                self.draw_circle(buffer, bx, 44, 2, 0xFFFFFFFF);
            }

            for v in 0..p1.lives.max(0).min(6) {
                let vx = 140 + (v as isize * 14);
                self.draw_circle(buffer, vx, 44, 4, 0xFF00D2FF);
            }

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
        let x_start = x.max(0);
        let y_start = y.max(0);
        let x_end = (x + rw as isize).min(w);
        let y_end = (y + rh as isize).min(h);

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        let is_translucent = (color >> 24) != 0xFF && (color >> 24) != 0;
        let alpha = ((color >> 24) & 0xFF) as f32 / 255.0;

        for py in y_start..y_end {
            let row = py as usize * self.width;
            for px in x_start..x_end {
                let idx = row + px as usize;
                if is_translucent {
                    buffer[idx] = Self::blend_alpha(buffer[idx], color, alpha);
                } else {
                    buffer[idx] = color;
                }
            }
        }
    }

    pub fn draw_circle(&self, buffer: &mut [u32], cx: isize, cy: isize, radius: usize, color: u32) {
        let w = self.width as isize;
        let h = self.height as isize;
        let r = radius as isize;
        let r2 = r * r;

        let y_min = (cy - r).max(0);
        let y_max = (cy + r).min(h - 1);
        let x_min = (cx - r).max(0);
        let x_max = (cx + r).min(w - 1);

        for py in y_min..=y_max {
            let dy = py - cy;
            let dy2 = dy * dy;
            let row = py as usize * self.width;
            for px in x_min..=x_max {
                let dx = px - cx;
                if dx * dx + dy2 <= r2 {
                    buffer[row + px as usize] = color;
                }
            }
        }
    }

    pub fn draw_point_light(&self, buffer: &mut [u32], lx: isize, ly: isize, radius: usize, color: u32, intensity: f32) {
        let w = self.width as isize;
        let h = self.height as isize;
        let r = radius as isize;
        let r2 = (r * r) as f32;
        let inv_r2 = 1.0 / r2;

        let y_min = (ly - r).max(0);
        let y_max = (ly + r).min(h - 1);
        let x_min = (lx - r).max(0);
        let x_max = (lx + r).min(w - 1);

        for py in y_min..=y_max {
            let dy = (py - ly) as f32;
            let dy2 = dy * dy;
            let row = py as usize * self.width;
            for px in x_min..=x_max {
                let dx = (px - lx) as f32;
                let d2 = dx * dx + dy2;
                if d2 <= r2 {
                    let factor = (1.0 - d2 * inv_r2).powi(2) * intensity;
                    let idx = row + px as usize;
                    buffer[idx] = Self::blend_additive(buffer[idx], color, factor);
                }
            }
        }
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
