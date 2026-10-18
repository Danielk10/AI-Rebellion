//! Bucle Principal del Juego (Master Game Loop) de IA Rebellion
//! Orquesta lógica de juego, colisiones, 8 niveles, 8 jefes, audio y multijugador

use std::sync::{Arc, Mutex};
use crate::audio::{AudioEngine, SoundEffect};
use crate::level::{LevelManager, StagePhase};
use crate::boss::Boss;
use crate::bullet::{Bullet, BulletOwner, BulletType};
use crate::enemy::{Enemy, EnemyAnchor, EnemyType, Item, ItemType};
use crate::multiplayer::MultiplayerManager;
use crate::player::{Player, WeaponType};
use crate::renderer::Renderer;
use crate::touch::TouchControls;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameState {
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
    pub paused_from_boss: bool,
    pub settings_from_pause: bool,
}

impl Game {
    pub fn new_with_audio(w: usize, h: usize, audio_engine: Arc<Mutex<AudioEngine>>) -> Self {
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
            touch_controls: TouchControls::new(),
            renderer: Renderer::new(w, h),
            audio_engine,
            multiplayer: MultiplayerManager::new(true, 0),
            total_score: 0,
            paused_from_boss: false,
            settings_from_pause: false,
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
        self.renderer.resize(w, h);
    }

