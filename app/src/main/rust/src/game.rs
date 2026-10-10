//! Bucle Principal del Juego (Master Game Loop) de IA Rebellion
//! Orquesta lógica de juego, colisiones, 8 niveles, 8 jefes, audio y multijugador

use std::sync::{Arc, Mutex};
use crate::audio::{AudioEngine, SoundEffect};
use crate::level::{LevelManager, StagePhase};
use crate::boss::Boss;
use crate::bullet::{Bullet, BulletOwner};
use crate::enemy::Enemy;
use crate::multiplayer::MultiplayerManager;
use crate::player::Player;
use crate::renderer::Renderer;
use crate::touch::TouchControls;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
    TitleMenu,
    StageIntro,
    InGame,
    BossBattle,
    StageClear,
    GameOver,
    Victory,
}

pub struct Game {
    pub state: GameState,
    pub state_timer: f32,
    pub width: usize,
    pub height: usize,

    pub players: Vec<Player>,
    pub enemies: Vec<Enemy>,
    pub boss: Option<Boss>,
    pub bullets: Vec<Bullet>,

    pub level_manager: LevelManager,
    pub touch_controls: TouchControls,
    pub renderer: Renderer,
    pub audio_engine: Arc<Mutex<AudioEngine>>,
    pub multiplayer: MultiplayerManager,

    pub total_score: u32,
    pub high_score: u32,
}

impl Game {
    pub fn new(w: usize, h: usize) -> Self {
        Self::new_with_audio(w, h, Arc::new(Mutex::new(AudioEngine::new())))
    }

    pub fn new_with_audio(w: usize, h: usize, audio_engine: Arc<Mutex<AudioEngine>>) -> Self {
        let w_f = w as f32;
        let h_f = h as f32;

        let p1 = Player::new(0, 160.0, h_f * 0.5);
        let players = vec![p1];

        Self {
            state: GameState::TitleMenu,
            state_timer: 0.0,
            width: w,
            height: h,
            players,
            enemies: Vec::with_capacity(64),
            boss: None,
            bullets: Vec::with_capacity(256),
            level_manager: LevelManager::new(1),
            touch_controls: TouchControls::new(w_f, h_f),
            renderer: Renderer::new(w, h),
            audio_engine,
            multiplayer: MultiplayerManager::new(true, 0),
            total_score: 0,
            high_score: 50000,
        }
    }

    #[inline(always)]
    pub fn play_sfx(&self, sfx: SoundEffect) {
        if let Ok(mut engine) = self.audio_engine.lock() {
            engine.play_sfx(sfx);
        }
    }

    #[inline(always)]
    pub fn trigger_sfx(audio: &Arc<Mutex<AudioEngine>>, sfx: SoundEffect) {
        if let Ok(mut engine) = audio.lock() {
            engine.play_sfx(sfx);
        }
    }

    #[inline(always)]
    pub fn set_stage_theme(&mut self, stage: u8) {
        if let Ok(mut engine) = self.audio_engine.lock() {
            engine.stage_theme = stage;
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.width = w;
        self.height = h;
        self.touch_controls.resize(w as f32, h as f32);
        self.renderer.resize(w, h);
    }

    /// Evento táctil: dedo presiona la pantalla (Jugador.java toquePresionado)
    pub fn on_touch_down(&mut self, id: i32, x: f32, y: f32) {
        if self.state == GameState::TitleMenu {
            self.start_game(1);
            return;
        }
        if (self.state == GameState::GameOver || self.state == GameState::Victory) && self.state_timer > 2.0 {
            self.start_game(1);
            return;
        }

        let local_id = self.multiplayer.local_player_id as usize;
        let (px, py) = if let Some(p) = self.players.get(local_id) {
            (p.x, p.y)
        } else {
            (160.0, 270.0)
        };

        self.touch_controls.on_touch_down(id, x, y, px, py);

        // Bomba especial EMP activada por doble toque rápido (< 0.35s)
        if self.touch_controls.trigger_bomb {
            if let Some(p) = self.players.get_mut(local_id) {
                if p.bombs > 0 {
                    p.bombs -= 1;
                    let bx = p.x;
                    let by = p.y;
                    self.trigger_bomb(bx, by);
                }
            }
        }
    }

    /// Evento táctil: dedo se desliza por la pantalla (Jugador.java toqueDeslizando)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) {
        let w_f = self.width as f32;
        let h_f = self.height as f32;
        let local_id = self.multiplayer.local_player_id as usize;

        if let Some((target_x, target_y)) = self.touch_controls.on_touch_move(id, x, y) {
            if let Some(p) = self.players.get_mut(local_id) {
                let old_x = p.x;
                let old_y = p.y;
                let (min_x, max_x, min_y, max_y) = self.level_manager.get_player_bounds(w_f, h_f);
                let new_x = target_x.clamp(min_x, max_x);
                let new_y = target_y.clamp(min_y, max_y);

                let dx = new_x - old_x;
                let dy = new_y - old_y;

                p.x = new_x;
                p.y = new_y;
                p.vx = dx * 60.0;
                p.vy = dy * 60.0;
                p.jetpack_active = dx.abs() > 0.15 || dy.abs() > 0.15;

                // Gesto de Flick horizontal rápido para voltear:
                if let Some(flick_right) = self.touch_controls.flick_facing.take() {
                    p.facing_right = flick_right;
                }
            }
        }
    }

