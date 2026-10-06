//! Rasterizador Gráfico de Alto Rendimiento para IA R3bellion
//! Experiencia visual estilo Final Mission (Japón) y Abadox de NES/Famicom.
//! Dibuja soldados humanos con armadura cibernética y jetpacks con llamas animadas,
//! 8 escenarios atmosféricos ricos con paralaje (Tierra, Fábrica de Drones, Ciudad Desolada,
//! Colmena Bio-Orgánica de Abadox, Laboratorio Secreto, Cinturón de Asteroides, Nave Némesis,
//! y Vórtice Cuántico), proyectiles, efectos de partículas y HUD limpio SIN BOTONES VIRTUALES.

use crate::bullet::{Bullet, BulletType};
use crate::boss::Boss;
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

        // 1. Renderizado de escenario enriquecido según el nivel
        self.render_stage_environment(buffer, stage_num);

        // 2. Campo de estrellas en paralaje
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

        // 4. Enemigos comunes detallados y texturizados
        for e in enemies.iter() {
            if !e.active {
                continue;
            }
            self.draw_enemy(buffer, e);
        }

        // 5. Jefe de fase (Boss) con múltiples paneles y núcleo cuántico
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

        // 7. Soldados Humanos con Traje Cibernético y Jetpack (Mecánica Final Mission Japón)
        for player in players.iter() {
            if !player.active {
                continue;
            }
            self.draw_human_cyber_soldier(buffer, player);
        }

        // 8. HUD Limpio Superior (Salud, Bombas, Vidas, Escenario) - CERO BOTONES VIRTUALES
        self.render_clean_hud(buffer, players, stage_num, stage_name);
    }

    /// Renderiza escenarios temáticos ricos basados en el código Java original y el lore
    fn render_stage_environment(&self, buffer: &mut [u32], stage: u8) {
        let w = self.width;
        let h = self.height;
        let t = self.anim_time;
        let s = self.scroll_x;

        match stage {
            // Nivel 1: Órbita Terrestre (Earth Orbit) - Curvatura de la Tierra y Satélites
            1 => {
                buffer.fill(0xFF060919); // Espacio azul noche profundo
                // Curvatura planetaria de la Tierra de fondo
                let planet_cx = (w / 2) as isize;
                let planet_cy = (h + 380) as isize;
                let radius = 540;
                self.draw_circle(buffer, planet_cx, planet_cy, radius, 0xFF0D3B66);
                self.draw_circle(buffer, planet_cx, planet_cy, radius - 15, 0xFF145DA0);
                // Atmósfera resplandeciente
                self.draw_circle(buffer, planet_cx, planet_cy, radius - 30, 0xFF2E8BC0);
            }
            // Nivel 2: Fábrica Automatizada de Drones (Drone Forge - EscenarioFabricaDeDrones)
            2 => {
                buffer.fill(0xFF140D07); // Fondo fundición cobriza
                // Vigas industriales superiores e inferiores
                let beam_offset = (s * 0.4) as usize % 120;
                for bx in 0..(w / 120 + 2) {
                    let rx = (bx * 120) as isize - beam_offset as isize;
                    // Vigas verticales de acero
                    self.draw_rect(buffer, rx, 0, 14, 90, 0xFF2E2218);
                    self.draw_rect(buffer, rx, h as isize - 90, 14, 90, 0xFF2E2218);
                    // Luces estroboscópicas de fábrica
                    let flash = (t * 4.0).sin() > 0.0;
                    let light_col = if flash { 0xFFFF4500 } else { 0xFF661100 };
                    self.draw_circle(buffer, rx + 7, 85, 4, light_col);
                    self.draw_circle(buffer, rx + 7, h as isize - 85, 4, light_col);
                }
                // Metal fundido brillante en la base
                self.draw_rect(buffer, 0, h as isize - 24, w, 24, 0xFFFF5500);
                self.draw_rect(buffer, 0, h as isize - 12, w, 12, 0xFFFFCC00);
            }
            // Nivel 3: Ciudad Desolada Cyberpunk (Cyber Megacity - EscenarioCiudadDesolada)
            3 => {
                buffer.fill(0xFF0D0B18); // Cielo nocturno cyberpunk
                // Siluetas de rascacielos con ventanas de neón
                let city_scroll = (s * 0.35) as usize % 90;
                for bld in 0..(w / 90 + 2) {
                    let bx = (bld * 90) as isize - city_scroll as isize;
                    let bld_h = 160 + ((bld * 43) % 180);
                    let by = h as isize - bld_h as isize;
                    self.draw_rect(buffer, bx, by, 75, bld_h, 0xFF17142B);
                    // Ventanas iluminadas en cian y magenta
                    for wy in 0..(bld_h / 24) {
                        for wx in 0..3 {
                            let win_col = if (bld + wx + wy) % 3 == 0 { 0xFF00F0FF } else { 0xFFFF007F };
                            self.draw_rect(buffer, bx + 12 + (wx * 20) as isize, by + 18 + (wy * 24) as isize, 8, 10, win_col);
                        }
                    }
                }
            }
            // Nivel 4: Colmena Bio-Orgánica (Bio-Organic Hive - Tributo Abadox Japón)
            4 => {
                buffer.fill(0xFF1F0812); // Rojo visceral orgánico
                // Paredes de tejido biológico con costillas ondulantes
                let wave_step = (s * 0.6) as f32;
                for x_col in (0..w).step_by(8) {
                    let wave_top = (35.0 + ((x_col as f32 + wave_step) * 0.04).sin() * 22.0) as isize;
                    let wave_bot = (35.0 + ((x_col as f32 - wave_step) * 0.04).cos() * 22.0) as isize;
                    self.draw_rect(buffer, x_col as isize, 0, 8, wave_top as usize, 0xFF4A1024);
                    self.draw_rect(buffer, x_col as isize, h as isize - wave_bot, 8, wave_bot as usize, 0xFF4A1024);
                    // Venas de ácido verde fluorescente
                    let acid_y = wave_top - 4;
                    self.draw_rect(buffer, x_col as isize, acid_y, 8, 4, 0xFF39FF14);
                }
                // Ojos biónicos incrustados en la biomasa
                let eye_blink = (t * 2.0).sin() > -0.7;
                if eye_blink {
                    self.draw_circle(buffer, 180, 50, 12, 0xFFFF0033);
                    self.draw_circle(buffer, 180, 50, 5, 0xFFFFFF00);
                    self.draw_circle(buffer, w as isize - 240, h as isize - 50, 12, 0xFFFF0033);
                    self.draw_circle(buffer, w as isize - 240, h as isize - 50, 5, 0xFFFFFF00);
                }
            }
            // Nivel 5: Laboratorio Secreto & Estación Hephaestus (EscenarioLaboratorioSecreto)
            5 => {
                buffer.fill(0xFF08141E); // Acero y cian técnico
                // Paneles de reactores de fusión y cámaras criogénicas
                let lab_offset = (s * 0.5) as usize % 140;
                for i in 0..(w / 140 + 2) {
                    let rx = (i * 140) as isize - lab_offset as isize;
                    // Cámara cilíndrica de contención criogénica
                    self.draw_rect(buffer, rx + 20, 40, 50, 160, 0xFF142B3D);
                    self.draw_rect(buffer, rx + 26, 46, 38, 148, 0xFF00D2FF);
                    self.draw_rect(buffer, rx + 32, 52, 26, 136, 0xFFE0FFFF);
                }
            }
            // Nivel 6: Bosque Cibernético & Cinturón de Asteroides (EscenarioBosqueCibernetico)
            6 => {
                buffer.fill(0xFF09140C); // Verde fosforescente profundo
                // Árboles biónicos con fibra óptica
                let tree_offset = (s * 0.45) as usize % 110;
                for tr in 0..(w / 110 + 2) {
                    let tx = (tr * 110) as isize - tree_offset as isize;
                    // Tronco mecánico
                    self.draw_rect(buffer, tx + 40, h as isize - 150, 16, 150, 0xFF1A2E20);
                    // Ramas de fibra luminosa
                    self.draw_circle(buffer, tx + 48, h as isize - 150, 34, 0xFF00FF88);
                    self.draw_circle(buffer, tx + 48, h as isize - 150, 20, 0xFF70FFAA);
                }
            }
            // Nivel 7: Nave Nodriza Némesis (Nemesis Mothership)
            7 => {
                buffer.fill(0xFF1B070B); // Rojo carmesí y negro acorazado
                // Blindaje hexagonal y cañones pesados en las mamparas
                let plate_offset = (s * 0.5) as usize % 80;
                for px in 0..(w / 80 + 2) {
                    let rx = (px * 80) as isize - plate_offset as isize;
                    self.draw_rect(buffer, rx, 0, 76, 50, 0xFF3D1219);
                    self.draw_rect(buffer, rx, h as isize - 50, 76, 50, 0xFF3D1219);
                    self.draw_rect(buffer, rx + 10, 48, 14, 18, 0xFFFF0033);
                    self.draw_rect(buffer, rx + 10, h as isize - 66, 14, 18, 0xFFFF0033);
                }
            }
            // Nivel 8: Núcleo de Singularidad IA (Quantum Singularity Core)
            _ => {
                buffer.fill(0xFF05050C); // Vacío cuántico
                // Vórtice gravitacional en espiral de horizonte de sucesos
                let v_cx = (w as isize * 3) / 4;
                let v_cy = (h / 2) as isize;
                let pulse = ((t * 3.0).sin() * 15.0) as usize;
                self.draw_circle(buffer, v_cx, v_cy, 130 + pulse, 0xFF4B0082);
                self.draw_circle(buffer, v_cx, v_cy, 95 + pulse / 2, 0xFF8A2BE2);
                self.draw_circle(buffer, v_cx, v_cy, 60, 0xFF00FFFF);
                self.draw_circle(buffer, v_cx, v_cy, 35, 0xFF000000); // Agujero negro central
            }
        }
    }

    /// Renderiza un Soldado Humano Cibernético estilo Final Mission (Japón) con armadura y jetpack
    fn draw_human_cyber_soldier(&self, buffer: &mut [u32], player: &Player) {
        let px = player.x as isize;
        let py = player.y as isize;
        let dir = if player.facing_right { 1 } else { -1 };
        let t = player.anim_timer;

        // Efecto parpadeo de invulnerabilidad
        if player.invulnerable_timer > 0.0 && ((player.invulnerable_timer * 18.0) as usize % 2 == 0) {
            return;
        }

        // Flotación / oscilación natural de vuelo (hovering)
        let hover_y = (t * 6.0).sin() * 3.0;
        let cy = py + hover_y as isize;

        // 1. Llama de Plasma del Jetpack (animada y oscilante)
        let flame_osc = ((t * 28.0).sin().abs() * 8.0) as isize;
        let jet_x = px - dir * 16;
        let jet_y = cy - 2;

        // Llamarada naranja exterior
        self.draw_rect(buffer, jet_x - dir * (12 + flame_osc), jet_y - 3, (12 + flame_osc) as usize, 8, 0xFFFF5500);
        // Núcleo de plasma cian/blanco interior
        self.draw_rect(buffer, jet_x - dir * (6 + flame_osc / 2), jet_y - 1, (6 + flame_osc / 2) as usize, 4, 0xFF00FFFF);

        // 2. Mochila Propulsora Jetpack (en la espalda del soldado)
        self.draw_rect(buffer, px - dir * 14, cy - 8, 8, 18, 0xFF3D4452); // Cilindro metálico
        self.draw_rect(buffer, px - dir * 16, cy + 8, 6, 6, 0xFF6C7A89);  // Tobera propulsora
        self.draw_circle(buffer, px - dir * 10, cy - 4, 2, 0xFF00FF66);   // LED de estado verde

        // 3. Piernas y Botas Gravitacionales
        self.draw_rect(buffer, px - dir * 4, cy + 10, 6, 12, 0xFF2B303A); // Pierna trasera
        self.draw_rect(buffer, px + dir * 2, cy + 8, 6, 14, 0xFF3A4250);  // Pierna delantera
        self.draw_rect(buffer, px + dir * 2, cy + 20, 8, 4, player.color); // Bota propulsora

        // 4. Torso y Peto de Combate Blindado (Armadura Exo-esqueleto)
        self.draw_rect(buffer, px - dir * 6, cy - 6, 14, 16, player.color);
        // Reactor de energía en el pecho
        self.draw_circle(buffer, px + dir * 2, cy - 1, 3, 0xFFFFFFFF);
        // Hombrera blindada táctica
        self.draw_rect(buffer, px - dir * 8, cy - 8, 10, 6, 0xFFD8D8E0);

        // 5. Cabeza y Casco Táctico con Visor Cibernético
        self.draw_circle(buffer, px + dir * 2, cy - 14, 7, 0xFF2F3542);
        // Visor de combate iluminado (reflejo HUD estilo Final Mission Japón)
        let visor_col = if player.id == 0 { 0xFF00FFFF } else { 0xFFFFD700 };
        self.draw_rect(buffer, px + dir * 4, cy - 16, 6, 4, visor_col);
        self.draw_rect(buffer, px + dir * 6, cy - 16, 2, 2, 0xFFFFFFFF); // Destello de brillo

        // 6. Brazo y Rifle de Asalto de Plasma Pesado
        let gun_x = px + dir * 10;
        let gun_y = cy - 2;
        self.draw_rect(buffer, gun_x, gun_y, 18, 6, 0xFF1E272C);       // Cañón del rifle
        self.draw_rect(buffer, gun_x + dir * 14, gun_y + 1, 6, 4, 0xFF747D8C); // Boca de fuego
        self.draw_rect(buffer, gun_x + 4, gun_y - 3, 5, 3, 0xFFFF0055); // Mira holográfica LED

        // Fogonazo de disparo animado (Muzzle Flash)
        if player.fire_timer > 0.05 {
            let flash_x = gun_x + dir * 20;
            self.draw_circle(buffer, flash_x, gun_y + 3, 6, 0xFFFFFF00);
            self.draw_circle(buffer, flash_x, gun_y + 3, 3, 0xFFFFFFFF);
        }

        // 7. Satélites Orbitales de Apoyo (Final Mission Drones)
        for sat in player.satellites.iter() {
            let sat_x = (player.x + sat.angle.cos() * sat.distance) as isize;
            let sat_y = (player.y + sat.angle.sin() * sat.distance) as isize;
            let sat_color = if sat.is_locked { 0xFFFF0055 } else { 0xFF00E5FF };

            // Anillo exterior de aleación alienígena
            self.draw_circle(buffer, sat_x, sat_y, 9, 0xFF333344);
            self.draw_circle(buffer, sat_x, sat_y, 7, sat_color);
            // Núcleo de energía plasma blanco
            self.draw_circle(buffer, sat_x, sat_y, 3, 0xFFFFFFFF);

            // Haz de mira láser direccional si está fijado (Satellite Lock)
            if sat.is_locked {
                let lx = sat_x + (sat.angle.cos() * 26.0) as isize;
                let ly = sat_y + (sat.angle.sin() * 26.0) as isize;
                self.draw_circle(buffer, lx, ly, 2, 0xFFFF0055);
            }
        }
    }

    /// Renderiza la interfaz de juego limpia y elegante (HUD) SIN BOTONES EN PANTALLA
    fn render_clean_hud(
        &self,
        buffer: &mut [u32],
        players: &[Player],
        stage_num: u8,
        stage_name: &str,
    ) {
        if let Some(p1) = players.first() {
            // 1. Barra de Bio-Armadura Tecnológica de P1
            let max_hp_w = 180;
            let hp_w = ((p1.health / p1.max_health).clamp(0.0, 1.0) * max_hp_w as f32) as usize;
            self.draw_rect(buffer, 24, 20, max_hp_w + 8, 16, 0xFF1B1B26);
            self.draw_rect(buffer, 28, 24, max_hp_w, 8, 0xFF3D3D4E);
            let hp_col = if p1.health > 40.0 { 0xFF00FF77 } else { 0xFFFF3344 };
            self.draw_rect(buffer, 28, 24, hp_w, 8, hp_col);

            // 2. Bombas EMP disponibles (cápsulas azules de pulso)
            for b in 0..p1.bombs.min(6) {
                let bx = 28 + (b as isize * 18);
                self.draw_circle(buffer, bx, 44, 5, 0xFF00E5FF);
                self.draw_circle(buffer, bx, 44, 2, 0xFFFFFFFF);
            }

            // 3. Vidas restantes (Iconos miniatura de cascos cibernéticos)
            for v in 0..p1.lives.max(0).min(6) {
                let vx = 140 + (v as isize * 14);
                self.draw_circle(buffer, vx, 44, 4, 0xFF00D2FF);
            }

            // 4. Indicador del Nivel actual
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
        let _ = stage_name;
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

    /// Mezcla aditiva para simular resplandor (bloom) y luz ambiental
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

    /// Luz ambiental puntual dinámica con atenuación cuadrática suave
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

    /// Renderiza naves y drones enemigos con texturas, blindaje y luces de advertencia
    pub fn draw_enemy(&self, buffer: &mut [u32], e: &Enemy) {
        let ex = e.x as isize;
        let ey = e.y as isize;
        let r = e.radius as usize;
        let t = self.anim_time;

        match e.enemy_type {
            EnemyType::PatrolDrone => {
                // Drone patrullero: Chasis de titanio angular, visor rojo y toberas de iones
                self.draw_rect(buffer, ex - 14, ey - 6, 28, 12, 0xFF2A2D3A);
                self.draw_rect(buffer, ex - 10, ey - 10, 20, 20, 0xFF3E4354);
                let scan = ((t * 8.0).sin() * 4.0) as isize;
                self.draw_rect(buffer, ex - 8 + scan, ey - 2, 6, 4, 0xFFFF0033);
                self.draw_circle(buffer, ex + 14, ey, 3, 0xFF00E5FF);
                self.draw_point_light(buffer, ex + 14, ey, 14, 0xFF00E5FF, 0.4);
            }
            EnemyType::KamikazeWasp => {
                // Avispa suicida: Alas en flecha con franjas amarillas/negras y aguijón de plasma
                self.draw_rect(buffer, ex - 12, ey - 5, 24, 10, 0xFFFFCC00);
                self.draw_rect(buffer, ex - 6, ey - 5, 4, 10, 0xFF111111);
                self.draw_rect(buffer, ex + 2, ey - 5, 4, 10, 0xFF111111);
                self.draw_circle(buffer, ex - 14, ey, 4, 0xFFFF3300);
                self.draw_point_light(buffer, ex - 14, ey, 16, 0xFFFF4400, 0.5);
            }
            EnemyType::LaserTurret => {
                // Torreta pesada blindada: Chasis hexagonal y cañón láser doble
                self.draw_rect(buffer, ex - 18, ey - 18, 36, 36, 0xFF4A4E69);
                self.draw_rect(buffer, ex - 14, ey - 14, 28, 28, 0xFF22223B);
                self.draw_rect(buffer, ex - 28, ey - 8, 14, 4, 0xFF9A8C98);
                self.draw_rect(buffer, ex - 28, ey + 4, 14, 4, 0xFF9A8C98);
                self.draw_circle(buffer, ex, ey, 7, 0xFFC77DFF);
                self.draw_point_light(buffer, ex, ey, 22, 0xFFC77DFF, 0.6);
            }
            EnemyType::CyberCrab => {
                // Cangrejo biomecánico: Caparazón segmentado, pinzas mecánicas y escudo
                self.draw_circle(buffer, ex, ey, r, 0xFF5C677D);
                self.draw_circle(buffer, ex, ey, r.saturating_sub(4), 0xFF1F2430);
                self.draw_rect(buffer, ex - 22, ey - 14, 10, 6, 0xFF7D8597);
                self.draw_rect(buffer, ex - 22, ey + 8, 10, 6, 0xFF7D8597);
                self.draw_circle(buffer, ex, ey, 6, 0xFF00F5D4);
                self.draw_point_light(buffer, ex, ey, 24, 0xFF00F5D4, 0.5);
            }
            EnemyType::AsteroidLeech => {
                // Sanguijuela parasitaria (Abadox): Biomaterial viscoso con ojo central
                self.draw_circle(buffer, ex, ey, r, 0xFF7B2CBF);
                self.draw_circle(buffer, ex, ey, r.saturating_sub(5), 0xFF3C096C);
                let pulse = ((t * 6.0).sin() * 2.0) as usize;
                self.draw_circle(buffer, ex, ey, 5 + pulse, 0xFFFF0055);
                self.draw_circle(buffer, ex, ey, 2, 0xFFFFFFFF);
                self.draw_point_light(buffer, ex, ey, 26, 0xFFFF0055, 0.7);
            }
            EnemyType::StealthStriker => {
                // Caza furtivo: Ala delta negra mate con bordes cian neón
                self.draw_rect(buffer, ex - 16, ey - 8, 32, 16, 0xFF14171E);
                self.draw_rect(buffer, ex - 12, ey - 4, 24, 8, 0xFF0B0D12);
                self.draw_rect(buffer, ex - 18, ey - 10, 4, 20, 0xFF00FFFF);
                self.draw_circle(buffer, ex, ey, 4, 0xFF00FFFF);
                self.draw_point_light(buffer, ex, ey, 20, 0xFF00FFFF, 0.4);
            }
        }
    }

    /// Renderiza jefes colosales (Boss) con múltiples capas, núcleo cuántico pulsante y luz ambiental
    pub fn draw_boss(&self, buffer: &mut [u32], b: &Boss) {
        let bx = b.x as isize;
        let by = b.y as isize;
        let r = b.radius as usize;
        let t = self.anim_time;
        let w = self.width;

        // Casco blindado masivo multicapa
        self.draw_circle(buffer, bx, by, r, 0xFF2B2D42);
        self.draw_circle(buffer, bx, by, r.saturating_sub(6), 0xFF1A1B29);

        // Paneles de titanio y refuerzos angulares
        self.draw_rect(buffer, bx - (r as isize / 2), by - (r as isize / 2), r, r, 0xFF3D405B);

        // Baterías de cañones pesados en la proa
        self.draw_rect(buffer, bx - r as isize - 16, by - 24, 22, 10, 0xFF495057);
        self.draw_rect(buffer, bx - r as isize - 16, by + 14, 22, 10, 0xFF495057);

        // Núcleo cuántico central con pulsación de energía y luz dinámica
        let core_pulse = ((t * 5.0).sin() * 5.0) as usize;
        let core_color = if (t * 4.0).sin() > 0.0 { 0xFFFF0055 } else { 0xFFFF5500 };
        self.draw_circle(buffer, bx, by, 20 + core_pulse, core_color);
        self.draw_circle(buffer, bx, by, 12, 0xFFFFFFFF);

        // Luz dinámica ambiental emitida por el jefe (ilumina el escenario circundante)
        self.draw_point_light(buffer, bx, by, r + 70, core_color, 0.85);

        // Barra de salud superior del Jefe con diseño de advertencia sci-fi
        let max_w = 520;
        let cur_w = ((b.health / b.max_health).clamp(0.0, 1.0) * max_w as f32) as usize;
        let bar_x = (w as isize - max_w as isize) / 2;
        self.draw_rect(buffer, bar_x - 6, 22, max_w + 12, 22, 0xFF550000);
        self.draw_rect(buffer, bar_x - 2, 24, max_w + 4, 18, 0xFF1A0000);
        self.draw_rect(buffer, bar_x, 26, cur_w, 14, 0xFFFF2244);
        self.draw_rect(buffer, bar_x, 26, cur_w, 3, 0xFFFF99AA);
    }
}
