//! Módulo de Gestión de los 8 Niveles de IA Rebellion
//! Controla el desplazamiento del escenario, oleadas de enemigos inspiradas en Final Mission y activación del jefe

use crate::enemy::{Enemy, EnemyType};

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
    pub scroll_pos: f32,
    pub stage_progress: f32,
    pub boss_spawned: bool,
    pub spawn_timer: f32,
    pub config: StageConfig,
}

impl LevelManager {
    pub fn new(stage: u8) -> Self {
        let config = Self::get_config(stage);
        Self {
            current_stage: stage,
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
                stage_length: 5600.0, // Longitud extendida épica (de 3400 a 5600)
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
        self.scroll_pos += self.config.scroll_speed * dt;
        self.stage_progress = self.scroll_pos / self.config.stage_length;

        // Comprueba si debe spawnear el Boss al culminar el trayecto
        if self.stage_progress >= 1.0 && !self.boss_spawned {
            self.boss_spawned = true;
            return true;
        }

        // Oleadas periódicas con cadencia que se intensifica al aproximarse al clímax
        if !self.boss_spawned {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = match self.current_stage {
                    1 => {
                        if self.stage_progress < 0.35 {
                            1.8 // Fase A: Reconocimiento y contacto en la calle
                        } else if self.stage_progress < 0.70 {
                            1.5 // Fase B: Ascenso vertical y emboscada de torretas
                        } else {
                            1.2 // Fase C: Clímax de autopista aérea hacia el Titán
                        }
                    }
                    2..=4 => 1.4,
                    5..=6 => 1.2,
                    _ => 0.95,
                };
                self.spawn_wave(screen_w, screen_h, enemies);
            }
        }
        false
    }

    fn spawn_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let spawn_x = screen_w + 40.0;

        if self.current_stage == 1 {
            self.spawn_stage1_multiphase_wave(spawn_x, screen_w, screen_h, enemies);
            return;
        }

