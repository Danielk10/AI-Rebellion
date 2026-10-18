//! Bucle Principal del Juego (Master Game Loop) de IA Rebellion
//! Orquesta lógica de juego, colisiones, 8 niveles, 8 jefes, audio y multijugador

use std::sync::{Arc, Mutex};
use crate::audio::{AudioEngine, SoundEffect};
use crate::level::{LevelManager, StagePhase};
use crate::boss::Boss;
use crate::bullet::{Bullet, BulletOwner};
use crate::enemy::{Enemy, EnemyType, Item, ItemType};
use crate::multiplayer::MultiplayerManager;
use crate::player::{Player, WeaponType};
use crate::renderer::Renderer;
use crate::touch::TouchControls;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
    SplashScreen,
    TitleMenu,
    MultiplayerMenu,
    SettingsMenu,
    StageIntro,
    InGame,
    BossBattle,
    Paused,
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
    pub items: Vec<Item>,
    pub boss: Option<Boss>,
    pub bullets: Vec<Bullet>,

    pub level_manager: LevelManager,
    pub touch_controls: TouchControls,
    pub renderer: Renderer,
    pub audio_engine: Arc<Mutex<AudioEngine>>,
    pub multiplayer: MultiplayerManager,

    pub total_score: u32,
    pub high_score: u32,
    pub paused_from_boss: bool,
    pub settings_from_pause: bool,
    pub squad_size: u8,
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
            items: Vec::with_capacity(32),
            boss: None,
            bullets: Vec::with_capacity(256),
            level_manager: LevelManager::new(1),
            touch_controls: TouchControls::new(w_f, h_f),
            renderer: Renderer::new(w, h),
            audio_engine,
            multiplayer: MultiplayerManager::new(true, 0),
            total_score: 0,
            high_score: 50000,
            paused_from_boss: false,
            settings_from_pause: false,
            squad_size: 1,
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

    pub fn get_audio_flags(&self) -> (bool, bool) {
        if let Ok(engine) = self.audio_engine.lock() {
            (engine.music_enabled, engine.sfx_enabled)
        } else {
            (true, true)
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.width = w;
        self.height = h;
        self.touch_controls.resize(w as f32, h as f32);
        self.renderer.resize(w, h);
    }

    /// Reinicia completamente una nueva campaña desde el Nivel 1 con todas las vidas, bombas y armas restauradas
    pub fn start_new_campaign(&mut self) {
        let h_f = self.height as f32;
        if self.total_score > self.high_score {
            self.high_score = self.total_score;
        }
        self.total_score = 0;

        let count = (self.multiplayer.local_player_id + 1).clamp(1, 4) as usize;
        self.players.clear();
        for id in 0..count {
            let py = (h_f * 0.5 + (id as f32) * 52.0).clamp(90.0, h_f - 90.0);
            let px = 150.0 - (id as f32) * 22.0;
            let mut p = Player::new(id as u8, px, py);
            p.lives = 3;
            p.bombs = 3;
            p.health = p.max_health;
            p.weapon = WeaponType::Vulcan;
            p.weapon_power = 1;
            p.score = 0;
            p.active = id == (self.multiplayer.local_player_id as usize);
            p.facing_right = true;
            self.players.push(p);
        }

        self.start_game(1);
    }

    /// Evento táctil: dedo presiona la pantalla (Jugador.java toquePresionado)
    pub fn on_touch_down(&mut self, id: i32, x: f32, y: f32) {
        let w_f = self.width as f32;
        let h_f = self.height as f32;
        let cx = w_f * 0.5;
        let cy = h_f * 0.5;

        match self.state {
            GameState::SplashScreen => {
                self.play_sfx(SoundEffect::SatelliteLock);
                self.state = GameState::TitleMenu;
                self.state_timer = 0.0;
                return;
            }
            GameState::TitleMenu => {
                // Solo se activan las opciones al tocar directamente el botón correspondiente
                let cy_menu = cy - 40.0;
                let btn_w = 300.0;
                let btn_x0 = cx - btn_w * 0.5;
                let btn_x1 = cx + btn_w * 0.5;

                let y_start = cy_menu + 84.0;
                let y_multi = cy_menu + 132.0;
                let y_settings = cy_menu + 180.0;

                if x >= btn_x0 && x <= btn_x1 {
                    if y >= y_start && y <= y_start + 40.0 {
                        self.play_sfx(SoundEffect::PowerUp);
                        self.start_new_campaign();
                        return;
                    } else if y >= y_multi && y <= y_multi + 40.0 {
                        self.play_sfx(SoundEffect::SatelliteLock);
                        self.state = GameState::MultiplayerMenu;
                        self.state_timer = 0.0;
                        return;
                    } else if y >= y_settings && y <= y_settings + 40.0 {
                        self.play_sfx(SoundEffect::SatelliteLock);
                        self.settings_from_pause = false;
                        self.state = GameState::SettingsMenu;
                        self.state_timer = 0.0;
                        return;
                    }
                }
                return;
            }
            GameState::MultiplayerMenu => {
                let panel_w = 460.0;
                let px0 = cx - panel_w * 0.5;
                let px1 = cx + panel_w * 0.5;

                // Fila 1: Rol Bluetooth (CREAR PARTIDA HOST vs UNIRSE CLIENTE BT)
                let role_y = cy - 75.0;
                if y >= role_y && y <= role_y + 42.0 && x >= px0 && x <= px1 {
                    let is_host = x < cx;
                    let new_pid = if is_host { 0 } else { self.multiplayer.local_player_id.max(1) };
                    self.set_multiplayer(is_host, new_pid);
                    self.play_sfx(SoundEffect::SatelliteLock);
                    return;
                }

                // Fila 2: Selección de Jugador 1 a 4 (ARNOLD, SIGOURNEY, JAX, ORION)
                let slot_y = cy - 12.0;
                if y >= slot_y && y <= slot_y + 54.0 && x >= px0 && x <= px1 {
                    let rel_x = (x - px0) / panel_w;
                    let pid = ((rel_x * 4.0) as u8).min(3);
                    let is_host = pid == 0;
                    self.set_multiplayer(is_host, pid);
                    self.play_sfx(SoundEffect::PowerUp);
                    return;
                }

                // Fila 3: Botones de Acción (INICIAR PARTIDA / VOLVER)
                let bot_y = cy + 65.0;
                if y >= bot_y && y <= bot_y + 44.0 && x >= px0 && x <= px1 {
                    if x < cx {
                        self.play_sfx(SoundEffect::PowerUp);
                        self.start_new_campaign();
                    } else {
                        self.play_sfx(SoundEffect::Ricochet);
                        self.state = GameState::TitleMenu;
                        self.state_timer = 0.0;
                    }
                }
                return;
            }
            GameState::SettingsMenu => {
                let btn_w = 320.0;
                let bx0 = cx - btn_w * 0.5;
                let bx1 = cx + btn_w * 0.5;

                let y_music = cy - 54.0;
                let y_sfx = cy + 2.0;
                let y_back = cy + 58.0;

                if x >= bx0 && x <= bx1 {
                    if y >= y_music && y <= y_music + 44.0 {
                        if let Ok(mut engine) = self.audio_engine.lock() {
                            engine.music_enabled = !engine.music_enabled;
                        }
                        self.play_sfx(SoundEffect::SatelliteLock);
                        return;
                    } else if y >= y_sfx && y <= y_sfx + 44.0 {
                        if let Ok(mut engine) = self.audio_engine.lock() {
                            engine.sfx_enabled = !engine.sfx_enabled;
                        }
                        self.play_sfx(SoundEffect::SatelliteLock);
                        return;
                    } else if y >= y_back && y <= y_back + 44.0 {
                        self.play_sfx(SoundEffect::Ricochet);
                        self.state = if self.settings_from_pause {
                            GameState::Paused
                        } else {
                            GameState::TitleMenu
                        };
                        self.state_timer = 0.0;
                        return;
                    }
                }
                return;
            }
            GameState::Paused => {
                let btn_w = 280.0;
                let bx0 = cx - btn_w * 0.5;
                let bx1 = cx + btn_w * 0.5;

                let y_resume = cy - 58.0;
                let y_settings = cy - 2.0;
                let y_menu = cy + 54.0;

                // También permite reanudar tocando de nuevo el botón esquina superior derecha
                if x >= w_f - 78.0 && y <= 58.0 {
                    self.play_sfx(SoundEffect::SatelliteLock);
                    self.state = if self.paused_from_boss {
                        GameState::BossBattle
                    } else {
                        GameState::InGame
                    };
                    return;
                }

                if x >= bx0 && x <= bx1 {
                    if y >= y_resume && y <= y_resume + 42.0 {
                        self.play_sfx(SoundEffect::PowerUp);
                        self.state = if self.paused_from_boss {
                            GameState::BossBattle
                        } else {
                            GameState::InGame
                        };
                        return;
                    } else if y >= y_settings && y <= y_settings + 42.0 {
                        self.play_sfx(SoundEffect::SatelliteLock);
                        self.settings_from_pause = true;
                        self.state = GameState::SettingsMenu;
                        self.state_timer = 0.0;
                        return;
                    } else if y >= y_menu && y <= y_menu + 42.0 {
                        self.play_sfx(SoundEffect::Ricochet);
                        self.touch_controls.primary_id = -1;
                        self.touch_controls.secondary_id = -1;
                        self.touch_controls.is_touching = false;
                        self.touch_controls.is_firing = false;
                        self.state = GameState::TitleMenu;
                        self.state_timer = 0.0;
                        return;
                    }
                }
                return;
            }
            GameState::GameOver | GameState::Victory => {
                // Al terminar Game Over o Victoria, regresar al Menú Principal
                if self.state_timer > 1.5 {
                    self.play_sfx(SoundEffect::SatelliteLock);
                    self.touch_controls.primary_id = -1;
                    self.touch_controls.secondary_id = -1;
                    self.touch_controls.is_touching = false;
                    self.touch_controls.is_firing = false;
                    self.state = GameState::TitleMenu;
                    self.state_timer = 0.0;
                }
                return;
            }
            GameState::InGame | GameState::BossBattle => {
                // Botón virtual de Pausa en la esquina superior derecha [ || ]
                if x >= w_f - 78.0 && y <= 58.0 {
                    self.paused_from_boss = self.state == GameState::BossBattle;
                    self.state = GameState::Paused;
                    self.state_timer = 0.0;
                    self.touch_controls.primary_id = -1;
                    self.touch_controls.secondary_id = -1;
                    self.touch_controls.is_touching = false;
                    self.touch_controls.is_firing = false;
                    self.play_sfx(SoundEffect::SatelliteLock);
                    return;
                }
            }
            GameState::StageIntro | GameState::StageClear => {}
        }

        let local_id = self.multiplayer.local_player_id as usize;
        let (px, py) = if let Some(p) = self.players.get(local_id) {
            (p.x, p.y)
        } else {
            (160.0, 270.0)
        };

        self.touch_controls.on_touch_down(id, x, y, px, py);
    }

    /// Evento táctil: dedo se desliza por la pantalla (Jugador.java toqueDeslizando)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) {
        if self.state != GameState::InGame && self.state != GameState::BossBattle {
            return;
        }

        let w_f = self.width as f32;
        let h_f = self.height as f32;
        let local_id = self.multiplayer.local_player_id as usize;

        // El desplazamiento del personaje se realiza exclusivamente por el dedo primario
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
            }
        }

        // Gesto de Flick horizontal rápido ejecutado por el segundo dedo (dedo libre):
        if let Some(flick_right) = self.touch_controls.flick_facing.take() {
            if let Some(p) = self.players.get_mut(local_id) {
                p.facing_right = flick_right;
            }
        }
    }

    /// Evento táctil: dedo se levanta (Jugador.java toqueLevantado)
    pub fn on_touch_up(&mut self, id: i32, x: f32, y: f32) {
        let local_id = self.multiplayer.local_player_id as usize;
        self.touch_controls.on_touch_up(id, x, y);

        if self.state != GameState::InGame && self.state != GameState::BossBattle {
            return;
        }

        // Gesto de Flick horizontal ejecutado al levantar el dedo libre (Swipe & Release):
        if let Some(flick_right) = self.touch_controls.flick_facing.take() {
            if let Some(p) = self.players.get_mut(local_id) {
                p.facing_right = flick_right;
            }
        }

        // Bomba especial EMP activada exclusivamente por toques rápidos con el dedo libre
        if self.touch_controls.trigger_bomb {
            self.touch_controls.trigger_bomb = false;
            if let Some(p) = self.players.get_mut(local_id) {
                if p.bombs > 0 {
                    p.bombs -= 1;
                    let bx = p.x;
                    let by = p.y;
                    self.trigger_bomb(bx, by);
                }
            }
        }

        // Si se promovió el segundo dedo a primario (cambio de dedo de movimiento),
        // inicializar delta con la posición actual del jugador para evitar saltos
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
        if self.state != GameState::Paused {
            self.renderer.update_fx(dt);
        }

        match self.state {
            GameState::SplashScreen | GameState::TitleMenu | GameState::MultiplayerMenu | GameState::SettingsMenu | GameState::Paused => {
                // La navegación de menús se gestiona por eventos táctiles en on_touch_down
            }
            GameState::StageIntro => {
                if self.state_timer > 2.5 {
                    self.state = GameState::InGame;
                    self.state_timer = 0.0;
                }
            }
            GameState::InGame => {
                // Actualizar nivel de forma lineal y verificar si debe entrar el Boss
                let trigger_boss = self.level_manager.update(dt, w_f, h_f, &mut self.enemies);
                if trigger_boss {
                    self.state = GameState::BossBattle;
                    self.boss = Some(Boss::new_for_stage(self.level_manager.current_stage, w_f, h_f));
                    self.play_sfx(SoundEffect::BossAlarm);
                }

                self.update_gameplay(dt, w_f, h_f);
            }
            GameState::BossBattle => {
                let local_id = self.multiplayer.local_player_id as usize;
                let p_x = self.players.get(local_id).map(|p| p.x).unwrap_or(w_f * 0.5);
                let p_y = self.players.get(local_id).map(|p| p.y).unwrap_or(h_f * 0.5);

                if let Some(b) = self.boss.as_mut() {
                    b.update(dt, p_x, p_y, w_f, h_f, &mut self.bullets);
                    if b.defeated {
                        self.renderer.add_explosion(b.x, b.y, 40, 0xFFFF4500);
                        self.play_sfx(SoundEffect::Explosion);
                        self.total_score += 10000 * (self.level_manager.current_stage as u32);
                        if self.total_score > self.high_score {
                            self.high_score = self.total_score;
                        }

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
            GameState::GameOver | GameState::Victory => {
                // Espera toque del usuario en on_touch_down para volver al menú principal
            }
        }
    }

    fn start_game(&mut self, stage: u8) {
        let h_f = self.height as f32;
        self.level_manager.set_stage(stage);
        self.set_stage_theme(stage);

        // Reinicio limpio del sintetizador para el tema musical del nivel
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
        self.items.clear();
        self.bullets.clear();

        for (idx, p) in self.players.iter_mut().enumerate() {
            p.x = 150.0 - (idx as f32) * 22.0;
            p.y = (h_f * 0.28 + (idx as f32) * 68.0).clamp(90.0, h_f - 90.0);
            p.health = p.max_health;
            if p.lives <= 0 {
                p.lives = 3;
            }
            if p.bombs == 0 {
                p.bombs = 3;
            }
            p.active = true;
            p.invulnerable_timer = 2.0;
            p.facing_right = true;
        }

        self.touch_controls.toggle_facing = false;
        self.touch_controls.flick_facing = None;
        self.touch_controls.trigger_bomb = false;
        self.touch_controls.time_since_last_tap = 10.0;

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
        self.touch_controls.trigger_bomb = false;

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

        // 3. Actualizar jugador local y compañeros de escuadrón activos (hasta 4 jugadores)
        let mut bomb_trigger_pos = None;
        let (lead_x, lead_y, lead_facing) = self
            .players
            .get(local_id)
            .map(|p| (p.x, p.y, p.facing_right))
            .unwrap_or((160.0, h_f * 0.5, true));

        for (idx, p) in self.players.iter_mut().enumerate() {
            if !p.active {
                continue;
            }
            if idx == local_id {
                p.update(dt, sat_lock, sat_angle);

                if is_firing {
                    let bullets_before = self.bullets.len();
                    p.fire(&mut self.bullets);
                    if self.bullets.len() > bullets_before {
                        Self::trigger_sfx(&audio, SoundEffect::Laser);
                    }
                }

                if trigger_bomb && p.bombs > 0 {
                    p.bombs -= 1;
                    bomb_trigger_pos = Some((p.x, p.y));
                }
            } else {
                // Compañeros de escuadrón (P2-P4): siguen formación táctica lineal si no reciben paquete externo inmediato
                let offset_idx = idx as f32;
                let form_x = (lead_x - 36.0 * ((idx % 2) as f32 + 1.0)).clamp(60.0, w_f - 60.0);
                let form_y = (lead_y + if idx % 2 == 1 { -58.0 } else { 58.0 } * ((idx as f32 * 0.5).ceil()))
                    .clamp(75.0, h_f - 75.0);
                p.x += (form_x - p.x) * (3.5 * dt).min(1.0);
                p.y += (form_y - p.y) * (3.5 * dt).min(1.0);
                p.facing_right = lead_facing;
                p.jetpack_active = true;
                p.update(dt, sat_lock, sat_angle.map(|a| a + offset_idx * 0.8));
                if is_firing {
                    p.fire(&mut self.bullets);
                }
            }
        }

        if let Some((bx, by)) = bomb_trigger_pos {
            self.trigger_bomb(bx, by);
        }

        // 4. Actualizar proyectiles
        let p_x = self.players.get(local_id).map(|p| p.x).unwrap_or(lead_x);
        let p_y = self.players.get(local_id).map(|p| p.y).unwrap_or(lead_y);
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

        // 5. Actualizar enemigos comunes con desplazamiento lineal continuo 2D (Final Mission style)
        let scroll_vy = self.level_manager.scroll_vy;
        let current_phase = self.level_manager.current_phase;

        for e in self.enemies.iter_mut() {
            if current_phase == StagePhase::AscendUp || current_phase == StagePhase::DescendDown {
                e.y -= scroll_vy * dt;
                e.x -= e.vx * dt * 0.45;
            }
            e.update(dt, p_x, p_y, &mut self.bullets);
        }
        self.enemies.retain(|e| {
            e.active && e.x >= -110.0 && e.x <= w_f + 110.0 && e.y >= -110.0 && e.y <= h_f + 110.0
        });

        // 6. Actualizar cápsulas de items en pantalla
        for item in self.items.iter_mut() {
            item.update(dt);
        }
        self.items.retain(|item| item.active);

        // 7. Detectar colisiones (incluyendo colisiones físicas con estructuras del escenario)
        self.check_collisions(w_f, h_f);
    }

    fn trigger_bomb(&mut self, bx: f32, by: f32) {
        self.renderer.add_explosion(bx, by, 60, 0xFF00FFFF);
        self.play_sfx(SoundEffect::BombExplosion);
        self.play_sfx(SoundEffect::EmpShockwave);

        for b in self.bullets.iter_mut() {
            if b.owner == BulletOwner::Enemy || b.owner == BulletOwner::Boss {
                b.active = false;
            }
        }

        for e in self.enemies.iter_mut() {
            e.health -= 300.0;
            if e.health <= 0.0 {
                e.active = false;
                self.total_score += 150;
                if e.enemy_type == EnemyType::ItemCarrier {
                    self.items.push(Item::new(e.x, e.y, ItemType::Bomb));
                }
            }
        }

        for obs in self.level_manager.obstacles.iter_mut() {
            if obs.active && obs.destructible {
                obs.health -= 250.0;
                if obs.health <= 0.0 {
                    obs.active = false;
                    self.renderer.add_explosion(obs.x, obs.y, 18, 0xFFFF8800);
                    self.total_score += 250;
                }
            }
        }

        if let Some(b) = self.boss.as_mut() {
            b.take_damage(500.0);
        }
    }

    fn check_collisions(&mut self, w_f: f32, h_f: f32) {
        let audio = self.audio_engine.clone();

        // A. Colisión física Jugador vs Obstáculos / Escenario (Estilo Final Mission)
        let (min_x, max_x, min_y, max_y) = self.level_manager.get_player_bounds(w_f, h_f);
        let local_id = self.multiplayer.local_player_id as usize;

        for (p_idx, p) in self.players.iter_mut().enumerate() {
            if !p.active {
                continue;
            }
            let pr = 16.0;
            for obs in self.level_manager.obstacles.iter() {
                if !obs.active {
                    continue;
                }
                let half_w = obs.width * 0.5 + pr;
                let half_h = obs.height * 0.5 + pr;
                let dx = p.x - obs.x;
                let dy = p.y - obs.y;

                if dx.abs() < half_w && dy.abs() < half_h {
                    let overlap_x = half_w - dx.abs();
                    let overlap_y = half_h - dy.abs();

                    // Empujar al jugador fuera del obstáculo sólido
                    if overlap_x < overlap_y {
                        if dx >= 0.0 {
                            p.x += overlap_x;
                        } else {
                            p.x -= overlap_x;
                        }
                    } else if dy >= 0.0 {
                        p.y += overlap_y;
                    } else {
                        p.y -= overlap_y;
                    }

                    p.x = p.x.clamp(min_x, max_x);
                    p.y = p.y.clamp(min_y, max_y);

                    // Sincronizar delta táctil para que el dedo no arrastre dentro del muro
                    if p_idx == local_id && self.touch_controls.primary_id != -1 {
                        self.touch_controls.delta_x_tactil = self.touch_controls.primary_x - p.x;
                        self.touch_controls.delta_y_tactil = self.touch_controls.primary_y - p.y;
                    }

                    // Daño por impacto / aplastamiento contra estructuras del escenario
                    if p.invulnerable_timer <= 0.0 {
                        let died = p.take_damage(12.0);
                        p.invulnerable_timer = 0.65;
                        Self::trigger_sfx(&audio, SoundEffect::MetalClang);
                        self.renderer.add_explosion(p.x, p.y, 8, 0xFFFFAA00);
                        if died && p.lives <= 0 {
                            p.active = false;
                        }
                    }
                }
            }
        }

        // B. Colisión: Proyectiles vs Obstáculos del Escenario, Enemigos, Boss y Jugadores
        for b in self.bullets.iter_mut() {
            if !b.active {
                continue;
            }

            // Proyectiles chocan contra obstáculos físicos del escenario
            for obs in self.level_manager.obstacles.iter_mut() {
                if !obs.active {
                    continue;
                }
                let half_w = obs.width * 0.5 + b.radius;
                let half_h = obs.height * 0.5 + b.radius;
                if (b.x - obs.x).abs() < half_w && (b.y - obs.y).abs() < half_h {
                    b.active = false;
                    self.renderer.add_explosion(b.x, b.y, 4, 0xFF00E5FF);

                    if obs.destructible {
                        if matches!(b.owner, BulletOwner::Player(_) | BulletOwner::Satellite(_)) {
                            obs.health -= b.damage;
                            if obs.health <= 0.0 {
                                obs.active = false;
                                self.total_score += 300;
                                self.renderer.add_explosion(obs.x, obs.y, 18, 0xFFFF6600);
                                Self::trigger_sfx(&audio, SoundEffect::Explosion);
                            } else {
                                Self::trigger_sfx(&audio, SoundEffect::MetalClang);
                            }
                        }
                    } else {
                        Self::trigger_sfx(&audio, SoundEffect::Ricochet);
                    }
                    break;
                }
            }

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

                                if e.enemy_type == EnemyType::ItemCarrier {
                                    let item_choice =
                                        match ((self.total_score / 100) + (e.y as u32 / 30)) % 6 {
                                            0 => ItemType::WeaponLaser,
                                            1 => ItemType::WeaponSpread,
                                            2 => ItemType::WeaponHoming,
                                            3 => ItemType::WeaponVulcan,
                                            4 => ItemType::Bomb,
                                            _ => ItemType::Shield,
                                        };
                                    self.items.push(Item::new(e.x, e.y, item_choice));
                                }
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
                        let dist = ((b.x - p.x).powi(2) + (p.y - b.y).powi(2)).sqrt();
                        if dist < (b.radius + 18.0) {
                            b.active = false;
                            let died = p.take_damage(b.damage);
                            Self::trigger_sfx(&audio, SoundEffect::PlayerHit);
                            self.renderer.add_explosion(p.x, p.y, 18, 0xFFFF0000);

                            if died && p.lives <= 0 {
                                p.active = false;
                            }
                            break;
                        }
                    }
                }
            }
        }

        // C. Colisión física Jugador vs Enemigos (contacto directo como en Final Mission)
        for p in self.players.iter_mut() {
            if !p.active || p.invulnerable_timer > 0.0 {
                continue;
            }
            for e in self.enemies.iter_mut() {
                if !e.active {
                    continue;
                }
                let dist = ((p.x - e.x).powi(2) + (p.y - e.y).powi(2)).sqrt();
                if dist < (e.radius + 16.0) {
                    let died = p.take_damage(18.0);
                    e.health -= 40.0;
                    if e.health <= 0.0 {
                        e.active = false;
                        self.renderer.add_explosion(e.x, e.y, 14, 0xFFFF4500);
                    }
                    Self::trigger_sfx(&audio, SoundEffect::PlayerHit);
                    self.renderer.add_explosion(p.x, p.y, 14, 0xFFFF2200);
                    if died && p.lives <= 0 {
                        p.active = false;
                    }
                    break;
                }
            }
        }

        // Verificar si todos los miembros del escuadrón han caído -> GameOver
        let any_alive = self.players.iter().any(|p| p.active && p.lives > 0);
        if !any_alive && (self.state == GameState::InGame || self.state == GameState::BossBattle) {
            if self.total_score > self.high_score {
                self.high_score = self.total_score;
            }
            self.state = GameState::GameOver;
            self.state_timer = 0.0;
        }

        // D. Colisión: Jugador vs Ítems / Cápsulas de Suministro
        for p in self.players.iter_mut() {
            if !p.active {
                continue;
            }
            for item in self.items.iter_mut() {
                if !item.active {
                    continue;
                }
                let dist = ((p.x - item.x).powi(2) + (p.y - item.y).powi(2)).sqrt();
                if dist < (item.radius + 22.0) {
                    item.active = false;
                    self.renderer.add_explosion(item.x, item.y, 14, 0xFF00FFFF);
                    Self::trigger_sfx(&audio, SoundEffect::PowerUp);

                    match item.item_type {
                        ItemType::WeaponVulcan => {
                            p.weapon = WeaponType::Vulcan;
                            p.weapon_power = (p.weapon_power + 1).min(3);
                        }
                        ItemType::WeaponLaser => {
                            p.weapon = WeaponType::Laser;
                            p.weapon_power = (p.weapon_power + 1).min(3);
                        }
                        ItemType::WeaponSpread => {
                            p.weapon = WeaponType::Spread;
                            p.weapon_power = (p.weapon_power + 1).min(3);
                        }
                        ItemType::WeaponHoming => {
                            p.weapon = WeaponType::Homing;
                            p.weapon_power = (p.weapon_power + 1).min(3);
                        }
                        ItemType::Bomb => {
                            p.bombs = (p.bombs + 1).min(6);
                        }
                        ItemType::Shield => {
                            p.health = (p.health + 40.0).min(p.max_health);
                        }
                        ItemType::ExtraLife => {
                            p.lives = (p.lives + 1).min(9);
                        }
                    }
                    self.total_score += 500;
                }
            }
        }
    }

    pub fn render(&mut self, buffer: &mut [u32]) {
        match self.state {
            GameState::SplashScreen => {
                self.renderer.render_splash_screen(buffer, self.state_timer);
            }
            GameState::TitleMenu => {
                self.renderer.render_title_menu(
                    buffer,
                    self.state_timer,
                );
            }
            GameState::MultiplayerMenu => {
                self.renderer.render_multiplayer_menu(
                    buffer,
                    self.state_timer,
                    self.multiplayer.is_host,
                    self.multiplayer.local_player_id,
                    self.squad_size,
                    self.multiplayer.connected_players.iter().filter(|&&c| c).count() as u8,
                );
            }
            GameState::SettingsMenu => {
                let (music_on, sfx_on) = self.get_audio_flags();
                self.renderer.render_settings_menu(
                    buffer,
                    self.state_timer,
                    music_on,
                    sfx_on,
                );
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
                let (hp, bombs) = self
                    .players
                    .first()
                    .map(|p| (p.health, p.bombs))
                    .unwrap_or((100.0, 3));
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
                self.renderer
                    .render_game_over_scene(buffer, self.state_timer, self.total_score);
            }
            GameState::Victory => {
                self.renderer
                    .render_victory_scene(buffer, self.state_timer, self.total_score);
            }
            GameState::InGame | GameState::BossBattle | GameState::Paused => {
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
                    &self.items,
                    &self.boss,
                    &self.bullets,
                    &self.level_manager.obstacles,
                    self.level_manager.inside_chamber,
                    s_num,
                    &s_name,
                    self.level_manager.transition_timer,
                    self.level_manager.transition_text,
                );

                if self.state == GameState::Paused {
                    self.renderer.render_pause_menu(
                        buffer,
                        self.state_timer,
                        s_num,
                        self.total_score,
                    );
                }
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
