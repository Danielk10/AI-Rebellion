//! Módulo de Gestión de los 8 Niveles de IA Rebellion
//! Controla el desplazamiento multidireccional lineal (Horizontal, Ascenso Vertical, Descenso Vertical),
//! oleadas de enemigos inspiradas en Final Mission / Abadox y activación del jefe.

use crate::enemy::{Enemy, EnemyType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StagePhase {
    HorizontalRight, // Sección 1 & 4: Vuelo horizontal hacia adelante (Megaciudad / Autopista aérea)
    AscendUp,        // Sección 2: Ascenso vertical (scrolling UP) por cajas de ascensores y rascacielos
    DescendDown,     // Sección 3: Descenso vertical (scrolling DOWN) a fundiciones y fosas subterráneas
    BossEncounter,   // Arena de combate contra el Jefe Colosal (velocidad estabilizada)
}

pub type Phase = StagePhase;

#[derive(Clone, Debug)]
pub struct StageConfig {
    pub number: u8,
    pub name: String,
    pub subtitle: String,
    pub scroll_speed: f32,
    pub stage_length: f32,
    pub bg_color: u32,
    pub grid_color: u32,
}

#[derive(Clone, Debug)]
pub struct LevelManager {
    pub current_stage: u8,
    // Vector de cámara 2D y velocidades
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scroll_vx: f32,
    pub scroll_vy: f32,
    pub target_vx: f32,
    pub target_vy: f32,
    pub current_phase: StagePhase,
    pub last_phase: StagePhase,
    pub transition_timer: f32,
    pub transition_text: &'static str,
    pub distance_traveled: f32,
    pub scroll_pos: f32, // Compatibilidad retrospectiva (alias de distance_traveled)
    pub stage_progress: f32,
    pub boss_spawned: bool,
    pub spawn_timer: f32,
    pub config: StageConfig,
}

impl LevelManager {
    pub fn new(stage: u8) -> Self {
        let config = Self::get_config(stage);
        let speed = config.scroll_speed;
        Self {
            current_stage: stage,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scroll_vx: speed,
            scroll_vy: 0.0,
            target_vx: speed,
            target_vy: 0.0,
            current_phase: StagePhase::HorizontalRight,
            last_phase: StagePhase::HorizontalRight,
            transition_timer: 0.0,
            transition_text: "",
            distance_traveled: 0.0,
            scroll_pos: 0.0,
            stage_progress: 0.0,
            boss_spawned: false,
            spawn_timer: 1.0,
            config,
        }
    }

    pub fn set_stage(&mut self, stage: u8) {
        self.current_stage = stage.clamp(1, 8);
        self.config = Self::get_config(self.current_stage);
        let speed = self.config.scroll_speed;
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
        self.scroll_vx = speed;
        self.scroll_vy = 0.0;
        self.target_vx = speed;
        self.target_vy = 0.0;
        self.current_phase = StagePhase::HorizontalRight;
        self.last_phase = StagePhase::HorizontalRight;
        self.transition_timer = 0.0;
        self.transition_text = "";
        self.distance_traveled = 0.0;
        self.scroll_pos = 0.0;
        self.stage_progress = 0.0;
        self.boss_spawned = false;
        self.spawn_timer = 1.0;
    }

    fn get_config(stage: u8) -> StageConfig {
        match stage {
            1 => StageConfig {
                number: 1,
                name: "STAGE 1: RUINED NEW YORK CITY".to_string(),
                subtitle: "Earth Zero Zone - Ruined Bastion & Broken Skylines".to_string(),
                scroll_speed: 160.0,
                stage_length: 5600.0,
                bg_color: 0xFF000000,
                grid_color: 0x337F00EF,
            },
            2 => StageConfig {
                number: 2,
                name: "STAGE 2: DRONE FORGE".to_string(),
                subtitle: "Earth - Automated Swarm Assembly & Smelting Vats".to_string(),
                scroll_speed: 175.0,
                stage_length: 5800.0,
                bg_color: 0xFF140D07,
                grid_color: 0x33FF4500,
            },
            3 => StageConfig {
                number: 3,
                name: "STAGE 3: MARS CYBER-FOUNDRY".to_string(),
                subtitle: "Mars - Iron Oxide Canyons & Automated Mining Rigs".to_string(),
                scroll_speed: 185.0,
                stage_length: 6000.0,
                bg_color: 0xFF1A0802,
                grid_color: 0x33FF3300,
            },
            4 => StageConfig {
                number: 4,
                name: "STAGE 4: EUROPA SUB-GLACIAL NETWORK".to_string(),
                subtitle: "Jupiter's Moon Europa - Sub-Zero Cryo Caverns".to_string(),
                scroll_speed: 180.0,
                stage_length: 6200.0,
                bg_color: 0xFF03141C,
                grid_color: 0x3300E5FF,
            },
            5 => StageConfig {
                number: 5,
                name: "STAGE 5: HEPHAESTUS SOLAR BASTION".to_string(),
                subtitle: "Mercury / Solar Orbit - Solar Prominence & Defense Grids".to_string(),
                scroll_speed: 210.0,
                stage_length: 6400.0,
                bg_color: 0xFF1F0F00,
                grid_color: 0x33FFAA00,
            },
            6 => StageConfig {
                number: 6,
                name: "STAGE 6: TITAN METHANE SPIRE".to_string(),
                subtitle: "Saturn's Moon Titan - Rings of Saturn & Methane Refineries".to_string(),
                scroll_speed: 220.0,
                stage_length: 6600.0,
                bg_color: 0xFF191004,
                grid_color: 0x33FF7700,
            },
            7 => StageConfig {
                number: 7,
                name: "STAGE 7: NEMESIS MOTHERSHIP FLEET".to_string(),
                subtitle: "Deep Space - AI Armada Flagship & Flak Corridors".to_string(),
                scroll_speed: 240.0,
                stage_length: 7000.0,
                bg_color: 0xFF0D0614,
                grid_color: 0x33FF007F,
            },
            _ => StageConfig {
                number: 8,
                name: "STAGE 8: QUANTUM SINGULARITY CORE".to_string(),
                subtitle: "The AI Overmind - Quantum Singularity & Master Core".to_string(),
                scroll_speed: 260.0,
                stage_length: 7500.0,
                bg_color: 0xFF000000,
                grid_color: 0x33FF0033,
            },
        }
    }

    pub fn update(&mut self, dt: f32, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) -> bool {
        // 1. Determinar fase y velocidades objetivo de trayectoria según el avance del nivel
        let old_phase = self.current_phase;
        self.update_phase_and_trajectory();
        if self.current_phase != old_phase {
            self.last_phase = old_phase;
            self.transition_timer = 2.8;
            self.transition_text = match self.current_phase {
                StagePhase::AscendUp => ">>> WARNING: CYBER-TOWER ELEVATOR ASCENT >>>",
                StagePhase::DescendDown => ">>> CAUTION: SUBTERRANEAN FOUNDRY DESCENT >>>",
                StagePhase::HorizontalRight => ">>> ALERT: HIGH-SPEED SKYWAY SPRINT >>>",
                StagePhase::BossEncounter => ">>> CRITICAL WARNING: COLOSSAL AI WARSHIP INCOMING >>>",
            };
        }
        if self.transition_timer > 0.0 {
            self.transition_timer = (self.transition_timer - dt).max(0.0);
        }

        // 2. Transición suave de la trayectoria de cámara (curvas y amortiguación física)
        let blend_rate = 2.8;
        self.scroll_vx += (self.target_vx - self.scroll_vx) * (blend_rate * dt).min(1.0);
        self.scroll_vy += (self.target_vy - self.scroll_vy) * (blend_rate * dt).min(1.0);

        // 3. Integración del vector de desplazamiento 2D
        self.scroll_x += self.scroll_vx * dt;
        self.scroll_y += self.scroll_vy * dt;

        // 4. Acumular distancia lineal recorrida a lo largo de la ruta
        let speed = (self.scroll_vx * self.scroll_vx + self.scroll_vy * self.scroll_vy).sqrt();
        self.distance_traveled += speed * dt;
        self.scroll_pos = self.distance_traveled;
        self.stage_progress = (self.distance_traveled / self.config.stage_length).clamp(0.0, 1.0);

        // 5. Comprueba si debe spawnear el Boss al culminar el trayecto
        if self.stage_progress >= 1.0 && !self.boss_spawned {
            self.boss_spawned = true;
            self.current_phase = StagePhase::BossEncounter;
            self.transition_timer = 2.8;
            self.transition_text = ">>> CRITICAL WARNING: COLOSSAL AI WARSHIP INCOMING >>>";
            self.target_vx = 0.0;
            self.target_vy = 0.0;
            return true;
        }

        // 6. Oleadas periódicas adaptadas a la dirección del vuelo
        if !self.boss_spawned {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = self.calculate_spawn_timer();
                self.spawn_wave(screen_w, screen_h, enemies);
            }
        }
        false
    }

    fn update_phase_and_trajectory(&mut self) {
        if self.boss_spawned || self.stage_progress >= 1.0 {
            self.current_phase = StagePhase::BossEncounter;
            self.target_vx = 0.0;
            self.target_vy = 0.0;
            return;
        }

        let speed = self.config.scroll_speed;

        // Trayecto multidireccional lineal fiel a NES Final Mission & Abadox:
        // - Sección 1 (0.00..0.28): Vuelo horizontal por la megaciudad destruida
        // - Sección 2 (0.28..0.54): Ascenso vertical (scrolling UP) por rascacielos y cajas de ascensores
        // - Sección 3 (0.54..0.78): Descenso vertical (scrolling DOWN) a fundiciones y fosas subterráneas
        // - Sección 4 (0.78..1.00): Retorno a vuelo horizontal supersónico hacia el Boss
        if self.stage_progress < 0.28 {
            self.current_phase = StagePhase::HorizontalRight;
            self.target_vx = speed;
            self.target_vy = 0.0;
        } else if self.stage_progress < 0.54 {
            self.current_phase = StagePhase::AscendUp;
            self.target_vx = 0.0;
            self.target_vy = -speed; // Cámara sube hacia arriba (escenario se desplaza hacia abajo)
        } else if self.stage_progress < 0.78 {
            self.current_phase = StagePhase::DescendDown;
            self.target_vx = 0.0;
            self.target_vy = speed;  // Cámara desciende hacia abajo (escenario se desplaza hacia arriba)
        } else {
            self.current_phase = StagePhase::HorizontalRight;
            self.target_vx = speed * 1.15; // Sprint final acelerado
            self.target_vy = 0.0;
        }
    }

    fn calculate_spawn_timer(&self) -> f32 {
        match self.current_stage {
            1 => match self.current_phase {
                StagePhase::HorizontalRight => {
                    if self.stage_progress < 0.28 { 1.8 } else { 1.2 }
                }
                StagePhase::AscendUp => 1.5,
                StagePhase::DescendDown => 1.4,
                StagePhase::BossEncounter => 999.0,
            },
            2..=4 => 1.4,
            5..=6 => 1.2,
            _ => 0.95,
        }
    }

    /// Devuelve los límites espaciales del jugador (min_x, max_x, min_y, max_y)
    /// según la fase activa (corredor horizontal vs pozo vertical)
    pub fn get_player_bounds(&self, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
        match self.current_phase {
            StagePhase::HorizontalRight => (36.0, screen_w - 36.0, 52.0, screen_h - 48.0),
            StagePhase::AscendUp => (56.0, screen_w - 56.0, 36.0, screen_h - 36.0),
            StagePhase::DescendDown => (56.0, screen_w - 56.0, 36.0, screen_h - 36.0),
            StagePhase::BossEncounter => (36.0, screen_w - 36.0, 44.0, screen_h - 44.0),
        }
    }

    fn spawn_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        if self.current_stage == 1 {
            self.spawn_stage1_multidirectional_wave(screen_w, screen_h, enemies);
            return;
        }
        self.spawn_planetary_wave(screen_w, screen_h, enemies);
    }

    /// Trayecto multifase de Stage 1 inspirado fielmente en Final Mission y Abadox
    fn spawn_stage1_multidirectional_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let spawn_x_right = screen_w + 40.0;
        let step = (self.distance_traveled as usize / 120) % 4;

        match self.current_phase {
            StagePhase::HorizontalRight => {
                if self.stage_progress < 0.28 {
                    // === SECCIÓN 1: Lower ruined street & pipe platforms (Vuelo horizontal inicial) ===
                    match step {
                        0 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.52, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 65.0, screen_h * 0.72, EnemyType::PatrolDrone));
                        }
                        1 => {
                            for i in 0..4 {
                                let y = screen_h * 0.50 + (i as f32 * 55.0);
                                enemies.push(Enemy::new(spawn_x_right + (i as f32 * 45.0), y, EnemyType::KamikazeWasp));
                            }
                        }
                        2 => {
                            enemies.push(Enemy::new(spawn_x_right, 68.0, EnemyType::LaserTurret));
                            enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h - 68.0, EnemyType::LaserTurret));
                        }
                        _ => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.60, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 50.0, screen_h - 68.0, EnemyType::LaserTurret));
                            enemies.push(Enemy::new(spawn_x_right + 90.0, screen_h * 0.38, EnemyType::PatrolDrone));
                        }
                    }
                } else {
                    // === SECCIÓN 4: Elevated sky-highway sprint toward TITAN-01 Warcrawler ===
                    match step {
                        0 => {
                            // Muro defensivo de 4 PatrolDrones alineados verticalmente (Drone Barrier)
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.22, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.42, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.62, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.82, EnemyType::PatrolDrone));
                        }
                        1 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.35, EnemyType::StealthStriker));
                            enemies.push(Enemy::new(spawn_x_right + 85.0, screen_h * 0.65, EnemyType::StealthStriker));
                        }
                        2 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.50, EnemyType::CyberCrab));
                            enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h - 75.0, EnemyType::LaserTurret));
                            enemies.push(Enemy::new(spawn_x_right + 90.0, 65.0, EnemyType::LaserTurret));
                        }
                        _ => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.30, EnemyType::StealthStriker));
                            enemies.push(Enemy::new(spawn_x_right + 50.0, screen_h * 0.50, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 90.0, screen_h * 0.70, EnemyType::StealthStriker));
                        }
                    }
                }
            }
            StagePhase::AscendUp => {
                // === SECCIÓN 2: Vertical ASCENT (scrolling UP) through Skyscraper Shaft ===
                // El jugador sube: los peligros descienden desde arriba (-Y) o surgen de los muros laterales del pozo
                let spawn_y_top = -35.0;
                let left_wall_x = 48.0;
                let right_wall_x = screen_w - 48.0;

                match step {
                    0 => {
                        // Torretas centinela ancladas a las paredes laterales del pozo de ascensor
                        let mut t_left = Enemy::new(left_wall_x, spawn_y_top, EnemyType::LaserTurret);
                        t_left.aim_angle = 0.0; // Orientada hacia el centro del pozo (derecha)
                        let mut t_right = Enemy::new(right_wall_x, spawn_y_top - 60.0, EnemyType::LaserTurret);
                        t_right.aim_angle = std::f32::consts::PI; // Orientada hacia el centro (izquierda)
                        enemies.push(t_left);
                        enemies.push(t_right);
                    }
                    1 => {
                        // Enjambre de avispas que caen en picada en formación vertical
                        for i in 0..4 {
                            let x = screen_w * 0.30 + (i as f32 * 50.0);
                            let mut wasp = Enemy::new(x, spawn_y_top - (i as f32 * 45.0), EnemyType::KamikazeWasp);
                            wasp.vy = 180.0;
                            enemies.push(wasp);
                        }
                    }
                    2 => {
                        // Sondas parásitas / minas magnéticas descendentes en el centro del eje
                        let mut leech1 = Enemy::new(screen_w * 0.38, spawn_y_top, EnemyType::AsteroidLeech);
                        leech1.vy = 140.0;
                        let mut leech2 = Enemy::new(screen_w * 0.62, spawn_y_top - 50.0, EnemyType::AsteroidLeech);
                        leech2.vy = 140.0;
                        enemies.push(leech1);
                        enemies.push(leech2);
                    }
                    _ => {
                        // Androide de asalto pesado que cae con retrocohetes y torreta lateral
                        let mut crab = Enemy::new(screen_w * 0.50, spawn_y_top, EnemyType::CyberCrab);
                        crab.vy = 110.0;
                        let mut t_right = Enemy::new(right_wall_x, spawn_y_top - 40.0, EnemyType::LaserTurret);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(crab);
                        enemies.push(t_right);
                    }
                }
            }
            StagePhase::DescendDown => {
                // === SECCIÓN 3: Vertical DESCENT (scrolling DOWN) into Subterranean Foundry ===
                // El jugador desciende a la fosa: los peligros ascienden desde abajo (+Y) o se montan en tuberías de fundición
                let spawn_y_bottom = screen_h + 35.0;
                let left_wall_x = 52.0;
                let right_wall_x = screen_w - 52.0;

                match step {
                    0 => {
                        // Torretas de fundición en muros inferiores emergiendo desde el foso
                        let mut t_left = Enemy::new(left_wall_x, spawn_y_bottom, EnemyType::LaserTurret);
                        t_left.aim_angle = 0.0;
                        let mut t_right = Enemy::new(right_wall_x, spawn_y_bottom + 50.0, EnemyType::LaserTurret);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(t_left);
                        enemies.push(t_right);
                    }
                    1 => {
                        // Enjambre de drones de magma ascendiendo velozmente hacia el jugador
                        for i in 0..4 {
                            let x = screen_w * 0.28 + (i as f32 * 55.0);
                            let mut wasp = Enemy::new(x, spawn_y_bottom + (i as f32 * 45.0), EnemyType::KamikazeWasp);
                            wasp.vy = -170.0;
                            enemies.push(wasp);
                        }
                    }
                    2 => {
                        // Minas de succión magnética y drones de fundición
                        let mut drone = Enemy::new(screen_w * 0.45, spawn_y_bottom, EnemyType::PatrolDrone);
                        drone.vy = -120.0;
                        let mut leech = Enemy::new(screen_w * 0.65, spawn_y_bottom + 40.0, EnemyType::AsteroidLeech);
                        leech.vy = -150.0;
                        enemies.push(drone);
                        enemies.push(leech);
                    }
                    _ => {
                        // Cañonera furtiva que asciende desde la fosa y torreta izquierda
                        let mut striker = Enemy::new(screen_w * 0.50, spawn_y_bottom, EnemyType::StealthStriker);
                        striker.vy = -190.0;
                        let mut t_left = Enemy::new(left_wall_x, spawn_y_bottom + 60.0, EnemyType::LaserTurret);
                        t_left.aim_angle = 0.0;
                        enemies.push(striker);
                        enemies.push(t_left);
                    }
                }
            }
            StagePhase::BossEncounter => {}
        }
    }

    /// Oleadas para escenarios planetarios (Fases 2 a 8) con soporte multidireccional
    fn spawn_planetary_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let spawn_x_right = screen_w + 40.0;
        let step = (self.distance_traveled as usize / 120) % 6;

        match self.current_phase {
            StagePhase::HorizontalRight => {
                match step {
                    0 => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.3, EnemyType::PatrolDrone));
                        enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h * 0.5, EnemyType::PatrolDrone));
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.7, EnemyType::PatrolDrone));
                    }
                    1 => {
                        for i in 0..4 {
                            let y = screen_h * 0.25 + (i as f32 * 110.0);
                            enemies.push(Enemy::new(spawn_x_right + (i as f32 * 50.0), y, EnemyType::KamikazeWasp));
                        }
                    }
                    2 => {
                        enemies.push(Enemy::new(spawn_x_right, 68.0, EnemyType::LaserTurret));
                        enemies.push(Enemy::new(spawn_x_right + 70.0, screen_h - 68.0, EnemyType::LaserTurret));
                    }
                    3 => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.5, EnemyType::CyberCrab));
                        enemies.push(Enemy::new(spawn_x_right + 90.0, screen_h * 0.35, EnemyType::AsteroidLeech));
                    }
                    4 => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.38, EnemyType::StealthStriker));
                        enemies.push(Enemy::new(spawn_x_right + 80.0, screen_h * 0.62, EnemyType::StealthStriker));
                    }
                    _ => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.45, EnemyType::PatrolDrone));
                        enemies.push(Enemy::new(spawn_x_right + 50.0, screen_h - 68.0, EnemyType::LaserTurret));
                    }
                }
            }
            StagePhase::AscendUp => {
                let spawn_y_top = -35.0;
                let mut e1 = Enemy::new(screen_w * 0.35, spawn_y_top, EnemyType::PatrolDrone);
                e1.vy = 150.0;
                let mut e2 = Enemy::new(screen_w * 0.65, spawn_y_top - 40.0, EnemyType::AsteroidLeech);
                e2.vy = 150.0;
                enemies.push(e1);
                enemies.push(e2);
            }
            StagePhase::DescendDown => {
                let spawn_y_bottom = screen_h + 35.0;
                let mut e1 = Enemy::new(screen_w * 0.40, spawn_y_bottom, EnemyType::KamikazeWasp);
                e1.vy = -160.0;
                let mut e2 = Enemy::new(screen_w * 0.60, spawn_y_bottom + 40.0, EnemyType::CyberCrab);
                e2.vy = -110.0;
                enemies.push(e1);
                enemies.push(e2);
            }
            StagePhase::BossEncounter => {}
        }
    }
}
