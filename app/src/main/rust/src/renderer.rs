//! Rasterizador RGBA y Renderizado Gráfico Retro para IA Rebellion
//! Dibuja el campo de estrellas en paralaje, naves, jefes, efectos de partículas, HUD y controles táctiles

use crate::bullet::Bullet;
use crate::boss::Boss;
use crate::enemy::Enemy;
use crate::player::Player;
use crate::touch::TouchControls;

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
}

impl Renderer {
    pub fn new(w: usize, h: usize) -> Self {
        let mut stars = Vec::with_capacity(120);
        for i in 0..120 {
            let speed = match i % 3 {
                0 => 40.0,
                1 => 90.0,
                _ => 160.0,
            };
            stars.push(Star {
                x: ((i * 37) % w) as f32,
                y: ((i * 47) % h) as f32,
                speed,
                size: if speed > 100.0 { 2 } else { 1 },
                brightness: ((i * 19) % 155 + 100) as u8,
            });
        }

        Self {
            stars,
            particles: Vec::with_capacity(256),
            width: w,
            height: h,
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.width = w;
        self.height = h;
    }

    pub fn add_explosion(&mut self, x: f32, y: f32, count: usize, base_color: u32) {
        for i in 0..count {
            let angle = (i as f32) * (std::f32::consts::PI * 2.0 / count as f32);
            let speed = 90.0 + ((i * 13) % 180) as f32;
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.6,
                max_life: 0.6,
                color: base_color,
                size: 3.5,
            });
        }
    }

    pub fn update_fx(&mut self, dt: f32) {
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

    pub fn render_frame(
        &mut self,
        buffer: &mut [u32],
        bg_color: u32,
        players: &[Player],
        enemies: &[Enemy],
        boss: &Option<Boss>,
        bullets: &[Bullet],
        touch: &TouchControls,
        _stage_num: u8,
        _stage_name: &str,
    ) {
        let w = self.width;
        let h = self.height;

        // 1. Limpieza de pantalla y fondo
        buffer.fill(bg_color);

        // 2. Fondo de estrellas en paralaje
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

        // 3. Renderizado de partículas
        for p in self.particles.iter() {
            let px = p.x as isize;
            let py = p.y as isize;
            let rad = p.size as isize;
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            if alpha > 0.1 {
                self.draw_rect(buffer, px - rad, py - rad, (rad * 2) as usize, (rad * 2) as usize, p.color);
            }
        }

        // 4. Renderizado de proyectiles
        for b in bullets.iter() {
            if b.active {
                let bx = b.x as isize;
                let by = b.y as isize;
                let r = b.radius as usize;
                self.draw_circle(buffer, bx, by, r, b.color);
            }
        }

        // 5. Renderizado de enemigos comunes
        for e in enemies.iter() {
            if e.active {
                let ex = e.x as isize;
                let ey = e.y as isize;
                let r = e.radius as usize;
                // Color agresivo rojo / naranja
                self.draw_rect(buffer, ex - (r as isize), ey - (r as isize), r * 2, r * 2, 0xFFFF3333);
                // Núcleo cibernético
                self.draw_rect(buffer, ex - 4, ey - 4, 8, 8, 0xFF00FFFF);
            }
        }

        // 6. Renderizado de Boss (si está presente)
        if let Some(b) = boss {
            if b.active && !b.defeated {
                let bx = b.x as isize;
                let by = b.y as isize;
                let r = b.radius as usize;
                // Cuerpo colosal del jefe
                self.draw_rect(buffer, bx - (r as isize), by - (r as isize), r * 2, r * 2, 0xFFFF0055);
                // Núcleo energético parpadeante
                let core_color = if b.phase == 3 { 0xFFFF0000 } else { 0xFFFFFF00 };
                self.draw_circle(buffer, bx, by, r / 3, core_color);

                // Barra de vida del jefe en la parte superior
                let bar_w = (w as f32 * 0.6) as usize;
                let bar_x = (w - bar_w) / 2;
                let hp_ratio = (b.health / b.max_health).clamp(0.0, 1.0);
                let current_bar_w = (bar_w as f32 * hp_ratio) as usize;
                self.draw_rect(buffer, bar_x as isize, 40, bar_w, 14, 0xFF333333);
                self.draw_rect(buffer, bar_x as isize, 40, current_bar_w, 14, 0xFFFF2222);
            }
        }

        // 7. Renderizado de Jugadores y sus Satélites Orbitales (Mecánica Final Mission)
        for player in players.iter() {
            if !player.active {
                continue;
            }

            let px = player.x as isize;
            let py = player.y as isize;

            // Nave del jugador (fuselaje triangular)
            self.draw_rect(buffer, px - 18, py - 12, 36, 24, player.color);
            self.draw_rect(buffer, px + 8, py - 6, 16, 12, 0xFFFFFFFF); // Cabina de mando
            // Llama del propulsor
            self.draw_rect(buffer, px - 26, py - 5, 8, 10, 0xFFFF8C00);

            // Satélites orbitales que giran alrededor del jugador
            for sat in player.satellites.iter() {
                let sat_x = (player.x + sat.angle.cos() * sat.distance) as isize;
                let sat_y = (player.y + sat.angle.sin() * sat.distance) as isize;
                let sat_color = if sat.is_locked { 0xFFFF0000 } else { 0xFF00FFFF };
                self.draw_circle(buffer, sat_x, sat_y, 7, sat_color);
                self.draw_circle(buffer, sat_x, sat_y, 3, 0xFFFFFFFF);
            }
        }

        // 8. Interfaz HUD (Vida, Puntos, Nivel)
        if let Some(p1) = players.first() {
            let hp_w = (p1.health.max(0.0) * 2.0) as usize;
            self.draw_rect(buffer, 30, 30, 200, 16, 0xFF222222);
            self.draw_rect(buffer, 30, 30, hp_w, 16, 0xFF00FF00); // Barra de vida P1
        }

        // 9. Controles táctiles en pantalla
        self.render_touch_overlay(buffer, touch);
    }

    fn render_touch_overlay(&self, buffer: &mut [u32], touch: &TouchControls) {
        let w = self.width;
        let h = self.height;

        // Joystick virtual (izq)
        if touch.joystick_active {
            self.draw_circle(buffer, touch.joystick_base_x as isize, touch.joystick_base_y as isize, 60, 0x55AAAAAA);
            self.draw_circle(buffer, touch.joystick_current_x as isize, touch.joystick_current_y as isize, 32, 0x8800D2FF);
        } else {
            self.draw_circle(buffer, 220, h as isize - 220, 60, 0x33AAAAAA);
        }

        // Botón Disparo (Gran botón circular abajo a la derecha)
        let fire_col = if touch.btn_fire { 0xAAFF3333 } else { 0x55FF0000 };
        self.draw_circle(buffer, w as isize - 160, h as isize - 160, 52, fire_col);

        // Botón Bomba Especial
        let bomb_col = if touch.btn_special_bomb { 0xAAFFAA00 } else { 0x55FF8800 };
        self.draw_circle(buffer, w as isize - 290, h as isize - 230, 38, bomb_col);

        // Botón Satélites: Fijar / Rotar
        let sat_col = if touch.btn_satellite_lock { 0xAA00FFFF } else { 0x550088FF };
        self.draw_circle(buffer, w as isize - 130, h as isize - 310, 38, sat_col);
    }

    fn draw_rect(&self, buffer: &mut [u32], x: isize, y: isize, rw: usize, rh: usize, color: u32) {
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

    fn draw_circle(&self, buffer: &mut [u32], cx: isize, cy: isize, radius: usize, color: u32) {
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
}