        self.spawn_planetary_wave(spawn_x, screen_w, screen_h, enemies);
    }

    /// Trayecto multifase de Stage 1 inspirado fielmente en las capturas de Final Mission
    fn spawn_stage1_multiphase_wave(&mut self, spawn_x: f32, _screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        if self.stage_progress < 0.35 {
            // === FASE A: Lower ruined street & pipe platforms ===
            // Drones de patrulla baja, enjambre bivalvo a ras de suelo y torretas montadas en tuberías inferiores
            let step = (self.scroll_pos as usize / 130) % 4;
            match step {
                0 => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.52, EnemyType::PatrolDrone));
                    enemies.push(Enemy::new(spawn_x + 65.0, screen_h * 0.72, EnemyType::PatrolDrone));
                }
                1 => {
                    for i in 0..4 {
                        let y = screen_h * 0.50 + (i as f32 * 55.0);
                        enemies.push(Enemy::new(spawn_x + (i as f32 * 45.0), y, EnemyType::KamikazeWasp));
                    }
                }
                2 => {
                    enemies.push(Enemy::new(spawn_x, 68.0, EnemyType::LaserTurret));
                    enemies.push(Enemy::new(spawn_x + 60.0, screen_h - 68.0, EnemyType::LaserTurret));
                }
                _ => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.60, EnemyType::PatrolDrone));
                    enemies.push(Enemy::new(spawn_x + 50.0, screen_h - 68.0, EnemyType::LaserTurret));
                    enemies.push(Enemy::new(spawn_x + 90.0, screen_h * 0.38, EnemyType::PatrolDrone));
                }
            }
        } else if self.stage_progress < 0.70 {
            // === FASE B: Vertical tower ascent with hanging sentry turrets ===
            // Ascenso entre rascacielos y cajas de ascensores; torretas colgantes del techo y zarcillos magnéticos
            let step = (self.scroll_pos as usize / 125) % 4;
            match step {
                0 => {
                    // Torretas centinela colgantes del techo (y < 140 -> is_ceiling = true)
                    enemies.push(Enemy::new(spawn_x, 62.0, EnemyType::LaserTurret));
                    enemies.push(Enemy::new(spawn_x + 85.0, 95.0, EnemyType::LaserTurret));
                    enemies.push(Enemy::new(spawn_x + 40.0, screen_h * 0.50, EnemyType::AsteroidLeech));
                }
                1 => {
                    for i in 0..4 {
                        let y = 90.0 + (i as f32 * 65.0);
                        enemies.push(Enemy::new(spawn_x + (i as f32 * 48.0), y, EnemyType::KamikazeWasp));
                    }
                }
                2 => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.45, EnemyType::CyberCrab));
                    enemies.push(Enemy::new(spawn_x + 70.0, 62.0, EnemyType::LaserTurret));
                }
                _ => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.32, EnemyType::AsteroidLeech));
                    enemies.push(Enemy::new(spawn_x + 60.0, screen_h * 0.68, EnemyType::AsteroidLeech));
                    enemies.push(Enemy::new(spawn_x + 110.0, 62.0, EnemyType::LaserTurret));
                }
            }
        } else {
            // === FASE C: Elevated sky-highway with heavy drone barriers leading to TITAN-01 Warcrawler ===
            // Autopista aérea suspendida a gran altitud, barreras pesadas de drones y cañoneras tácticas
            let step = (self.scroll_pos as usize / 115) % 4;
            match step {
                0 => {
                    // Muro defensivo de 4 PatrolDrones alineados verticalmente (Drone Barrier)
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.22, EnemyType::PatrolDrone));
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.42, EnemyType::PatrolDrone));
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.62, EnemyType::PatrolDrone));
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.82, EnemyType::PatrolDrone));
                }
                1 => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.35, EnemyType::StealthStriker));
                    enemies.push(Enemy::new(spawn_x + 85.0, screen_h * 0.65, EnemyType::StealthStriker));
                }
                2 => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.50, EnemyType::CyberCrab));
                    enemies.push(Enemy::new(spawn_x + 60.0, screen_h - 75.0, EnemyType::LaserTurret));
                    enemies.push(Enemy::new(spawn_x + 90.0, 65.0, EnemyType::LaserTurret));
                }
                _ => {
                    enemies.push(Enemy::new(spawn_x, screen_h * 0.30, EnemyType::StealthStriker));
                    enemies.push(Enemy::new(spawn_x + 50.0, screen_h * 0.50, EnemyType::PatrolDrone));
                    enemies.push(Enemy::new(spawn_x + 90.0, screen_h * 0.70, EnemyType::StealthStriker));
                }
            }
        }
    }

    /// Oleadas para escenarios planetarios (Fases 2 a 8)
    fn spawn_planetary_wave(&mut self, spawn_x: f32, _screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let p = (self.scroll_pos as usize / 120) % 6;
        match p {
            0 => {
                enemies.push(Enemy::new(spawn_x, screen_h * 0.3, EnemyType::PatrolDrone));
                enemies.push(Enemy::new(spawn_x + 60.0, screen_h * 0.5, EnemyType::PatrolDrone));
                enemies.push(Enemy::new(spawn_x, screen_h * 0.7, EnemyType::PatrolDrone));
            }
            1 => {
                for i in 0..4 {
                    let y = screen_h * 0.25 + (i as f32 * 110.0);
                    enemies.push(Enemy::new(spawn_x + (i as f32 * 50.0), y, EnemyType::KamikazeWasp));
                }
            }
            2 => {
                enemies.push(Enemy::new(spawn_x, 68.0, EnemyType::LaserTurret));
                enemies.push(Enemy::new(spawn_x + 70.0, screen_h - 68.0, EnemyType::LaserTurret));
            }
            3 => {
                enemies.push(Enemy::new(spawn_x, screen_h * 0.5, EnemyType::CyberCrab));
                enemies.push(Enemy::new(spawn_x + 90.0, screen_h * 0.35, EnemyType::AsteroidLeech));
            }
            4 => {
                enemies.push(Enemy::new(spawn_x, screen_h * 0.38, EnemyType::StealthStriker));
                enemies.push(Enemy::new(spawn_x + 80.0, screen_h * 0.62, EnemyType::StealthStriker));
            }
            _ => {
                enemies.push(Enemy::new(spawn_x, screen_h * 0.45, EnemyType::PatrolDrone));
                enemies.push(Enemy::new(spawn_x + 50.0, screen_h - 68.0, EnemyType::LaserTurret));
            }
        }
    }
}
