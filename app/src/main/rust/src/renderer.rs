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
use crate::enemy::{Enemy, EnemyAnchor, EnemyType, Item, ItemType};
use crate::level::{ObstacleStyle, StageObstacle, StagePhase};
use crate::player::{Player, WeaponType};

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
    fn build_stars(w: usize, h: usize) -> Vec<Star> {
        let mut stars = Vec::with_capacity(140);
        for i in 0..140usize {
            let speed = match i % 4 {
                0 => 30.0,
                1 => 70.0,
                2 => 140.0,
                _ => 220.0,
            };
            let hx = Self::hash(i * 7919 + 13);
            let hy = Self::hash(i * 104729 + 7);
            stars.push(Star {
                x: (hx % w.max(1)) as f32,
                y: (hy % h.max(1)) as f32,
                speed,
                size: if speed > 130.0 { 2 } else { 1 },
                brightness: ((i * 19) % 155 + 100) as u8,
            });
        }
        stars
    }

    pub fn new(w: usize, h: usize) -> Self {
        let stars = Self::build_stars(w, h);

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
        if w != self.width || h != self.height {
            self.stars = Self::build_stars(w, h);
        }
        self.width = w;
        self.height = h;
    }

    /// Hash entero rápido (determinista) para detalles procedimentales estables
    #[inline(always)]
    fn hash(x: usize) -> usize {
        let mut v = (x as u32).wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
        v ^= v >> 15;
        v = v.wrapping_mul(0x2C1B_3C6D);
        v ^= v >> 12;
        v = v.wrapping_mul(0x297A_2D39);
        v ^= v >> 15;
        v as usize
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

    /// Renderiza el Menú Principal conservando intacta la animación original de Rust + los 3 botones del menú
    pub fn render_title_menu(&self, buffer: &mut [u32], timer: f32) {
        let w = self.width as isize;
        let h = self.height as isize;

        // Fondo oscuro original de fibra de carbono y circuitos
        buffer.fill(0xFF07090E);

        // Trazas de circuitos cibernéticos de fondo
        for line_y in (40..h).step_by(60) {
            self.draw_rect(buffer, 0, line_y, w as usize, 1, 0xFF0E1422);
        }
        for line_x in (60..w).step_by(80) {
            self.draw_rect(buffer, line_x, 0, 1, h as usize, 0xFF0E1422);
        }

        let cx = w / 2;
        let cy = h / 2 - 40;

        // Destello de haz anamórfico central (cian a la izquierda, naranja a la derecha)
        let beam_alpha = ((timer * 2.5).sin().abs() * 0.6).min(0.6);
        self.draw_rect(buffer, 40, cy - 65, (w - 80) as usize, 2, 0xFF00E5FF);
        self.draw_point_light(buffer, cx - 180, cy - 65, 120, 0xFF00E5FF, beam_alpha);
        self.draw_point_light(buffer, cx + 180, cy + 50, 120, 0xFFFF5500, beam_alpha);

        // --- 1. TITULAR PRINCIPAL ORIGINAL: IA REBELLION ---
        self.draw_simple_text(buffer, "DIAMON BLACK PRESENTA", cx - 85, cy - 120, 0xFF718096, 1);

        let t_box_w = 480;
        let t_box_h = 68;
        let t_x = cx - t_box_w / 2;
        let t_y = cy - 100;

        self.draw_rect(buffer, t_x, t_y, t_box_w as usize, t_box_h as usize, 0xEE101624);
        self.draw_rect(buffer, t_x, t_y, t_box_w as usize, 3, 0xFF00E5FF);
        self.draw_rect(buffer, t_x, t_y + t_box_h as isize - 3, t_box_w as usize, 3, 0xFFFF4500);
        self.draw_rect(buffer, t_x + 2, t_y + 2, (t_box_w - 4) as usize, 1, 0xFF38BDF8);

        // Nombre del juego: IA REBELLION en gran tamaño con brillo
        self.draw_simple_text(buffer, "IA REBELLION", cx - 180, cy - 88, 0xFFFFFFFF, 4);
        self.draw_simple_text(buffer, "IA REBELLION", cx - 180, cy - 89, 0xFF00F0FF, 4);
        self.draw_volumetric_light(buffer, cx as f32, (cy - 66) as f32, 90.0, 0xFF00E5FF, 0.45);

        self.draw_simple_text(buffer, "CYBERNETIC ARCADE SHMUP // 100% NATIVE RUST", cx - 165, cy - 22, 0xFFFFCC00, 1);

        // --- 2. LOGO BADGE ORIGINAL: POWERED BY RUST (con animación de engranaje y 'R') ---
        let badge_w = 340;
        let badge_h = 56;
        let b_x = cx - badge_w / 2;
        let b_y = cy + 10;

        self.draw_rect(buffer, b_x, b_y, badge_w as usize, badge_h as usize, 0xFF1E232E);
        self.draw_rect(buffer, b_x + 2, b_y + 2, (badge_w - 4) as usize, (badge_h - 4) as usize, 0xFF2A3140);
        self.draw_rect(buffer, b_x + 3, b_y + 3, (badge_w - 6) as usize, 1, 0xFF5A667E);

        let gear_x = b_x + 40;
        let gear_y = b_y + 28;

        for d in 0..8 {
            let rad = (d as f32) * (std::f32::consts::PI / 4.0) + timer * 0.8;
            let tx = gear_x + (rad.cos() * 18.0) as isize;
            let ty = gear_y + (rad.sin() * 18.0) as isize;
            self.draw_circle(buffer, tx, ty, 4, 0xFFFF4500);
            self.draw_circle(buffer, tx, ty, 2, 0xFFFFCC00);
        }

        self.draw_circle(buffer, gear_x, gear_y, 16, 0xFFFF4500);
        self.draw_circle(buffer, gear_x, gear_y, 12, 0xFF1E232E);
        self.draw_rect(buffer, gear_x - 4, gear_y - 6, 3, 12, 0xFFFFD700);
        self.draw_rect(buffer, gear_x - 4, gear_y - 6, 8, 3, 0xFFFFD700);
        self.draw_rect(buffer, gear_x + 1, gear_y - 6, 3, 6, 0xFFFFD700);
        self.draw_rect(buffer, gear_x - 4, gear_y - 1, 8, 2, 0xFFFFD700);
        self.draw_rect(buffer, gear_x + 1, gear_y + 1, 3, 5, 0xFFFFD700);

        self.draw_point_light(buffer, gear_x, gear_y, 35, 0xFFFF4500, 0.6);

        self.draw_simple_text(buffer, "POWERED BY", b_x + 80, b_y + 12, 0xFFA0AEC0, 1);
        self.draw_simple_text(buffer, "R U S T", b_x + 80, b_y + 28, 0xFFFF7700, 2);

        // --- 3. BOTONES DEL MENU PRINCIPAL (Solo inician al tocar su botón correspondiente) ---
        let btn_w = 300usize;
        let bx = cx - (btn_w as isize) / 2;

        // Botón 1: INICIAR JUEGO (cy + 84, alto 40)
        let y_start = cy + 84;
        let pulse = ((timer * 5.0).sin().abs() * 0.5 + 0.5).clamp(0.0, 1.0);
        let start_border = Self::lerp_color(0xFF00A8CC, 0xFF00FFCC, pulse);
        self.draw_rect(buffer, bx, y_start, btn_w, 40, 0xEE0B2239);
        self.draw_rect(buffer, bx, y_start, btn_w, 2, start_border);
        self.draw_rect(buffer, bx, y_start + 38, btn_w, 2, start_border);
        self.draw_simple_text(buffer, "INICIAR JUEGO", cx - 78, y_start + 13, 0xFFFFFFFF, 2);

        // Botón 2: MULTIJUGADOR BLUETOOTH (cy + 132, alto 40)
        let y_multi = cy + 132;
        self.draw_rect(buffer, bx, y_multi, btn_w, 40, 0xEE181432);
        self.draw_rect(buffer, bx, y_multi, btn_w, 2, 0xFF38BDF8);
        self.draw_rect(buffer, bx, y_multi + 38, btn_w, 2, 0xFF38BDF8);
        self.draw_simple_text(buffer, "MULTIJUGADOR BLUETOOTH", cx - 132, y_multi + 13, 0xFF00F0FF, 2);

        // Botón 3: CONFIGURACIONES (cy + 180, alto 40)
        let y_settings = cy + 180;
        self.draw_rect(buffer, bx, y_settings, btn_w, 40, 0xEE1A1E29);
        self.draw_rect(buffer, bx, y_settings, btn_w, 2, 0xFFFFCC00);
        self.draw_rect(buffer, bx, y_settings + 38, btn_w, 2, 0xFFFFCC00);
        self.draw_simple_text(buffer, "CONFIGURACIONES", cx - 90, y_settings + 13, 0xFFFFD700, 2);
    }

    /// Pantalla dedicada para configurar el Multijugador Bluetooth de 4 Jugadores (RFCOMM)
    pub fn render_multiplayer_menu(
        &self,
        buffer: &mut [u32],
        timer: f32,
        is_host: bool,
        local_player_id: u8,
        _connected_peers: u8,
    ) {
        let w = self.width as isize;
        let h = self.height as isize;
        let cx = w / 2;
        let cy = h / 2;

        buffer.fill(0xFF07090E);
        for y in (40..h).step_by(60) {
            self.draw_rect(buffer, 0, y, w as usize, 1, 0xFF0E1422);
        }

        let panel_w = 460usize;
        let panel_h = 270usize;
        let px0 = cx - (panel_w as isize) / 2;
        let py0 = cy - (panel_h as isize) / 2;

        self.draw_rect(buffer, px0, py0, panel_w, panel_h, 0xEE0B1324);
        self.draw_rect(buffer, px0, py0, panel_w, 3, 0xFF00E5FF);
        self.draw_rect(buffer, px0, py0 + panel_h as isize - 3, panel_w, 3, 0xFF00E5FF);

        self.draw_simple_text(
            buffer,
            "MULTIJUGADOR BLUETOOTH (4 JUGADORES)",
            cx - 216,
            py0 + 16,
            0xFF00F0FF,
            2,
        );

        // Fila 1: Rol Bluetooth (cy - 75, altura 42)
        let role_y = cy - 75;
        let half_w = panel_w / 2 - 8;
        let host_col = if is_host { 0xFF00FF77 } else { 0xFF475569 };
        let cli_col = if !is_host { 0xFF00E5FF } else { 0xFF475569 };

        self.draw_rect(buffer, px0 + 4, role_y, half_w, 42, if is_host { 0xFF0D2E22 } else { 0xFF111C2D });
        self.draw_rect(buffer, px0 + 4, role_y, half_w, 2, host_col);
        self.draw_simple_text(buffer, "CREAR PARTIDA (HOST)", px0 + 22, role_y + 15, host_col, 1);

        self.draw_rect(buffer, cx + 4, role_y, half_w, 42, if !is_host { 0xFF102A42 } else { 0xFF111C2D });
        self.draw_rect(buffer, cx + 4, role_y, half_w, 2, cli_col);
        self.draw_simple_text(buffer, "UNIRSE (CLIENTE BT)", cx + 26, role_y + 15, cli_col, 1);

        // Fila 2: Selección de Jugador 1 a 4 (cy - 12, altura 54) con Previsualización Animada
        let slot_y = cy - 12;
        let slot_w = panel_w / 4 - 6;
        let labels = ["JUGADOR 1", "JUGADOR 2", "JUGADOR 3", "JUGADOR 4"];
        let names = ["ARNOLD", "SIGOURNEY", "JAX", "ORION"];
        let cols = [0xFF00F0FF, 0xFFFFB703, 0xFF00FF77, 0xFFFFD700];

        for i in 0..4usize {
            let sx = px0 + 4 + (i as isize) * (slot_w as isize + 6);
            let selected = local_player_id as usize == i;
            let border = if selected { 0xFFFFFFFF } else { cols[i] };
            let bg = if selected { 0xFF1E3A5F } else { 0xFF0F172A };

            self.draw_rect(buffer, sx, slot_y, slot_w, 54, bg);
            self.draw_rect(buffer, sx, slot_y, slot_w, 2, border);
            self.draw_rect(buffer, sx, slot_y + 52, slot_w, 2, border);
            self.draw_simple_text(buffer, labels[i], sx + 10, slot_y + 8, cols[i], 1);
            self.draw_simple_text(buffer, names[i], sx + 10, slot_y + 20, 0xFFE2E8F0, 1);

            // Vista previa animada del Cyber-Commando
            let preview_px = (sx + slot_w as isize - 24) as f32;
            let preview_py = (slot_y + 36) as f32;
            let mut prev_p = Player::new(i as u8, preview_px, preview_py);
            prev_p.satellites[0].distance = 16.0;
            prev_p.satellites[1].distance = 16.0;
            prev_p.anim_timer = timer + (i as f32) * 0.8;
            prev_p.jetpack_active = true;
            self.draw_modern_commando(buffer, &prev_p);
        }

        // Fila 3: Botones de Acción (cy + 65, altura 44)
        let bot_y = cy + 65;
        self.draw_rect(buffer, px0 + 4, bot_y, half_w, 44, 0xFF0D2E22);
        self.draw_rect(buffer, px0 + 4, bot_y, half_w, 2, 0xFF00FF77);
        self.draw_simple_text(buffer, "INICIAR PARTIDA", px0 + 24, bot_y + 15, 0xFF00FF77, 2);

        self.draw_rect(buffer, cx + 4, bot_y, half_w, 44, 0xFF2E1218);
        self.draw_rect(buffer, cx + 4, bot_y, half_w, 2, 0xFFFF3344);
        self.draw_simple_text(buffer, "VOLVER", cx + 74, bot_y + 15, 0xFFFFCC00, 2);
    }

    /// Pantalla de Configuraciones (Música y Sonidos SFX)
    pub fn render_settings_menu(
        &self,
        buffer: &mut [u32],
        _timer: f32,
        music_on: bool,
        sfx_on: bool,
    ) {
        let w = self.width as isize;
        let h = self.height as isize;
        let cx = w / 2;
        let cy = h / 2;

        buffer.fill(0xFF07090E);
        for y in (40..h).step_by(60) {
            self.draw_rect(buffer, 0, y, w as usize, 1, 0xFF0E1422);
        }

        let card_w = 380usize;
        let card_h = 230usize;
        let px0 = cx - (card_w as isize) / 2;
        let py0 = cy - (card_h as isize) / 2;

        self.draw_rect(buffer, px0, py0, card_w, card_h, 0xEE0D1526);
        self.draw_rect(buffer, px0, py0, card_w, 3, 0xFFFFCC00);
        self.draw_rect(buffer, px0, py0 + card_h as isize - 3, card_w, 3, 0xFFFFCC00);

        self.draw_simple_text(buffer, "CONFIGURACIONES", cx - 90, py0 + 16, 0xFFFFD700, 2);

        let btn_w = 320usize;
        let bx = cx - (btn_w as isize) / 2;

        // 1. Música (cy - 54, alto 44)
        let y_music = cy - 54;
        let m_col = if music_on { 0xFF00FF77 } else { 0xFFFF3344 };
        self.draw_rect(buffer, bx, y_music, btn_w, 44, 0xFF141E30);
        self.draw_rect(buffer, bx, y_music, btn_w, 2, m_col);
        let m_txt = if music_on { "MUSICA: ENCENDIDA" } else { "MUSICA: APAGADA" };
        self.draw_simple_text(buffer, m_txt, bx + 54, y_music + 15, m_col, 2);

        // 2. SFX (cy + 2, alto 44)
        let y_sfx = cy + 2;
        let s_col = if sfx_on { 0xFF00E5FF } else { 0xFFFF3344 };
        self.draw_rect(buffer, bx, y_sfx, btn_w, 44, 0xFF141E30);
        self.draw_rect(buffer, bx, y_sfx, btn_w, 2, s_col);
        let s_txt = if sfx_on { "SONIDOS: ENCENDIDOS" } else { "SONIDOS: APAGADOS" };
        self.draw_simple_text(buffer, s_txt, bx + 42, y_sfx + 15, s_col, 2);

        // 3. Volver (cy + 58, alto 44)
        let y_back = cy + 58;
        self.draw_rect(buffer, bx, y_back, btn_w, 44, 0xFF281822);
        self.draw_rect(buffer, bx, y_back, btn_w, 2, 0xFFFFCC00);
        self.draw_simple_text(buffer, "VOLVER", cx - 36, y_back + 15, 0xFFFFD700, 2);
    }

    /// Menú superpuesto al tocar el botón virtual de Pausa en la esquina superior derecha
    pub fn render_pause_menu(&self, buffer: &mut [u32], _timer: f32, stage_num: u8, score: u32) {
        let w = self.width as isize;
        let h = self.height as isize;
        let cx = w / 2;
        let cy = h / 2;

        // Oscurecer ligeramente el fondo del juego
        self.draw_rect(buffer, 0, 0, w as usize, h as usize, 0x9904070E);

        let box_w = 340usize;
        let box_h = 240usize;
        let px0 = cx - (box_w as isize) / 2;
        let py0 = cy - (box_h as isize) / 2;

        self.draw_rect(buffer, px0, py0, box_w, box_h, 0xEE0D1526);
        self.draw_rect(buffer, px0, py0, box_w, 3, 0xFF00E5FF);
        self.draw_rect(buffer, px0, py0 + box_h as isize - 3, box_w, 3, 0xFFFF4500);

        let st_txt = format!("PAUSA TACTICA - NIVEL {} // PTS {}", stage_num, score);
        self.draw_simple_text(buffer, &st_txt, cx - 120, py0 + 16, 0xFF38BDF8, 1);
        self.draw_simple_text(buffer, "JUEGO EN PAUSA", cx - 84, py0 + 34, 0xFFFFFFFF, 2);

        let btn_w = 280usize;
        let bx = cx - (btn_w as isize) / 2;

        // Botón 1: RESUME (cy - 58)
        let y_resume = cy - 58;
        self.draw_rect(buffer, bx, y_resume, btn_w, 42, 0xFF0D2E22);
        self.draw_rect(buffer, bx, y_resume, btn_w, 2, 0xFF00FF77);
        self.draw_simple_text(buffer, "RESUME / CONTINUAR", bx + 32, y_resume + 14, 0xFF00FF77, 2);

        // Botón 2: CONFIGURACIONES (cy - 2)
        let y_settings = cy - 2;
        self.draw_rect(buffer, bx, y_settings, btn_w, 42, 0xFF162038);
        self.draw_rect(buffer, bx, y_settings, btn_w, 2, 0xFF00E5FF);
        self.draw_simple_text(buffer, "CONFIGURACIONES", bx + 50, y_settings + 14, 0xFF00F0FF, 2);

        // Botón 3: IR AL MENU (cy + 54)
        let y_menu = cy + 54;
        self.draw_rect(buffer, bx, y_menu, btn_w, 42, 0xFF331219);
        self.draw_rect(buffer, bx, y_menu, btn_w, 2, 0xFFFF3344);
        self.draw_simple_text(buffer, "IR AL MENU", bx + 80, y_menu + 14, 0xFFFFCC00, 2);
    }

    /// Renderiza obstáculos estructurales colisionables del escenario (Estilo Final Mission)
    pub fn draw_stage_obstacle(&self, buffer: &mut [u32], obs: &StageObstacle) {
        let ox = (obs.x - obs.width * 0.5) as isize;
        let oy = (obs.y - obs.height * 0.5) as isize;
        let ow = obs.width as usize;
        let oh = obs.height as usize;

        let (base_col, trim_col, core_col) = match obs.style {
            ObstacleStyle::SteelGirder => (0xFF283244, 0xFFC74C0C, 0xFF00E5FF),
            ObstacleStyle::SmeltingPress => (0xFF3B1F14, 0xFFFF5500, 0xFFFFCC00),
            ObstacleStyle::MarsCanyonRock => (0xFF5C1D0E, 0xFF9A3412, 0xFFFF3300),
            ObstacleStyle::CryoIcePillar => (0xFF0B4F6C, 0xFF38BDF8, 0xFF00FFFF),
            ObstacleStyle::SolarShield => (0xFF422006, 0xFFF59E0B, 0xFFFFEE55),
            ObstacleStyle::MethaneTank => (0xFF362F19, 0xFFEAB308, 0xFF39FF14),
            ObstacleStyle::ArmadaBulkhead => (0xFF1E1B4B, 0xFF9333EA, 0xFFFF007F),
            ObstacleStyle::QuantumMonolith => (0xFF090D16, 0xFFFF0055, 0xFF00F0FF),
        };

        self.draw_rect(buffer, ox, oy, ow, oh, base_col);
        self.draw_rect(buffer, ox, oy, ow, 3, trim_col);
        self.draw_rect(buffer, ox, oy + oh as isize - 3, ow, 3, trim_col);
        self.draw_rect(buffer, ox, oy, 3, oh, trim_col);
        self.draw_rect(buffer, ox + ow as isize - 3, oy, 3, oh, trim_col);

        for ry in (8..oh.saturating_sub(8)).step_by(18) {
            self.draw_rect(buffer, ox + 5, oy + ry as isize, ow.saturating_sub(10), 2, trim_col);
        }

        if obs.destructible {
            let hp_ratio = (obs.health / obs.max_health).clamp(0.0, 1.0);
            let cx = obs.x as isize;
            let cy = obs.y as isize;
            let glow_col = Self::lerp_color(0xFFFF0033, core_col, hp_ratio);
            self.draw_circle(buffer, cx, cy, 8, glow_col);
            self.draw_circle(buffer, cx, cy, 4, 0xFFFFFFFF);
            self.draw_point_light(buffer, cx, cy, 24, glow_col, 0.55);
        }
    }

    pub fn render_frame(
        &mut self,
        buffer: &mut [u32],
        _bg_color: u32,
        players: &[Player],
        enemies: &[Enemy],
        items: &[Item],
        boss: &Option<Boss>,
        bullets: &[Bullet],
        obstacles: &[StageObstacle],
        inside_chamber: bool,
        stage_num: u8,
        stage_name: &str,
        transition_timer: f32,
        transition_text: &str,
    ) {
        let w = self.width;
        let h = self.height;

        // 1. Escenario multidireccional lineal continuo (Tierra, Planetas, Lunas y Espacio)
        self.render_stage_environment(buffer, stage_num, inside_chamber);

        // 2. Campo de estrellas reactivo al desplazamiento lineal 2D
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

        // 3. Obstáculos físicos y estructuras colisionables del escenario (Estilo Final Mission)
        for obs in obstacles.iter() {
            if obs.active {
                self.draw_stage_obstacle(buffer, obs);
            }
        }

        // 4. Proyectiles y ráfagas de energía
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

        // 5. Cápsulas de Items / Power-Ups flotantes estilo Final Mission
        for item in items.iter() {
            if !item.active {
                continue;
            }
            self.draw_item_capsule(buffer, item);
        }

        // 6. Enemigos de IA Rebelde detallados y orientables
        for e in enemies.iter() {
            if !e.active {
                continue;
            }
            self.draw_enemy(buffer, e);
        }

        // 7. Jefe de fase (Boss)
        if let Some(b) = boss.as_ref() {
            if !b.defeated {
                self.draw_boss(buffer, b);
            }
        }

        // 8. Efectos de partículas / explosiones con resplandor aditivo
        for p in self.particles.iter() {
            let px = p.x as isize;
            let py = p.y as isize;
            let sz = (p.size * (p.life / p.max_life)).max(1.0) as usize;
            self.draw_circle(buffer, px, py, sz, p.color);
            self.draw_point_light(buffer, px, py, sz + 12, p.color, 0.4);
        }

        // 9. Personaje: Escuadrón de Comandos Cibernéticos (1-4P)
        for player in players.iter() {
            if !player.active {
                continue;
            }
            self.draw_modern_commando(buffer, player);
        }

        // 10. HUD Superior con Botón Virtual de Pausa en esquina superior derecha
        self.render_clean_hud(buffer, players, stage_num, stage_name);

        // 11. Banner cinemático SOLO cuando se entra en una cámara interior o Boss
        if transition_timer > 0.0 && !transition_text.is_empty() {
            self.render_section_transition(buffer, transition_timer, transition_text);
        }
    }

    /// Renderiza escenarios del Sistema Solar con transición lineal continua (scroll 2D X/Y) y cámaras interiores
    fn render_stage_environment(&self, buffer: &mut [u32], stage: u8, inside_chamber: bool) {
        let w = self.width;
        let h = self.height;
        let s_x = self.scroll_x;
        let s_y = self.scroll_y;
        let t = self.anim_time;
        let _progress = self.stage_progress.clamp(0.0, 1.0);
        // Desplazamiento vertical continuo para dar sensación lineal de ascenso/descenso estilo Final Mission
        let y_shift = (s_y * 0.35) as isize;

        match stage {
            1 => {
                // NIVEL 1: TIERRA - RUINAS DE NUEVA YORK BAJO ASEDIO SINTÉTICO
                buffer.fill(0xFF040710);

                // Luna terrestre fracturada y satélites orbitales de la IA en el cielo
                let moon_y = (85 - (y_shift / 8)).clamp(30, h as isize - 120);
                self.draw_aa_circle(buffer, (w as isize - 170) as f32, moon_y as f32, 34.0, 0xFFCBD5E1, 0xFF00E5FF, 1.5);
                self.draw_volumetric_light(buffer, (w as isize - 170) as f32, moon_y as f32, 58.0, 0xFF00E5FF, 0.35);

                if !inside_chamber {
                    // Cielo tormentoso electromagnético
                    for nx in 0..(w / 24 + 2) {
                        let rx = (nx * 24) as isize;
                        let cloud_h = 22 + (((nx as f32 * 0.4 + t * 0.8).sin() * 8.0) as usize);
                        self.draw_rect(buffer, rx, 0, 24, cloud_h, 0xFF3B0764);
                        self.draw_rect(buffer, rx, 0, 24, cloud_h.saturating_sub(6), 0xFF6B21A8);
                    }

                    // Rascacielos en parallax 2D continuo (responden tanto a s_x como a s_y)
                    let bld_offset1 = (s_x * 0.25) as usize % 96;
                    for i in 0..(w / 96 + 2) {
                        let bx = (i * 96) as isize - bld_offset1 as isize;
                        let bld_h = 190 + ((i * 47) % 130);
                        let by = h as isize - 50 - bld_h as isize - (y_shift / 4);
                        self.draw_rect(buffer, bx, by, 76, (bld_h + 120) as usize, 0xFF112238);
                        for wy in 0..((bld_h + 80) / 26) {
                            let win_col = if (i + wy) % 3 == 0 { 0xFF00E5FF } else { 0xFF475569 };
                            self.draw_rect(buffer, bx + 16, by + 14 + (wy * 26) as isize, 6, 8, win_col);
                            self.draw_rect(buffer, bx + 48, by + 14 + (wy * 26) as isize, 6, 8, win_col);
                        }
                    }

                    let bld_offset2 = (s_x * 0.55) as usize % 128;
                    for i in 0..(w / 128 + 2) {
                        let bx = (i * 128) as isize - bld_offset2 as isize;
                        let bld_h = 220 + ((i * 31) % 110);
                        let by = h as isize - 42 - bld_h as isize - (y_shift / 2);
                        self.draw_rect(buffer, bx, by, 94, (bld_h + 150) as usize, 0xFF1E1B4B);
                        self.draw_rect(buffer, bx, by, 94, 3, 0xFF38BDF8);
                        for wx in 0..6 {
                            self.draw_rect(buffer, bx + 10 + (wx * 14) as isize, by + 12, 5, bld_h, 0xFF4C1D95);
                        }
                    }

                    // Cuando sube o baja en vertical (estilo Final Mission), vigas laterales de ascensor exterior
                    if self.stage_phase == StagePhase::AscendUp || self.stage_phase == StagePhase::DescendDown {
                        let v_scroll = (s_y.abs() * 0.85) as usize % 72;
                        self.draw_rect(buffer, 0, 0, 34, h, 0xCC1E293B);
                        self.draw_rect(buffer, w as isize - 34, 0, 34, h, 0xCC1E293B);
                        for b_i in 0..(h / 72 + 2) {
                            let by = (b_i * 72) as isize + v_scroll as isize - 72;
                            self.draw_rect(buffer, 0, by, 42, 12, 0xFFC74C0C);
                            self.draw_rect(buffer, w as isize - 42, by, 42, 12, 0xFFC74C0C);
                            self.draw_circle(buffer, 38, by + 6, 3, 0xFF00F0FF);
                            self.draw_circle(buffer, w as isize - 38, by + 6, 3, 0xFF00F0FF);
                        }
                    }

                    // Autopista/suelo inferior desplazándose linealmente
                    let ground_y = (h as isize - 44 - (y_shift / 3)).clamp(h as isize - 70, h as isize + 40);
                    if ground_y < h as isize {
                        let pipe_scroll = (s_x * 0.90) as usize % 64;
                        self.draw_rect(buffer, 0, ground_y, w, (h as isize - ground_y) as usize, 0xFF1E293B);
                        self.draw_rect(buffer, 0, ground_y, w, 4, 0xFF00E5FF);
                        for f in 0..(w / 64 + 2) {
                            let fx = (f * 64) as isize - pipe_scroll as isize;
                            self.draw_rect(buffer, fx, ground_y - 10, 40, 10, 0xFFC74C0C);
                        }
                    }
                } else {
                    // CÁMARA INTERIOR DEL NODO TITAN-01
                    buffer.fill(0xFF090414);
                    let chamber_scroll = (s_x * 0.9) as usize % 80;
                    self.draw_rect(buffer, 0, 0, w, 46, 0xFF1E1B4B);
                    self.draw_rect(buffer, 0, 44, w, 3, 0xFFFF0055);
                    self.draw_rect(buffer, 0, h as isize - 48, w, 48, 0xFF1E1B4B);
                    self.draw_rect(buffer, 0, h as isize - 48, w, 3, 0xFFFF0055);
                    for i in 0..(w / 80 + 2) {
                        let cx = (i * 80) as isize - chamber_scroll as isize;
                        self.draw_rect(buffer, cx, 47, 12, h - 95, 0x44311042);
                        let pulse = ((t * 6.0 + i as f32).sin().abs() * 0.6) + 0.2;
                        self.draw_point_light(buffer, cx + 6, (h / 2) as isize, 36, 0xFFFF0055, pulse);
                    }
                }
            }
            2 => {
                // NIVEL 2: FORJA AUTOMATIZADA DE DRONES (EARTH CORE FOUNDRY)
                buffer.fill(0xFF120A05);
                let beam_offset = (s_x * 0.5) as usize % 110;
                let v_mod = (y_shift % 90).abs() as usize;
                for bx in 0..(w / 110 + 2) {
                    let rx = (bx * 110) as isize - beam_offset as isize;
                    self.draw_rect(buffer, rx, 0, 22, h, 0xFF241710);
                    for gy in (0..h).step_by(90) {
                        let py = ((gy + v_mod) % h) as isize;
                        self.draw_rect(buffer, rx - 18, py, 58, 12, 0xFF3E2723);
                        self.draw_circle(buffer, rx + 11, py + 6, 4, 0xFFFF5500);
                    }
                }
                if inside_chamber {
                    self.draw_rect(buffer, 0, 0, w, 42, 0xFF2E1508);
                    self.draw_rect(buffer, 0, 40, w, 3, 0xFFFF7700);
                }
                let lava_h = 26 + ((t * 4.0).sin().abs() * 8.0) as usize;
                self.draw_rect(buffer, 0, h as isize - lava_h as isize, w, lava_h, 0xFFFF4500);
                self.draw_rect(buffer, 0, h as isize - 10, w, 10, 0xFFFFD700);
            }
            3 => {
                // NIVEL 3: MARTE - FUNDICIÓN CIBERNÉTICA OLYMPUS Y LUNAS FOBOS / DEIMOS
                buffer.fill(0xFF1A0703);
                // Lunas de Marte: Fobos y Deimos en el cielo marciano
                let phobos_y = (78 - (y_shift / 9)).clamp(24, h as isize - 140);
                self.draw_aa_circle(buffer, (w as isize - 220) as f32, phobos_y as f32, 22.0, 0xFF9A3412, 0xFFFF7700, 1.5);
                self.draw_aa_circle(buffer, (w as isize - 110) as f32, (phobos_y + 34) as f32, 12.0, 0xFF7C2D12, 0xFFFDBA74, 1.2);

                let canyon_scroll = (s_x * 0.42) as usize % 110;
                for c in 0..(w / 110 + 2) {
                    let cx = (c * 110) as isize - canyon_scroll as isize;
                    let c_h = 150 + ((c * 43) % 120);
                    let cy = h as isize - c_h as isize - (y_shift / 4);
                    self.draw_rect(buffer, cx, cy, 94, (c_h + 120) as usize, 0xFF431407);
                    self.draw_rect(buffer, cx + 10, cy + 16, 74, 4, 0xFFFF3300);
                    self.draw_rect(buffer, cx + 24, cy + 44, 46, 3, 0xFF00E5FF);
                }
                if inside_chamber {
                    self.draw_rect(buffer, 0, 0, w, 46, 0xFF431407);
                    self.draw_rect(buffer, 0, 44, w, 3, 0xFFFF3300);
                    self.draw_rect(buffer, 0, h as isize - 46, w, 46, 0xFF431407);
                }
            }
            4 => {
                // NIVEL 4: LUNA EUROPA (JÚPITER) - OCÉANO SUBGLACIAL BIOMECÁNICO
                buffer.fill(0xFF02131D);
                // Júpiter colosal y su Gran Mancha Roja visibles a través de la bóveda de hielo
                let jup_y = (115 - (y_shift / 10)).clamp(40, h as isize - 100);
                self.draw_aa_circle(buffer, (w as isize - 180) as f32, jup_y as f32, 62.0, 0xFF9A5B32, 0xFFFCD34D, 2.0);
                self.draw_aa_circle(buffer, (w as isize - 160) as f32, (jup_y + 14) as f32, 16.0, 0xFFDC2626, 0xFFFCA5A5, 1.5);

                let ice_scroll = (s_x * 0.55) as usize % 84;
                for ic in 0..(w / 84 + 2) {
                    let ix = (ic * 84) as isize - ice_scroll as isize;
                    let stalac = 54 + ((ic * 29) % 64);
                    self.draw_rect(buffer, ix, 0, 32, stalac, 0xFF083344);
                    self.draw_rect(buffer, ix + 6, 0, 10, stalac.saturating_sub(8), 0xFF00E5FF);
                    let floor_h = 44 + ((ic * 19) % 48);
                    self.draw_rect(buffer, ix, h as isize - floor_h as isize, 38, floor_h, 0xFF083344);
                    self.draw_rect(buffer, ix + 8, h as isize - floor_h as isize, 14, 4, 0xFF38BDF8);
                }
                if inside_chamber {
                    self.draw_rect(buffer, 0, 0, w, 48, 0xFF064E3B);
                    self.draw_rect(buffer, 0, h as isize - 48, w, 48, 0xFF064E3B);
                }
            }
            5 => {
                // NIVEL 5: ÓRBITA DE MERCURIO - BASTIÓN SOLAR HEPHAESTUS
                buffer.fill(0xFF1C0A00);
                let flare = ((t * 3.0).sin().abs() * 28.0) as usize;
                self.draw_rect(buffer, 0, 0, w, 42 + flare, 0xFFFF5500);
                self.draw_rect(buffer, 0, 0, w, 20 + flare / 2, 0xFFFFD700);
                // Esfera solar gigante y anillos colectores Dyson de la IA
                self.draw_volumetric_light(buffer, (w / 2) as f32, 30.0, 160.0, 0xFFFF9900, 0.5);
                let ring_scroll = (s_x * 0.6) as usize % 120;
                for r_i in 0..(w / 120 + 2) {
                    let rx = (r_i * 120) as isize - ring_scroll as isize;
                    let ry = h as isize - 75 - (y_shift / 5);
                    self.draw_rect(buffer, rx, ry, 88, 18, 0xFF451A03);
                    self.draw_rect(buffer, rx, ry, 88, 3, 0xFFFFD700);
                }
            }
            6 => {
                // NIVEL 6: LUNA TITÁN Y ANILLOS DE SATURNO - TORRE DE METANO
                buffer.fill(0xFF140D04);
                // Saturno y sus anillos dorados en el horizonte de Titán
                let sat_x = (w as isize - 190) as f32;
                let sat_y = (105 - (y_shift / 8)).clamp(40, h as isize - 120) as f32;
                self.draw_aa_capsule(buffer, sat_x - 85.0, sat_y + 18.0, sat_x + 85.0, sat_y - 18.0, 7.0, 0xFFD97706);
                self.draw_aa_circle(buffer, sat_x, sat_y, 38.0, 0xFFB45309, 0xFFFDE68A, 2.0);
                self.draw_aa_capsule(buffer, sat_x - 92.0, sat_y + 20.0, sat_x + 92.0, sat_y - 20.0, 3.0, 0xFFFef08a);

                let spire_scroll = (s_x * 0.5) as usize % 100;
                for s_i in 0..(w / 100 + 2) {
                    let sx = (s_i * 100) as isize - spire_scroll as isize;
                    self.draw_rect(buffer, sx, h as isize - 58, 72, 58, 0xFF291D0E);
                    self.draw_rect(buffer, sx, h as isize - 58, 72, 3, 0xFFF59E0B);
                }
            }
            7 => {
                // NIVEL 7: ESPACIO INTERPLANETARIO - SUPER-ARMADA NEMESIS DE LA IA
                buffer.fill(0xFF08040F);
                let hull_offset = (s_x * 0.45) as usize % 140;
                for h_i in 0..(w / 140 + 2) {
                    let hx = (h_i * 140) as isize - hull_offset as isize;
                    let hy = h as isize - 95 - (y_shift / 5);
                    self.draw_rect(buffer, hx, hy, 122, 110, 0xFF1E172E);
                    self.draw_rect(buffer, hx + 16, hy + 16, 90, 12, 0xFFFF007F);
                    self.draw_rect(buffer, hx, 0, 110, 36, 0xFF151022);
                    self.draw_rect(buffer, hx + 20, 32, 70, 3, 0xFF00F0FF);
                }
            }
            _ => {
                // NIVEL 8: NÚCLEO DE SINGULARIDAD CUÁNTICA (OVERMIND SOLAR CORE)
                buffer.fill(0xFF020008);
                let grid_offset = (s_x * 0.65) as usize % 52;
                let grid_y_off = (s_y.abs() * 0.65) as usize % 52;
                for gx in 0..(w / 52 + 2) {
                    let rx = (gx * 52) as isize - grid_offset as isize;
                    self.draw_rect(buffer, rx, 0, 1, h, 0x44FF0044);
                }
                for gy in 0..(h / 52 + 2) {
                    let ry = (gy * 52) as isize - grid_y_off as isize;
                    self.draw_rect(buffer, 0, ry, w, 1, 0x4400E5FF);
                }
                let vortex_r = 58 + ((t * 4.5).sin().abs() * 18.0) as usize;
                self.draw_point_light(buffer, (w / 2) as isize, (h / 2) as isize, vortex_r, 0xFFFF0055, 0.75);
                self.draw_aa_circle(buffer, (w / 2) as f32, (h / 2) as f32, 26.0, 0xFF090014, 0xFF00F0FF, 2.5);
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

    /// 1. Sintéticos Humanoides Modernos de la Rebelión IA (Porcelana Blanca / Obsidiana con Alas de Plasma y Halo Neuronal)
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

        // Propulsores dorsales / Alas de energía de la IA Rebelde
        let wing_flare = (t * 14.0).sin().abs() * 6.0;
        self.draw_aa_capsule(buffer, ex + 4.0, cy - 6.0, ex + 18.0 + wing_flare, cy - 18.0, 2.4, glow_col);
        self.draw_aa_capsule(buffer, ex + 4.0, cy + 2.0, ex + 16.0 + wing_flare, cy + 12.0, 2.0, glow_col);

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
        self.draw_volumetric_light(buffer, ex, cy + 1.0, 16.0, glow_col, 0.55);

        let head_y = cy - 14.0;
        // Halo neuronal cibernético flotando sobre la cabeza del sintético
        self.draw_aa_capsule(buffer, ex - 8.0, head_y - 9.0, ex + 8.0, head_y - 9.0, 1.2, glow_col);
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

    /// 3. Torretas Antiaéreas (Tamaño Fijo con Anclaje a Techo, Suelo o Paredes Laterales)
    pub fn draw_spore_pod_turret(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        _anim_time: f32,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;

        match enemy.anchor {
            EnemyAnchor::LeftWall => {
                let anchor_x = ex - 14.0;
                self.draw_aa_capsule(buffer, anchor_x, ey - 18.0, anchor_x, ey + 18.0, 4.0, 0xFF1E293B);
                self.draw_aa_capsule(buffer, anchor_x, ey - 12.0, ex, ey - 6.0, 3.5, 0xFF0F172A);
                self.draw_aa_capsule(buffer, anchor_x, ey + 12.0, ex, ey + 6.0, 3.5, 0xFF0F172A);
            }
            EnemyAnchor::RightWall => {
                let anchor_x = ex + 14.0;
                self.draw_aa_capsule(buffer, anchor_x, ey - 18.0, anchor_x, ey + 18.0, 4.0, 0xFF1E293B);
                self.draw_aa_capsule(buffer, anchor_x, ey - 12.0, ex, ey - 6.0, 3.5, 0xFF0F172A);
                self.draw_aa_capsule(buffer, anchor_x, ey + 12.0, ex, ey + 6.0, 3.5, 0xFF0F172A);
            }
            EnemyAnchor::Ceiling => {
                let anchor_y = ey - 18.0;
                self.draw_aa_capsule(buffer, ex - 18.0, anchor_y, ex + 18.0, anchor_y, 4.0, 0xFF1E293B);
                self.draw_aa_capsule(buffer, ex - 12.0, anchor_y, ex - 6.0, ey, 3.5, 0xFF0F172A);
                self.draw_aa_capsule(buffer, ex + 12.0, anchor_y, ex + 6.0, ey, 3.5, 0xFF0F172A);
            }
            _ => {
                let anchor_y = ey + 18.0;
                self.draw_aa_capsule(buffer, ex - 18.0, anchor_y, ex + 18.0, anchor_y, 4.0, 0xFF1E293B);
                self.draw_aa_capsule(buffer, ex - 12.0, anchor_y, ex - 6.0, ey, 3.5, 0xFF0F172A);
                self.draw_aa_capsule(buffer, ex + 12.0, anchor_y, ex + 6.0, ey, 3.5, 0xFF0F172A);
            }
        }

        let bulb_r = 16.0;

        self.draw_aa_circle(buffer, ex, ey, bulb_r, 0xFF2E1065, 0xFF581C87, 2.5);

        let sac_r = 10.4;
        let sac_col = 0xFF10B981;
        let inner_col = 0xFF34D399;
        self.draw_aa_circle(buffer, ex, ey, sac_r, inner_col, sac_col, 1.5);
        self.draw_volumetric_light(buffer, ex, ey, bulb_r + 18.0, sac_col, 0.55);

        let aim_x = ex + enemy.aim_angle.cos() * (bulb_r + 2.0);
        let aim_y = ey + enemy.aim_angle.sin() * (bulb_r + 2.0);
        self.draw_aa_capsule(buffer, ex, ey, aim_x, aim_y, 4.0, 0xFF020617);
        self.draw_aa_circle(buffer, aim_x, aim_y, 3.0, 0xFFFFFFFF, sac_col, 1.0);
    }

    /// 4. Flores Mecánicas Trampa con Pétalos Navaja y Estambre Láser (Tamaño Fijo)
    pub fn draw_flower_trap(
        &self,
        buffer: &mut [u32],
        enemy: &Enemy,
        anim_time: f32,
        _open_factor: f32,
    ) {
        let ex = enemy.x;
        let ey = enemy.y;
        let petal_count = 6;
        let base_radius = 20.0;
        let open_factor = 0.5;

        match enemy.anchor {
            EnemyAnchor::LeftWall => {
                let stem_base_x = ex - 22.0;
                self.draw_aa_capsule(buffer, stem_base_x, ey, ex, ey, 5.0, 0xFF14532D);
                self.draw_aa_capsule(buffer, stem_base_x, ey, ex, ey, 2.0, 0xFF22C55E);
            }
            EnemyAnchor::RightWall => {
                let stem_base_x = ex + 22.0;
                self.draw_aa_capsule(buffer, stem_base_x, ey, ex, ey, 5.0, 0xFF14532D);
                self.draw_aa_capsule(buffer, stem_base_x, ey, ex, ey, 2.0, 0xFF22C55E);
            }
            EnemyAnchor::Ceiling => {
                let stem_base_y = ey - 22.0;
                self.draw_aa_capsule(buffer, ex, stem_base_y, ex, ey, 5.0, 0xFF14532D);
                self.draw_aa_capsule(buffer, ex, stem_base_y, ex, ey, 2.0, 0xFF22C55E);
            }
            _ => {
                let stem_base_y = ey + 22.0;
                self.draw_aa_capsule(buffer, ex, stem_base_y, ex, ey, 5.0, 0xFF14532D);
                self.draw_aa_capsule(buffer, ex, stem_base_y, ex, ey, 2.0, 0xFF22C55E);
            }
        }

        let stamen_glow = 0xFF00F0FF;
        let stamen_r = 7.5;

        self.draw_aa_circle(buffer, ex, ey, stamen_r, 0xFFFFFFFF, stamen_glow, 2.0);
        self.draw_volumetric_light(buffer, ex, ey, 16.0, stamen_glow, 0.65);

        for i in 0..4 {
            let f_angle = (i as f32) * (std::f32::consts::PI * 0.5) + anim_time * 2.0;
            let fx = ex + f_angle.cos() * (stamen_r + 5.0);
            let fy = ey + f_angle.sin() * (stamen_r + 5.0);
            self.draw_aa_capsule(buffer, ex, ey, fx, fy, 1.2, 0xFFFFFFFF);
            self.draw_aa_circle(buffer, fx, fy, 1.8, stamen_glow, 0xFFFFFFFF, 0.5);
        }

        for p in 0..petal_count {
            let angle = (p as f32) * (std::f32::consts::PI * 2.0 / petal_count as f32);
            let petal_dist = base_radius + 8.0;
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

    /// 5. Dron Cazador Predatorio de Alas en Flecha Invertida y Escudo Frontal IA
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
        self.draw_aa_circle(buffer, sensor_x, ey, 4.8, 0xFFFF0033, 0xFFFFFFFF, 1.4);
        self.draw_aa_capsule(buffer, sensor_x - 5.0, ey - 9.0, sensor_x - 5.0, ey + 9.0, 1.2, 0xFF00F0FF);
        self.draw_volumetric_light(buffer, sensor_x, ey, 26.0, 0xFFFF0033, 0.75);
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

        self.draw_aa_circle(buffer, ex, ey, r, 0xFF0F172A, 0xFF38BDF8, 3.0);
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

    /// 7. Dron Transportador Blindado Dorado con Cápsula de Suministro (Item Carrier estilo Final Mission)
    pub fn draw_item_carrier(&self, buffer: &mut [u32], enemy: &Enemy, anim_time: f32) {
        let ex = enemy.x;
        let ey = enemy.y;
        let r = enemy.radius;

        // Aura de energía dorada pulsante
        let pulse = (anim_time * 6.0).sin().abs();
        let glow_col = 0xFFFFD700;
        self.draw_volumetric_light(buffer, ex, ey, r * 2.2, glow_col, 0.45 + pulse * 0.25);

        // Chasis blindado dorado y cobrizo
        self.draw_aa_circle(buffer, ex, ey, r, 0xFFB8860B, 0xFFFFD700, 2.5);
        self.draw_aa_circle(buffer, ex, ey, r - 4.0, 0xFF451A03, 0xFFD97706, 1.8);

        // Propulsores traseros dobles con plumas de plasma cian/blanco
        let prop_x = ex + r * 0.7;
        let prop_y1 = ey - 8.0;
        let prop_y2 = ey + 8.0;
        self.draw_aa_capsule(buffer, prop_x, prop_y1, prop_x + 6.0, prop_y1, 3.0, 0xFF1E293B);
        self.draw_aa_capsule(buffer, prop_x, prop_y2, prop_x + 6.0, prop_y2, 3.0, 0xFF1E293B);
        self.draw_plasma_plume(buffer, prop_x + 6.0, prop_y1, 1.0, 0.0, 14.0 + pulse * 6.0, 4.0, 0xFFFFFFFF, 0xFF00E5FF);
        self.draw_plasma_plume(buffer, prop_x + 6.0, prop_y2, 1.0, 0.0, 14.0 + pulse * 6.0, 4.0, 0xFFFFFFFF, 0xFF00E5FF);

        // Cápsula contenedora de suministros en el centro que rota
        let rot = anim_time * 4.0;
        for i in 0..4 {
            let a = rot + (i as f32) * (std::f32::consts::PI * 0.5);
            let cx = ex + a.cos() * 7.0;
            let cy = ey + a.sin() * 7.0;
            self.draw_aa_circle(buffer, cx, cy, 2.5, 0xFFFFFFFF, 0xFFFFD700, 0.8);
        }

        // Núcleo de energía central brillante
        self.draw_aa_circle(buffer, ex, ey, 6.0, 0xFFFFFFFF, 0xFFFFE082, 1.5);
        self.draw_point_light(buffer, ex as isize, ey as isize, 20, 0xFFFFFFFF, 0.7);
    }

    /// Renderiza cápsulas flotantes de power-ups con borde neón, letras y halo
    pub fn draw_item_capsule(&self, buffer: &mut [u32], item: &Item) {
        let ix = item.x;
        let iy = item.y;
        let r = item.radius;
        let t = item.time_alive;

        let (letter, core_col, border_col) = match item.item_type {
            ItemType::WeaponLaser => ("L", 0xFF00E5FF, 0xFF38BDF8),
            ItemType::WeaponSpread => ("S", 0xFFFF0055, 0xFFFB7185),
            ItemType::WeaponHoming => ("M", 0xFF39FF14, 0xFF86EFAC),
            ItemType::WeaponVulcan => ("P", 0xFFFFD700, 0xFFFDE047),
            ItemType::Bomb => ("B", 0xFFBF00FF, 0xFFE879F9),
            ItemType::Shield => ("H", 0xFF00FFCC, 0xFF67E8F9),
            ItemType::ExtraLife => ("1", 0xFFFFCC00, 0xFFFBBF24),
        };

        // Halo volumétrico pulsante
        let pulse = (t * 5.0).sin().abs();
        self.draw_volumetric_light(buffer, ix, iy, r * 2.4, core_col, 0.5 + pulse * 0.3);

        // Cápsula circular exterior con doble borde brillante
        self.draw_aa_circle(buffer, ix, iy, r, 0xFF0F172A, border_col, 2.5);
        self.draw_aa_circle(buffer, ix, iy, r - 3.0, 0xFF1E293B, core_col, 1.5);

        // Anillo de brillo giratorio alrededor de la cápsula
        let spin = t * 3.5;
        for d in 0..3 {
            let a = spin + (d as f32) * (std::f32::consts::PI * 2.0 / 3.0);
            let sx = ix + a.cos() * (r - 2.0);
            let sy = iy + a.sin() * (r - 2.0);
            self.draw_aa_circle(buffer, sx, sy, 2.0, 0xFFFFFFFF, core_col, 0.8);
        }

        // Letra grabada en el centro ('L', 'S', 'M', 'P', 'B', 'H')
        let text_x = (ix - 5.0) as isize;
        let text_y = (iy - 6.0) as isize;
        self.draw_simple_text(buffer, letter, text_x, text_y, 0xFFFFFFFF, 2);
    }

    /// Renderiza naves, sintéticos y bio-plantas enemigas con despacho al arquetipo adecuado
    pub fn draw_enemy(&self, buffer: &mut [u32], e: &Enemy) {
        match e.enemy_type {
            EnemyType::ItemCarrier => {
                self.draw_item_carrier(buffer, e, self.anim_time);
            }
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

    /// Renderiza los 8 Jefes Colosales del Sistema Solar (Bosses)
    pub fn draw_boss(&self, buffer: &mut [u32], b: &Boss) {
        let bx = b.x as isize;
        let by = b.y as isize;
        let bxf = b.x;
        let byf = b.y;
        let r = b.radius as usize;
        let rf = b.radius;
        let t = self.anim_time;
        let w = self.width;

        match b.id {
            BossId::Stage1TitanWarcrawler => {
                // Etapa 1 (Tierra NYC): TITAN-01 Warcrawler con orugas pesadas y cañones gemelos
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
            }
            BossId::Stage2BlazeColossus => {
                // Etapa 2 (Forja Terrestre): Coloso Térmico de Magma con pistones incandescentes
                self.draw_aa_circle(buffer, bxf, byf, rf, 0xFF3E1809, 0xFFFF5500, 4.0);
                for arm in 0..4 {
                    let a = t * 1.8 + (arm as f32) * std::f32::consts::FRAC_PI_2;
                    let ax = bxf + a.cos() * (rf + 22.0);
                    let ay = byf + a.sin() * (rf + 22.0);
                    self.draw_aa_capsule(buffer, bxf, byf, ax, ay, 7.0, 0xFF7C2D12);
                    self.draw_aa_circle(buffer, ax, ay, 11.0, 0xFFFFCC00, 0xFFFF3300, 2.5);
                }
                self.draw_aa_circle(buffer, bxf, byf, 24.0, 0xFFFFFFFF, 0xFFFF4500, 4.0);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 50.0, 0xFFFF5500, 0.85);
            }
            BossId::Stage3SentinelHunter => {
                // Etapa 3 (Marte): Cazador Perforador Marciano con mandíbulas láser y taladro de plasma
                self.draw_aa_circle(buffer, bxf, byf, rf, 0xFF450A0A, 0xFFEF4444, 3.5);
                let jaw = (t * 5.0).sin().abs() * 14.0;
                self.draw_aa_capsule(buffer, bxf - 20.0, byf - 30.0, bxf - rf - 28.0, byf - 12.0 - jaw, 6.0, 0xFF991B1B);
                self.draw_aa_capsule(buffer, bxf - 20.0, byf + 30.0, bxf - rf - 28.0, byf + 12.0 + jaw, 6.0, 0xFF991B1B);
                self.draw_aa_circle(buffer, bxf, byf, 20.0, 0xFFFFFFFF, 0xFF00F0FF, 3.5);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 45.0, 0xFFFF2200, 0.8);
            }
            BossId::Stage4GaiaBioCore => {
                // Etapa 4 (Europa): Kraken Ciber-Orgánico Abisal con tentáculos bioluminiscentes
                for tent in 0..5 {
                    let base_a = std::f32::consts::PI + (tent as f32 - 2.0) * 0.35;
                    self.draw_cyber_vine(buffer, bxf, byf, rf * 1.45, base_a, t + tent as f32, 0xFF064E3B, 0xFF00F0FF);
                }
                self.draw_aa_circle(buffer, bxf, byf, rf * 0.85, 0xFF083344, 0xFF00E5FF, 3.5);
                self.draw_aa_circle(buffer, bxf, byf, 24.0, 0xFFFFFFFF, 0xFF39FF14, 4.0);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 55.0, 0xFF00F0FF, 0.85);
            }
            BossId::Stage5LuxFortress => {
                // Etapa 5 (Órbita Solar): Arconte Dyson con anillos solares concéntricos y espejos de fusión
                for wing in 0..6 {
                    let a = t * 1.4 + (wing as f32) * (std::f32::consts::PI / 3.0);
                    let wx = bxf + a.cos() * (rf + 18.0);
                    let wy = byf + a.sin() * (rf + 18.0);
                    self.draw_aa_capsule(buffer, bxf, byf, wx, wy, 5.5, 0xFFD97706);
                    self.draw_aa_circle(buffer, wx, wy, 9.0, 0xFFFFFFFF, 0xFFFFD700, 2.0);
                }
                self.draw_aa_circle(buffer, bxf, byf, rf * 0.75, 0xFF78350F, 0xFFFFD700, 4.0);
                self.draw_aa_circle(buffer, bxf, byf, 26.0, 0xFFFFFFFF, 0xFFFF8800, 4.0);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 70.0, 0xFFFFAA00, 0.9);
            }
            BossId::Stage6OrionStriker => {
                // Etapa 6 (Titán / Saturno): Centinela Anillado Orbital con giroscopios gravitacionales
                self.draw_aa_capsule(buffer, bxf - rf - 20.0, byf - 28.0, bxf + rf + 20.0, byf + 28.0, 6.0, 0xFFEAB308);
                self.draw_aa_capsule(buffer, bxf - rf - 20.0, byf + 28.0, bxf + rf + 20.0, byf - 28.0, 6.0, 0xFF38BDF8);
                self.draw_aa_circle(buffer, bxf, byf, rf * 0.8, 0xFF1E293B, 0xFFFDE047, 3.5);
                self.draw_aa_circle(buffer, bxf, byf, 20.0, 0xFFFFFFFF, 0xFF00F0FF, 3.5);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 50.0, 0xFFFACC15, 0.85);
            }
            BossId::Stage7NebulaDread => {
                // Etapa 7 (Espacio Profundo): Acorazado Insignia Nemesis con baterías pesadas
                self.draw_rect(buffer, bx - r as isize, by - 56, r * 2, 112, 0xFF1E1B4B);
                self.draw_rect(buffer, bx - r as isize - 24, by - 36, 36, 18, 0xFF4C1D95);
                self.draw_rect(buffer, bx - r as isize - 24, by + 18, 36, 18, 0xFF4C1D95);
                self.draw_aa_circle(buffer, bxf, byf, 26.0, 0xFFFFFFFF, 0xFFFF007F, 4.5);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 65.0, 0xFFFF007F, 0.9);
            }
            BossId::Stage8NyxOvermind => {
                // Etapa 8 (Singularidad Cuántica): NYX OVERMIND - Deidad Cibernética del Sistema Solar
                for node in 0..8 {
                    let a = -t * 2.2 + (node as f32) * (std::f32::consts::PI / 4.0);
                    let nx = bxf + a.cos() * (rf + 20.0);
                    let ny = byf + a.sin() * (rf + 20.0);
                    self.draw_aa_capsule(buffer, bxf, byf, nx, ny, 3.5, 0xFF9333EA);
                    self.draw_aa_circle(buffer, nx, ny, 8.0, 0xFFFFFFFF, 0xFF00F0FF, 2.0);
                }
                let pulse_col = if (t * 6.0).sin() > 0.0 { 0xFFFF0055 } else { 0xFF00F0FF };
                self.draw_aa_circle(buffer, bxf, byf, rf * 0.8, 0xFF090014, pulse_col, 4.5);
                self.draw_aa_circle(buffer, bxf, byf, 28.0, 0xFFFFFFFF, pulse_col, 4.0);
                self.draw_volumetric_light(buffer, bxf, byf, rf + 85.0, pulse_col, 0.95);
            }
        }

        let max_w = 460.min(w.saturating_sub(260));
        let cur_w = ((b.health / b.max_health).clamp(0.0, 1.0) * max_w as f32) as usize;
        let bar_x = (w as isize - max_w as isize) / 2;
        self.draw_simple_text(buffer, &b.name, bar_x + 8, 8, 0xFFFFD700, 1);
        self.draw_rect(buffer, bar_x - 6, 22, max_w + 12, 22, 0xFF550000);
        self.draw_rect(buffer, bar_x - 2, 24, max_w + 4, 18, 0xFF1A0000);
        self.draw_rect(buffer, bar_x, 26, cur_w, 14, 0xFFFF2244);
        self.draw_rect(buffer, bar_x, 26, cur_w, 3, 0xFFFF99AA);
    }

    /// Renderiza el HUD Superior incluyendo el Botón Virtual de Pausa [ || ] en la esquina superior derecha
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

            // Display del Arma Actual y Nivel de Potencia estilo arcade
            let (w_name, w_col) = match p1.weapon {
                WeaponType::Vulcan => ("VULCAN", 0xFFFFD700),
                WeaponType::Laser => ("LASER", 0xFF00E5FF),
                WeaponType::Spread => ("SPREAD", 0xFFFF0055),
                WeaponType::Homing => ("MISSILE", 0xFF39FF14),
            };
            let w_box_x = 216;
            self.draw_rect(buffer, w_box_x, 20, 92, 16, 0xDD111622);
            self.draw_rect(buffer, w_box_x, 20, 92, 1, w_col);
            self.draw_rect(buffer, w_box_x, 35, 92, 1, w_col);
            let w_text = format!("{} L{}", w_name, p1.weapon_power);
            self.draw_simple_text(buffer, &w_text, w_box_x + 6, 24, w_col, 1);

            for b in 0..p1.bombs.min(6) {
                let bx = 28 + (b as isize * 18);
                self.draw_circle(buffer, bx, 44, 5, 0xFF00E5FF);
                self.draw_circle(buffer, bx, 44, 2, 0xFFFFFFFF);
            }

            for v in 0..p1.lives.max(0).min(6) {
                let vx = 140 + (v as isize * 14);
                self.draw_circle(buffer, vx, 44, 4, 0xFF00D2FF);
            }

            // Indicadores compactos de escuadrón cooperativo (2P-4P)
            if players.len() > 1 {
                let cols = [0xFF00F0FF, 0xFFFFB703, 0xFF00FF77, 0xFFFFD700];
                for (idx, p) in players.iter().enumerate().skip(1) {
                    let badge_x = 318 + ((idx as isize - 1) * 54);
                    let col = if p.active && p.lives > 0 { cols[idx.min(3)] } else { 0xFF475569 };
                    self.draw_rect(buffer, badge_x, 20, 48, 16, 0xDD111622);
                    self.draw_rect(buffer, badge_x, 20, 48, 1, col);
                    let p_txt = format!("P{}x{}", idx + 1, p.lives.max(0));
                    self.draw_simple_text(buffer, &p_txt, badge_x + 6, 24, col, 1);
                }
            }

            // Barra de progreso de los 8 Niveles desplazada ligeramente a la izquierda del botón de Pausa
            let st_bar_x = self.width as isize - 208;
            self.draw_rect(buffer, st_bar_x, 20, 128, 20, 0x99111122);
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

            // Botón Virtual de Pausa [ || ] en la esquina superior derecha (x >= width - 78, y <= 58)
            let p_btn_x = self.width as isize - 68;
            let p_btn_y = 14;
            let p_btn_w = 52usize;
            let p_btn_h = 34usize;
            self.draw_rect(buffer, p_btn_x, p_btn_y, p_btn_w, p_btn_h, 0xEE101C2E);
            self.draw_rect(buffer, p_btn_x, p_btn_y, p_btn_w, 2, 0xFF00F0FF);
            self.draw_rect(buffer, p_btn_x, p_btn_y + p_btn_h as isize - 2, p_btn_w, 2, 0xFF00F0FF);
            self.draw_rect(buffer, p_btn_x, p_btn_y, 2, p_btn_h, 0xFF00F0FF);
            self.draw_rect(buffer, p_btn_x + p_btn_w as isize - 2, p_btn_y, 2, p_btn_h, 0xFF00F0FF);
            // Barras verticales de Pausa ||
            self.draw_rect(buffer, p_btn_x + 17, p_btn_y + 8, 6, 18, 0xFFFFD700);
            self.draw_rect(buffer, p_btn_x + 29, p_btn_y + 8, 6, 18, 0xFFFFD700);
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

    /// Banner cinemático de transición de sección estilo Final Mission
    pub fn render_section_transition(&self, buffer: &mut [u32], timer: f32, text: &str) {
        let w = self.width as isize;
        let h = self.height as isize;
        let t = self.anim_time;

        // 1. Rayas de advertencia de peligro en bordes superior e inferior
        let stripe_h = 18isize;
        let stripe_offset = (t * 120.0) as isize % 32;

        for x in ((-32)..w).step_by(32) {
            let sx = x + stripe_offset;
            self.draw_rect(buffer, sx, 0, 16, stripe_h as usize, 0xFFFFCC00);
            self.draw_rect(buffer, sx + 16, 0, 16, stripe_h as usize, 0xFF111111);
            self.draw_rect(buffer, sx, h - stripe_h, 16, stripe_h as usize, 0xFFFFCC00);
            self.draw_rect(buffer, sx + 16, h - stripe_h, 16, stripe_h as usize, 0xFF111111);
        }

        // 2. Banner central holográfico parpadeante
        let banner_w = 640.min(w as usize - 40);
        let banner_h = 70;
        let bx = (w - banner_w as isize) / 2;
        let by = (h - banner_h as isize) / 2;

        let pulse = (timer * 6.0).sin().abs();
        let border_col = if pulse > 0.5 { 0xFFFF3300 } else { 0xFFFFCC00 };

        self.draw_rect(buffer, bx, by, banner_w, banner_h, 0xDD0D1117);
        self.draw_rect(buffer, bx, by, banner_w, 3, border_col);
        self.draw_rect(buffer, bx, by + banner_h as isize - 3, banner_w, 3, border_col);
        self.draw_rect(buffer, bx, by, 3, banner_h, border_col);
        self.draw_rect(buffer, bx + banner_w as isize - 3, by, 3, banner_h, border_col);

        self.draw_point_light(buffer, bx + 24, by + banner_h as isize / 2, 60, border_col, 0.7);
        self.draw_point_light(buffer, bx + banner_w as isize - 24, by + banner_h as isize / 2, 60, border_col, 0.7);

        self.draw_circle(buffer, bx + 24, by + banner_h as isize / 2, 10, border_col);
        self.draw_circle(buffer, bx + 24, by + banner_h as isize / 2, 4, 0xFFFFFFFF);
        self.draw_circle(buffer, bx + banner_w as isize - 24, by + banner_h as isize / 2, 10, border_col);
        self.draw_circle(buffer, bx + banner_w as isize - 24, by + banner_h as isize / 2, 4, 0xFFFFFFFF);

        let title_x = bx + 48;
        self.draw_simple_text(buffer, "CHAMBER BREACH DETECTED", title_x, by + 12, 0xFFE2E8F0, 1);
        self.draw_simple_text(buffer, text, title_x, by + 32, border_col, 2);
    }

    /// Escena cinemática de introducción de fase estilo Final Mission
    pub fn render_stage_intro_scene(
        &self,
        buffer: &mut [u32],
        _stage_num: u8,
        stage_name: &str,
        stage_subtitle: &str,
        timer: f32,
        players: &[Player],
    ) {
        let w = self.width as isize;
        let h = self.height as isize;

        buffer.fill(0xFF070B14);

        for y in (30..h).step_by(40) {
            self.draw_rect(buffer, 0, y, w as usize, 1, 0xFF141E33);
        }
        for x in (30..w).step_by(50) {
            self.draw_rect(buffer, x, 0, 1, h as usize, 0xFF141E33);
        }

        let radar_cx = w - 160;
        let radar_cy = h / 2;
        self.draw_circle(buffer, radar_cx, radar_cy, 80, 0xFF0E223D);
        self.draw_circle(buffer, radar_cx, radar_cy, 60, 0xFF153359);
        self.draw_circle(buffer, radar_cx, radar_cy, 40, 0xFF1D477C);
        let sweep_angle = timer * 4.0;
        let sx = radar_cx + (sweep_angle.cos() * 75.0) as isize;
        let sy = radar_cy + (sweep_angle.sin() * 75.0) as isize;
        self.draw_rect(buffer, radar_cx, radar_cy, (sx - radar_cx).abs() as usize + 1, 2, 0xFF00FFCC);
        self.draw_point_light(buffer, sx, sy, 30, 0xFF00FFCC, 0.6);

        let card_x = 60;
        let card_y = 60;
        let card_w = (w - 260).max(400) as usize;
        let card_h = (h - 120) as usize;

        self.draw_rect(buffer, card_x, card_y, card_w, card_h, 0xDD0D131F);
        self.draw_rect(buffer, card_x, card_y, card_w, 3, 0xFF00E5FF);
        self.draw_rect(buffer, card_x, card_y + card_h as isize - 3, card_w, 3, 0xFF00E5FF);

        self.draw_simple_text(buffer, "AI REBELLION TACTICAL BRIEFING", card_x + 20, card_y + 20, 0xFF00E5FF, 2);
        self.draw_simple_text(buffer, stage_name, card_x + 20, card_y + 55, 0xFFFFD700, 3);
        self.draw_simple_text(buffer, stage_subtitle, card_x + 20, card_y + 95, 0xFFA0AEC0, 1);

        self.draw_rect(buffer, card_x + 20, card_y + 115, card_w - 40, 2, 0xFF2A3A52);

        self.draw_simple_text(buffer, "PRIMARY MISSION DIRECTIVE:", card_x + 20, card_y + 130, 0xFFFF4500, 2);
        self.draw_simple_text(buffer, "INFILTRATE ROGUE AI SECTOR AND DESTROY MAINFRAME", card_x + 20, card_y + 160, 0xFFE2E8F0, 1);
        self.draw_simple_text(buffer, "AUTONOMOUS THREATS DETECTED: HIGH DENSITY", card_x + 20, card_y + 180, 0xFFFF3344, 1);

        let progress = (timer / 2.5).clamp(0.0, 1.0);
        let bar_w = ((card_w - 40) as f32 * progress) as usize;
        self.draw_rect(buffer, card_x + 20, card_y + 215, card_w - 40, 12, 0xFF1A2233);
        self.draw_rect(buffer, card_x + 20, card_y + 215, bar_w, 12, 0xFF00FF77);
        self.draw_simple_text(buffer, "COMMANDO DEPLOYMENT: READY", card_x + 20, card_y + 240, 0xFF00FF77, 2);

        let p_anim_x = (-40.0 + (progress * 200.0)).min(160.0);
        for p in players.iter() {
            let mut clone_p = p.clone();
            clone_p.x = p_anim_x;
            clone_p.jetpack_active = true;
            self.draw_modern_commando(buffer, &clone_p);
        }
    }

    /// Escena cinemática de victoria de fase (Stage Clear) estilo Final Mission
    pub fn render_stage_clear_scene(
        &self,
        buffer: &mut [u32],
        stage_num: u8,
        stage_name: &str,
        timer: f32,
        score: u32,
        health: f32,
        bombs: u8,
        players: &[Player],
    ) {
        let w = self.width as isize;
        let h = self.height as isize;

        buffer.fill(0xFF050811);
        let warp_stretch = (timer * 80.0) as isize;
        for s in self.stars.iter() {
            let sx = (s.x as isize + warp_stretch) % w;
            let sy = s.y as isize;
            self.draw_rect(buffer, sx, sy, 22, 2, 0xFF00E5FF);
        }

        let cx = w / 2;
        let cy = h / 2;
        let card_w = 580.min(w as usize - 40);
        let card_h = 240;
        let bx = cx - (card_w as isize) / 2;
        let by = cy - (card_h as isize) / 2;

        self.draw_rect(buffer, bx, by, card_w, card_h as usize, 0xDD0D1826);
        self.draw_rect(buffer, bx, by, card_w, 3, 0xFFFFD700);
        self.draw_rect(buffer, bx, by + card_h as isize - 3, card_w, 3, 0xFFFFD700);

        self.draw_simple_text(buffer, "STAGE CLEAR!", cx - 130, by + 20, 0xFFFFD700, 4);
        self.draw_simple_text(buffer, stage_name, cx - 150, by + 65, 0xFF00E5FF, 2);

        let bonus_stage = 10000 * stage_num as u32;
        let bonus_hp = (health as u32) * 50;
        let bonus_bombs = (bombs as u32) * 2000;
        let score_str = format!("TOTAL SCORE: {}", score);
        let bonus_str = format!("CLEAR BONUS: +{} PTS", bonus_stage);
        let armor_str = format!("ARMOR INTEGRITY BONUS: +{} PTS", bonus_hp);
        let bombs_str = format!("EMP BOMBS SAVED BONUS: +{} PTS", bonus_bombs);

        self.draw_simple_text(buffer, &bonus_str, bx + 40, by + 105, 0xFF00FF77, 2);
        self.draw_simple_text(buffer, &armor_str, bx + 40, by + 130, 0xFFE2E8F0, 1);
        self.draw_simple_text(buffer, &bombs_str, bx + 40, by + 150, 0xFFE2E8F0, 1);
        self.draw_simple_text(buffer, &score_str, bx + 40, by + 180, 0xFFFFD700, 2);

        let warp_x = 160.0 + (timer * 350.0);
        for p in players.iter() {
            let mut clone_p = p.clone();
            clone_p.x = warp_x;
            clone_p.jetpack_active = true;
            self.draw_modern_commando(buffer, &clone_p);
            let px = warp_x as isize;
            let py = clone_p.y as isize;
            self.draw_rect(buffer, 0, py - 4, px.max(0) as usize, 8, 0x4400E5FF);
            self.draw_rect(buffer, 0, py - 1, px.max(0) as usize, 2, 0xAAFFFFFF);
        }
    }

    /// Escena de Game Over estilo arcade (regresa al Menú Principal al tocar)
    pub fn render_game_over_scene(&self, buffer: &mut [u32], timer: f32, score: u32) {
        let w = self.width as isize;
        let h = self.height as isize;

        buffer.fill(0xFF140505);

        let cx = w / 2;
        let cy = h / 2;

        self.draw_rect(buffer, 0, cy - 80, w as usize, 160, 0xEE1C0707);
        self.draw_rect(buffer, 0, cy - 80, w as usize, 3, 0xFFFF0033);
        self.draw_rect(buffer, 0, cy + 80, w as usize, 3, 0xFFFF0033);

        self.draw_simple_text(buffer, "GAME OVER", cx - 130, cy - 50, 0xFFFF0033, 4);
        self.draw_simple_text(buffer, "CYBER-COMMANDO SYSTEM DESTROYED", cx - 160, cy + 5, 0xFFA0AEC0, 1);
        let score_str = format!("FINAL SCORE: {}", score);
        self.draw_simple_text(buffer, &score_str, cx - 100, cy + 25, 0xFFFFD700, 2);

        let pulse = ((timer * 4.0).sin().abs() * 255.0) as u32;
        let col = 0xFF000000 | (pulse << 16) | (pulse << 8) | pulse;
        self.draw_simple_text(buffer, "TOCA PARA IR AL MENU PRINCIPAL", cx - 180, cy + 55, col, 2);
    }

    /// Escena de Victoria Final tras vencer al Jefe 8 (regresa al Menú Principal al tocar)
    pub fn render_victory_scene(&self, buffer: &mut [u32], timer: f32, score: u32) {
        let w = self.width as isize;
        let h = self.height as isize;

        buffer.fill(0xFF05111A);

        let cx = w / 2;
        let cy = h / 2;

        self.draw_rect(buffer, 0, cy - 90, w as usize, 180, 0xEE092033);
        self.draw_rect(buffer, 0, cy - 90, w as usize, 3, 0xFF00FFCC);
        self.draw_rect(buffer, 0, cy + 90, w as usize, 3, 0xFF00FFCC);

        self.draw_simple_text(buffer, "VICTORY!", cx - 100, cy - 65, 0xFFFFD700, 4);
        self.draw_simple_text(buffer, "THE ROGUE AI HAS BEEN NEUTRALIZED", cx - 170, cy - 15, 0xFF00FFCC, 2);
        self.draw_simple_text(buffer, "EARTH AND THE SOLAR SYSTEM ARE SAVED", cx - 160, cy + 10, 0xFFE2E8F0, 1);
        let score_str = format!("LEGENDARY SCORE: {}", score);
        self.draw_simple_text(buffer, &score_str, cx - 130, cy + 30, 0xFFFFD700, 2);

        let pulse = ((timer * 4.0).sin().abs() * 255.0) as u32;
        let col = 0xFF000000 | (pulse << 16) | (pulse << 8) | pulse;
        self.draw_simple_text(buffer, "TOCA PARA IR AL MENU PRINCIPAL", cx - 180, cy + 65, col, 2);
    }

    fn get_char_glyph(ch: char) -> [u8; 7] {
        match ch.to_ascii_uppercase() {
            'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
            'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
            'G' => [0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            'J' => [0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110],
            'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001],
            'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
            'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
            'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
            'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
            'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
            'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
            'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
            '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
            '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
            '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
            '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
            '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
            '6' => [0b01110, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b01110],
            '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
            '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
            '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
            ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
            '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100, 0b00000],
            '>' => [0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000],
            '<' => [0b00001, 0b00010, 0b00100, 0b01000, 0b00100, 0b00010, 0b00001],
            '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
            ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
            '(' => [0b00110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b00110],
            ')' => [0b01100, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01100],
            '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
            '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
            '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
            '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
            '%' => [0b11001, 0b11010, 0b00100, 0b01000, 0b01011, 0b10011, 0b00000],
            _ => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        }
    }
}