    /// Reinicia completamente una nueva campaña desde el Nivel 1 con todas las vidas, bombas y armas restauradas
    pub fn start_new_campaign(&mut self) {
        let h_f = self.height as f32;
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
            GameState::StageIntro => {
                // La pantalla de briefing no se quita con un solo toque accidental; requiere que transcurra el briefing
                if self.state_timer >= 2.5 {
                    self.state = GameState::InGame;
                    self.state_timer = 0.0;
                    self.play_sfx(SoundEffect::SatelliteLock);
                }
                return;
            }
            GameState::StageClear => {}
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
            GameState::TitleMenu | GameState::MultiplayerMenu | GameState::SettingsMenu | GameState::Paused => {
                // La navegación de menús se gestiona por eventos táctiles en on_touch_down
            }
            GameState::StageIntro => {
                if self.state_timer > 3.0 {
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
                    self.level_manager.transition_timer = 0.0; // Despejar de inmediato para no tapar la pantalla del jefe
                    self.level_manager.transition_text = "";
                    self.play_sfx(SoundEffect::BossAlarm);
                }

                self.update_gameplay(dt, w_f, h_f);
            }
            GameState::BossBattle => {
                self.level_manager.transition_timer = 0.0;
                self.level_manager.transition_text = "";
                let local_id = self.multiplayer.local_player_id as usize;
                let p_x = self.players.get(local_id).map(|p| p.x).unwrap_or(w_f * 0.5);
                let p_y = self.players.get(local_id).map(|p| p.y).unwrap_or(h_f * 0.5);

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
            p.reset_pose();
        }

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

        let p_x = self.players.get(local_id).map(|p| p.x).unwrap_or(lead_x);
        let p_y = self.players.get(local_id).map(|p| p.y).unwrap_or(lead_y);

        // Búsqueda táctica de objetivo prioritario para Satélites y Misiles Teledirigidos (Mecánica Final Mission):
        // Prioridad 1: Torretas antiaéreas en el suelo (Floor), techo (Ceiling) o paredes (LeftWall/RightWall)
        // Prioridad 2: Drones y enemigos comunes activos en pantalla
        // Prioridad 3: Boss activo
        let mut best_target: Option<(f32, f32)> = None;
        let mut min_weight_dist = f32::MAX;

        for e in self.enemies.iter() {
            if !e.active || e.x < -40.0 || e.x > w_f + 80.0 || e.y < -40.0 || e.y > h_f + 40.0 {
                continue;
            }
            let dx = e.x - p_x;
            let dy = e.y - p_y;
            let dist = (dx * dx + dy * dy).sqrt();

            // Ponderación de amenaza antiaérea:
            // Las torretas fijas ancladas a superficies reciben ponderación preferente (dist * 0.45)
            // para que los satélites las adquieran y destruyan con prioridad absoluta
            let weight = if e.is_anchored() {
                dist * 0.45
            } else {
                dist
            };

            if weight < min_weight_dist {
                min_weight_dist = weight;
                best_target = Some((e.x, e.y));
            }
        }

        if best_target.is_none() {
            if let Some(boss) = self.boss.as_ref() {
                if boss.active && !boss.defeated {
                    best_target = Some((boss.x, boss.y));
                }
            }
        }

        let auto_aim_angle = best_target.map(|(tx, ty)| (ty - p_y).atan2(tx - p_x));
        let effective_sat_angle = if sat_lock { sat_angle.or(auto_aim_angle) } else { None };

        for (idx, p) in self.players.iter_mut().enumerate() {
            if !p.active {
                continue;
            }
            if idx == local_id {
                p.update(dt, sat_lock, effective_sat_angle);

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
                p.update(dt, sat_lock, effective_sat_angle.map(|a| a + offset_idx * 0.4));
                if is_firing {
                    p.fire(&mut self.bullets);
                }
            }
        }

        if let Some((bx, by)) = bomb_trigger_pos {
            self.trigger_bomb(bx, by);
        }

        // 4. Actualizar proyectiles (los misiles del jugador buscan activamente el enemigo más cercano)
        let p_facing_dir = self.players.get(local_id).map(|p| if p.facing_right { 1.0 } else { -1.0 }).unwrap_or(1.0);
        for b in self.bullets.iter_mut() {
            match b.owner {
                BulletOwner::Player(_) | BulletOwner::Satellite(_) => {
                    if b.b_type == BulletType::HomingMissile {
                        let mut nearest_target: Option<(f32, f32)> = None;
                        let mut min_dist_sq = f32::MAX;

                        for e in self.enemies.iter() {
                            if !e.active || e.x < -20.0 || e.x > w_f + 60.0 || e.y < -20.0 || e.y > h_f + 20.0 {
                                continue;
                            }
                            let dx = e.x - b.x;
                            let dy = e.y - b.y;
                            let d2 = dx * dx + dy * dy;
                            let weight_d2 = if e.is_anchored() { d2 * 0.35 } else { d2 };
                            if weight_d2 < min_dist_sq {
                                min_dist_sq = weight_d2;
                                nearest_target = Some((e.x, e.y));
                            }
                        }

                        if let Some(boss) = self.boss.as_ref() {
                            if boss.active && !boss.defeated {
                                let dx = boss.x - b.x;
                                let dy = boss.y - b.y;
                                let d2 = dx * dx + dy * dy;
                                if d2 < min_dist_sq {
                                    nearest_target = Some((boss.x, boss.y));
                                }
                            }
                        }

                        let (tx, ty) = nearest_target.unwrap_or((b.x + p_facing_dir * 800.0, b.y));
                        b.update(dt, tx, ty);
                    } else {
                        b.update(dt, b.x + p_facing_dir * 500.0, b.y);
                    }
                }
                BulletOwner::Enemy | BulletOwner::Boss => {
                    b.update(dt, p_x, p_y);
                }
            }
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
        let scroll_vx = self.level_manager.scroll_vx;
        let scroll_vy = self.level_manager.scroll_vy;
        let current_phase = self.level_manager.current_phase;

        for e in self.enemies.iter_mut() {
            if !e.active {
                continue;
            }
            if e.is_anchored() {
                // Torretas ancladas a superficies se desplazan rígidamente con el scroll de cámara
                e.x -= scroll_vx * dt;
                e.y -= scroll_vy * dt;

                // Solo bloquear contra la superficie si esa componente de scroll no está en movimiento relativo
                match e.anchor {
                    EnemyAnchor::Ceiling => {
                        if scroll_vy.abs() < 5.0 {
                            e.y = 78.0;
                        }
                    }
                    EnemyAnchor::Floor => {
                        if scroll_vy.abs() < 5.0 {
                            e.y = h_f - 82.0;
                        }
                    }
                    EnemyAnchor::LeftWall => {
                        if scroll_vx.abs() < 5.0 {
                            e.x = 52.0;
                        }
                    }
                    EnemyAnchor::RightWall => {
                        if scroll_vx.abs() < 5.0 {
                            e.x = w_f - 52.0;
                        }
                    }
                    EnemyAnchor::FreeFlying => {}
                }
            } else {
                // Enemigos voladores libres acompañan el flujo del scroll
                if current_phase == StagePhase::AscendUp || current_phase == StagePhase::DescendDown {
                    e.y -= scroll_vy * dt * 0.40;
                }
            }
            e.update(dt, p_x, p_y, &mut self.bullets);
        }
        self.enemies.retain(|e| {
            e.active && e.x >= -110.0 && e.x <= w_f + 110.0 && e.y >= -110.0 && e.y <= h_f + 110.0
        });

        // 6. Actualizar cápsulas de items en pantalla estilo Final Mission y como antes
        for item in self.items.iter_mut() {
            item.update(dt);
        }
        self.items.retain(|item| item.active);

        // 7. Detectar colisiones (incluyendo colisiones físicas con estructuras del escenario)
        self.check_collisions(w_f, h_f);
    }

    fn create_carrier_drop(ex: f32, ey: f32, drop: Option<ItemType>) -> Item {
        let item_type = drop.unwrap_or(ItemType::WeaponVulcan);
        Item::new(ex, ey, item_type)
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
            e.hit_flash = 1.0;
            if e.health <= 0.0 {
                e.active = false;
                self.total_score += 150;
                if e.enemy_type == EnemyType::ItemCarrier {
                    self.items.push(Self::create_carrier_drop(e.x, e.y, e.item_drop));
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

                    // Estilo Final Mission NES: Los obstáculos bloquean físicamente el avance del jugador sin daño ni parpadeo
                }
            }
        }

        // B. Colisión: Proyectiles vs Enemigos, Boss, Obstáculos del Escenario y Jugadores
        for b in self.bullets.iter_mut() {
            if !b.active {
                continue;
            }

            match b.owner {
                BulletOwner::Player(_) | BulletOwner::Satellite(_) => {
                    // 1. Contra enemigos comunes (Prioridad de impacto para destruir antiaéreos antes de colisión ambiental)
                    for e in self.enemies.iter_mut() {
                        if !e.active {
                            continue;
                        }
                        let dist = ((b.x - e.x).powi(2) + (b.y - e.y).powi(2)).sqrt();
                        if dist < (b.radius + e.radius) {
                            if b.pierce_count > 0 {
                                b.pierce_count -= 1;
                            } else {
                                b.active = false;
                            }
                            e.health -= b.damage;
                            e.hit_flash = 1.0;
                            self.renderer.add_explosion(b.x, b.y, 4, 0xFFFFFF00);

                            if e.health <= 0.0 {
                                e.active = false;
                                self.total_score += 200;
                                self.renderer.add_explosion(e.x, e.y, 14, 0xFFFF4500);
                                Self::trigger_sfx(&audio, SoundEffect::Explosion);

                                if e.enemy_type == EnemyType::ItemCarrier {
                                    self.items.push(Self::create_carrier_drop(e.x, e.y, e.item_drop));
                                }
                            }
                            if !b.active {
                                break;
                            }
                        }
                    }

                    if !b.active {
                        continue;
                    }

                    // 2. Contra el Boss
                    if let Some(boss) = self.boss.as_mut() {
                        if boss.active && !boss.defeated {
                            let dist = ((b.x - boss.x).powi(2) + (b.y - boss.y).powi(2)).sqrt();
                            if dist < (b.radius + boss.radius) {
                                if b.pierce_count > 0 {
                                    b.pierce_count -= 1;
                                } else {
                                    b.active = false;
                                }
                                boss.take_damage(b.damage);
                                self.total_score += 50;
                                self.renderer.add_explosion(b.x, b.y, 6, 0xFFFF0055);
                            }
                        }
                    }

                    if !b.active {
                        continue;
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

            // 3. Proyectiles chocan contra obstáculos físicos del escenario (con penetración para láser)
            if b.active {
                for obs in self.level_manager.obstacles.iter_mut() {
                    if !obs.active {
                        continue;
                    }
                    let half_w = obs.width * 0.5 + b.radius;
                    let half_h = obs.height * 0.5 + b.radius;
                    if (b.x - obs.x).abs() < half_w && (b.y - obs.y).abs() < half_h {
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
                            if b.pierce_count > 0 {
                                b.pierce_count -= 1;
                                self.renderer.add_explosion(b.x, b.y, 4, 0xFF00E5FF);
                            } else {
                                b.active = false;
                                self.renderer.add_explosion(b.x, b.y, 4, 0xFF00E5FF);
                                break;
                            }
                        } else {
                            // Muro o estructura indestructible: ningún proyectil la atraviesa
                            Self::trigger_sfx(&audio, SoundEffect::Ricochet);
                            b.active = false;
                            self.renderer.add_explosion(b.x, b.y, 4, 0xFF00E5FF);
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
                    e.hit_flash = 1.0;
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
                            if p.weapon == WeaponType::Vulcan {
                                p.weapon_power = (p.weapon_power + 1).min(3);
                            } else {
                                p.weapon = WeaponType::Vulcan;
                                p.weapon_power = 1;
                            }
                        }
                        ItemType::WeaponLaser => {
                            if p.weapon == WeaponType::Laser {
                                p.weapon_power = (p.weapon_power + 1).min(3);
                            } else {
                                p.weapon = WeaponType::Laser;
                                p.weapon_power = 1;
                            }
                        }
                        ItemType::WeaponSpread => {
                            if p.weapon == WeaponType::Spread {
                                p.weapon_power = (p.weapon_power + 1).min(3);
                            } else {
                                p.weapon = WeaponType::Spread;
                                p.weapon_power = 1;
                            }
                        }
                        ItemType::WeaponHoming => {
                            if p.weapon == WeaponType::Homing {
                                p.weapon_power = (p.weapon_power + 1).min(3);
                            } else {
                                p.weapon = WeaponType::Homing;
                                p.weapon_power = 1;
                            }
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

        // E. Colisión física y anti-solapamiento: Enemigos vs Obstáculos del Escenario y Límites de Fase
        for e in self.enemies.iter_mut() {
            if !e.active {
                continue;
            }

            // Los enemigos voladores libres no deben atravesar ni quedar dentro de obstáculos sólidos
            if !e.is_anchored() {
                for obs in self.level_manager.obstacles.iter() {
                    if !obs.active {
                        continue;
                    }
                    let half_w = obs.width * 0.5 + e.radius;
                    let half_h = obs.height * 0.5 + e.radius;
                    let dx = e.x - obs.x;
                    let dy = e.y - obs.y;

                    if dx.abs() < half_w && dy.abs() < half_h {
                        let overlap_x = half_w - dx.abs();
                        let overlap_y = half_h - dy.abs();

                        if overlap_x < overlap_y {
                            if dx >= 0.0 {
                                e.x += overlap_x;
                            } else {
                                e.x -= overlap_x;
                            }
                            e.vx = -e.vx * 0.5;
                        } else {
                            if dy >= 0.0 {
                                e.y += overlap_y;
                            } else {
                                e.y -= overlap_y;
                            }
                            e.vy = -e.vy * 0.5;
                        }
                    }
                }

                // Respetar los muros y límites de la fase para que los drones no salgan del pozo vertical o cielo
                match self.level_manager.current_phase {
                    StagePhase::AscendUp | StagePhase::DescendDown => {
                        let min_shaft_x = 56.0 + e.radius;
                        let max_shaft_x = w_f - 56.0 - e.radius;
                        if e.x < min_shaft_x {
                            e.x = min_shaft_x;
                            e.vx = e.vx.abs().max(30.0);
                        } else if e.x > max_shaft_x {
                            e.x = max_shaft_x;
                            e.vx = -e.vx.abs().max(30.0);
                        }
                    }
                    StagePhase::HorizontalRight => {
                        let min_sky_y = 52.0 + e.radius;
                        let max_sky_y = h_f - 48.0 - e.radius;
                        if e.y < min_sky_y {
                            e.y = min_sky_y;
                            e.vy = e.vy.abs();
                        } else if e.y > max_sky_y {
                            e.y = max_sky_y;
                            e.vy = -e.vy.abs();
                        }
                    }
                    StagePhase::BossEncounter => {}
                }
            }
        }
    }

    pub fn render(&mut self, buffer: &mut [u32]) {
        match self.state {
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