    /// Evento táctil: dedo se levanta (Jugador.java toqueLevantado)
    pub fn on_touch_up(&mut self, id: i32, x: f32, y: f32) {
        let local_id = self.multiplayer.local_player_id as usize;
        self.touch_controls.on_touch_up(id, x, y);

        // Multi-touch: Toque rápido con segundo dedo conmuta orientación (Volteo 180° estilo Final Mission)
        if self.touch_controls.toggle_facing {
            self.touch_controls.toggle_facing = false;
            if let Some(p) = self.players.get_mut(local_id) {
                p.toggle_facing();
            }
        }

        // Si se promovió el segundo dedo a primario, inicializar delta con la posición actual
        if self.touch_controls.primary_id != -1 && !self.touch_controls.touch_initialized {
            if let Some(p) = self.players.get(local_id) {
                self.touch_controls.delta_x_tactil = self.touch_controls.primary_x - p.x;
                self.touch_controls.delta_y_tactil = self.touch_controls.primary_y - p.y;
                self.touch_controls.touch_initialized = true;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let w_f = self.width as f32;
        let h_f = self.height as f32;
        self.state_timer += dt;

        // 1. Actualizar controles táctiles y efectos de fondo
        self.touch_controls.update(dt);
        self.renderer.update_fx(dt);

        match self.state {
            GameState::TitleMenu => {
                // Solo inicia cuando el usuario presiona la pantalla explícitamente
                if self.touch_controls.is_touching {
                    self.start_game(1);
                }
            }
            GameState::StageIntro => {
                if self.state_timer > 2.5 {
                    self.state = GameState::InGame;
                    self.state_timer = 0.0;
                }
            }
            GameState::InGame => {
                // Actualizar nivel y verificar si debe entrar el Boss
                let trigger_boss = self.level_manager.update(dt, w_f, h_f, &mut self.enemies);
                if trigger_boss {
                    self.state = GameState::BossBattle;
                    self.boss = Some(Boss::new_for_stage(self.level_manager.current_stage, w_f, h_f));
                    self.play_sfx(SoundEffect::BossAlarm);
                }

                self.update_gameplay(dt, w_f, h_f);
            }
            GameState::BossBattle => {
                let p_x = self.players.first().map(|p| p.x).unwrap_or(w_f * 0.5);
                let p_y = self.players.first().map(|p| p.y).unwrap_or(h_f * 0.5);

                if let Some(b) = self.boss.as_mut() {
                    b.update(dt, p_x, p_y, w_f, h_f, &mut self.bullets);
                    if b.defeated {
                        self.renderer.add_explosion(b.x, b.y, 40, 0xFFFF4500);
                        self.play_sfx(SoundEffect::Explosion);
                        self.total_score += 10000 * (self.level_manager.current_stage as u32);

                        if self.level_manager.current_stage >= 8 {
                            self.state = GameState::Victory;
                        } else {
                            self.state = GameState::StageClear;
                        }
                        self.state_timer = 0.0;
                    }
                }

                self.update_gameplay(dt, w_f, h_f);
            }
            GameState::StageClear => {
                if self.state_timer > 3.5 {
                    let next_stage = self.level_manager.current_stage + 1;
                    self.start_game(next_stage);
                }
            }
            GameState::GameOver => {
                if self.state_timer > 2.0 && self.touch_controls.is_touching {
                    self.start_game(1);
                }
            }
            GameState::Victory => {
                if self.state_timer > 4.0 && self.touch_controls.is_touching {
                    self.start_game(1);
                }
            }
        }
    }

    fn start_game(&mut self, stage: u8) {
        let _w_f = self.width as f32;
        let h_f = self.height as f32;
        self.level_manager.set_stage(stage);
        self.set_stage_theme(stage);

        // Reinicio completo y limpio del audio para evitar acumulación y lentitud
        if let Ok(mut engine) = self.audio_engine.lock() {
            engine.reset_for_new_game(stage);
        }

        self.renderer.scroll_x = 0.0;
        self.renderer.scroll_y = 0.0;
        self.renderer.scroll_vx = self.level_manager.config.scroll_speed;
        self.renderer.scroll_vy = 0.0;
        self.renderer.stage_phase = self.level_manager.current_phase;
        self.renderer.stage_progress = 0.0;
        self.state = GameState::StageIntro;
        self.state_timer = 0.0;
        self.boss = None;
        self.enemies.clear();
        self.bullets.clear();

        for p in self.players.iter_mut() {
            p.x = 160.0;
            p.y = h_f * 0.5;
            p.health = p.max_health;
            p.active = true;
            p.invulnerable_timer = 2.0;
            p.facing_right = true; // Inicia siempre mirando al frente hacia la derecha
        }

        self.touch_controls.toggle_facing = false;
        self.touch_controls.flick_facing = None;

        // Reinicializar desplazamiento relativo táctil con la posición inicial
        let local_id = self.multiplayer.local_player_id as usize;
        if self.touch_controls.primary_id != -1 {
            if let Some(p) = self.players.get(local_id) {
                self.touch_controls.delta_x_tactil = self.touch_controls.primary_x - p.x;
                self.touch_controls.delta_y_tactil = self.touch_controls.primary_y - p.y;
                self.touch_controls.touch_initialized = true;
            }
        }
    }

    fn update_gameplay(&mut self, dt: f32, w_f: f32, h_f: f32) {
        let local_id = self.multiplayer.local_player_id as usize;
        let is_firing = self.touch_controls.is_firing;
        let sat_lock = self.touch_controls.satellite_lock;
        let sat_angle = self.touch_controls.satellite_target_angle;
        let trigger_bomb = self.touch_controls.trigger_bomb;

        // Conmutación de orientación si quedó pendiente
        if self.touch_controls.toggle_facing {
            self.touch_controls.toggle_facing = false;
            if let Some(p) = self.players.get_mut(local_id) {
                p.toggle_facing();
            }
        }

        // 1. Transmitir controles vía Bluetooth (Vector normalizado de posición)
        if let Some(p) = self.players.get(local_id) {
            let norm_mx = (p.x / w_f * 2.0 - 1.0).clamp(-1.0, 1.0);
            let norm_my = (p.y / h_f * 2.0 - 1.0).clamp(-1.0, 1.0);
            self.multiplayer.encode_input_packet(
                norm_mx,
                norm_my,
                is_firing,
                trigger_bomb,
                sat_lock,
            );
        }

        // 2. Si es anfitrión, emitir sincronización de estado periódica (20 Hz)
        if self.multiplayer.is_host {
            self.multiplayer.sync_timer += dt;
            if self.multiplayer.sync_timer >= 0.05 {
                self.multiplayer.sync_timer = 0.0;
                for p in self.players.iter() {
                    if p.active {
                        self.multiplayer.encode_player_sync(
                            p.id,
                            p.x,
                            p.y,
                            p.health,
                            p.score,
                            p.weapon as u8,
                        );
                    }
                }
            }
        }

        let audio = self.audio_engine.clone();

        // 3. Actualizar satélites orbitales y auto-disparo continuo de Jugador.java
        let mut bomb_trigger_pos = None;
        if let Some(p_local) = self.players.get_mut(local_id) {
            p_local.update(dt, sat_lock, sat_angle);

            if is_firing {
                let bullets_before = self.bullets.len();
                p_local.fire(&mut self.bullets);
                if self.bullets.len() > bullets_before {
                    Self::trigger_sfx(&audio, SoundEffect::Laser);
                }
            }

            // Bomba especial EMP
            if trigger_bomb && p_local.bombs > 0 {
                p_local.bombs -= 1;
                bomb_trigger_pos = Some((p_local.x, p_local.y));
            }
        }

        if let Some((bx, by)) = bomb_trigger_pos {
            self.trigger_bomb(bx, by);
        }

        // 4. Actualizar proyectiles
        let p_x = self.players.first().map(|p| p.x).unwrap_or(0.0);
        let p_y = self.players.first().map(|p| p.y).unwrap_or(0.0);
        for b in self.bullets.iter_mut() {
            b.update(dt, p_x, p_y);
        }
        self.bullets.retain(|b| b.active);

        // Mantener a los jugadores dentro de los límites dinámicos de la fase activa
        let (min_x, max_x, min_y, max_y) = self.level_manager.get_player_bounds(w_f, h_f);
        for p in self.players.iter_mut() {
            if p.active {
                p.x = p.x.clamp(min_x, max_x);
                p.y = p.y.clamp(min_y, max_y);
            }
        }

        // 5. Actualizar enemigos comunes con arrastre vertical de cámara relativo
        let scroll_vy = self.level_manager.scroll_vy;
        let current_phase = self.level_manager.current_phase;

        for e in self.enemies.iter_mut() {
            if current_phase == StagePhase::AscendUp || current_phase == StagePhase::DescendDown {
                e.y -= scroll_vy * dt;
                e.x -= e.vx * dt;
            }
            e.update(dt, p_x, p_y, &mut self.bullets);
        }
        self.enemies.retain(|e| e.active && e.x >= -90.0 && e.x <= w_f + 90.0 && e.y >= -90.0 && e.y <= h_f + 90.0);

        // 6. Detectar colisiones
        self.check_collisions();
    }

    fn trigger_bomb(&mut self, bx: f32, by: f32) {
        self.renderer.add_explosion(bx, by, 60, 0xFF00FFFF);
        self.play_sfx(SoundEffect::BombExplosion);
        self.play_sfx(SoundEffect::EmpShockwave);

        // Elimina proyectiles enemigos
        for b in self.bullets.iter_mut() {
            if b.owner == BulletOwner::Enemy || b.owner == BulletOwner::Boss {
                b.active = false;
            }
        }

        // Daña a todos los enemigos en pantalla
        for e in self.enemies.iter_mut() {
            e.health -= 300.0;
            if e.health <= 0.0 {
                e.active = false;
                self.total_score += 150;
            }
        }

        // Daña al jefe si está activo
        if let Some(b) = self.boss.as_mut() {
            b.take_damage(500.0);
        }
    }

    fn check_collisions(&mut self) {
        let audio = self.audio_engine.clone();

        // Colisión: Proyectiles del jugador vs Enemigos
        for b in self.bullets.iter_mut() {
            if !b.active {
                continue;
            }

            match b.owner {
                BulletOwner::Player(_) | BulletOwner::Satellite(_) => {
                    // Contra enemigos comunes
                    for e in self.enemies.iter_mut() {
                        if !e.active {
                            continue;
                        }
                        let dist = ((b.x - e.x).powi(2) + (b.y - e.y).powi(2)).sqrt();
                        if dist < (b.radius + e.radius) {
                            b.active = false;
                            e.health -= b.damage;
                            self.renderer.add_explosion(b.x, b.y, 4, 0xFFFFFF00);

                            if e.health <= 0.0 {
                                e.active = false;
                                self.total_score += 200;
                                self.renderer.add_explosion(e.x, e.y, 14, 0xFFFF4500);
                                Self::trigger_sfx(&audio, SoundEffect::Explosion);
                            }
                            break;
                        }
                    }

                    // Contra el Boss
                    if let Some(boss) = self.boss.as_mut() {
                        if boss.active && !boss.defeated {
                            let dist = ((b.x - boss.x).powi(2) + (b.y - boss.y).powi(2)).sqrt();
                            if dist < (b.radius + boss.radius) {
                                b.active = false;
                                boss.take_damage(b.damage);
                                self.total_score += 50;
                                self.renderer.add_explosion(b.x, b.y, 6, 0xFFFF0055);
                            }
                        }
                    }
                }
                BulletOwner::Enemy | BulletOwner::Boss => {
                    // Contra Jugador o sus Satélites
                    for p in self.players.iter_mut() {
                        if !p.active || p.invulnerable_timer > 0.0 {
                            continue;
                        }

                        // Satélites orbitales destruyen balas enemigas (Mecánica Final Mission)
                        let mut blocked_by_sat = false;
                        for sat in p.satellites.iter() {
                            let sx = p.x + sat.angle.cos() * sat.distance;
                            let sy = p.y + sat.angle.sin() * sat.distance;
                            let dist_sat = ((b.x - sx).powi(2) + (b.y - sy).powi(2)).sqrt();
                            if dist_sat < (b.radius + 12.0) {
                                b.active = false;
                                blocked_by_sat = true;
                                self.renderer.add_explosion(sx, sy, 5, 0xFF00FFFF);
                                Self::trigger_sfx(&audio, SoundEffect::Ricochet);
                                break;
                            }
                        }

                        if blocked_by_sat {
                            break;
                        }

                        // Impacto en el soldado del jugador
                        let dist = ((b.x - p.x).powi(2) + (b.y - p.y).powi(2)).sqrt();
                        if dist < (b.radius + 18.0) {
                            b.active = false;
                            let died = p.take_damage(b.damage);
                            Self::trigger_sfx(&audio, SoundEffect::PlayerHit);
                            self.renderer.add_explosion(p.x, p.y, 18, 0xFFFF0000);

                            if died && p.lives <= 0 {
                                self.state = GameState::GameOver;
                                self.state_timer = 0.0;
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn render(&mut self, buffer: &mut [u32]) {
        match self.state {
            GameState::TitleMenu => {
                self.renderer.render_splash_screen(buffer, self.state_timer);
            }
            GameState::StageIntro => {
                self.renderer.render_stage_intro_scene(
                    buffer,
                    self.level_manager.current_stage,
                    &self.level_manager.config.name,
                    &self.level_manager.config.subtitle,
                    self.state_timer,
                    &self.players,
                );
            }
            GameState::StageClear => {
                let (hp, bombs) = self.players.first().map(|p| (p.health, p.bombs)).unwrap_or((100.0, 3));
                self.renderer.render_stage_clear_scene(
                    buffer,
                    self.level_manager.current_stage,
                    &self.level_manager.config.name,
                    self.state_timer,
                    self.total_score,
                    hp,
                    bombs,
                    &self.players,
                );
            }
            GameState::GameOver => {
                self.renderer.render_game_over_scene(buffer, self.state_timer, self.total_score);
            }
            GameState::Victory => {
                self.renderer.render_victory_scene(buffer, self.state_timer, self.total_score);
            }
            GameState::InGame | GameState::BossBattle => {
                let bg = self.level_manager.config.bg_color;
                let s_num = self.level_manager.current_stage;
                let s_name = self.level_manager.config.name.clone();

                self.renderer.stage_progress = self.level_manager.stage_progress;
                self.renderer.scroll_x = self.level_manager.scroll_x;
                self.renderer.scroll_y = self.level_manager.scroll_y;
                self.renderer.scroll_vx = self.level_manager.scroll_vx;
                self.renderer.scroll_vy = self.level_manager.scroll_vy;
                self.renderer.stage_phase = self.level_manager.current_phase;

                self.renderer.render_frame(
                    buffer,
                    bg,
                    &self.players,
                    &self.enemies,
                    &self.boss,
                    &self.bullets,
                    s_num,
                    &s_name,
                    self.level_manager.transition_timer,
                    self.level_manager.transition_text,
                );
            }
        }
    }

    pub fn process_bluetooth_data(&mut self, data: &[u8]) {
        let w_f = self.width as f32;
        let h_f = self.height as f32;
        self.multiplayer.process_incoming(
            data,
            &mut self.players,
            &mut self.bullets,
            w_f,
            h_f,
        );

        if let Some(pid) = self.multiplayer.trigger_bomb_request.take() {
            if let Some(p) = self.players.get(pid as usize) {
                let bx = p.x;
                let by = p.y;
                self.trigger_bomb(bx, by);
            }
        }
    }

    pub fn get_bluetooth_outgoing(&mut self) -> Vec<u8> {
        self.multiplayer.take_outgoing_bytes()
    }

    pub fn set_multiplayer(&mut self, is_host: bool, player_id: u8) {
        let pid = player_id.min(3);
        self.multiplayer.set_mode(is_host, pid);

        let h_f = self.height as f32;
        while self.players.len() <= (pid as usize) {
            let next_id = self.players.len() as u8;
            let py = (100.0 + (next_id as f32) * 80.0).min(h_f - 100.0);
            self.players.push(Player::new(next_id, 160.0, py));
        }
    }
}
