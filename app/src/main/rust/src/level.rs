//! Módulo de Gestión de los 8 Niveles de IA Rebellion
//! Controla el desplazamiento del escenario, oleadas de enemigos y activación del jefe

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
                name: "STAGE 1: EARTH ORBIT".to_string(),
                subtitle: "Entrada Atmosférica y Escuadrón Centinela".to_string(),
                scroll_speed: 160.0,
                stage_length: 3200.0,
                bg_color: 0xFF050515,
                grid_color: 0x3300D2FF,
            },
            2 => StageConfig {
                number: 2,
                name: "STAGE 2: DRONE FORGE".to_string(),
                subtitle: "Fábrica Automatizada de Enjambres en la Tierra".to_string(),
                scroll_speed: 180.0,
                stage_length: 3400.0,
                bg_color: 0xFF150800,
                grid_color: 0x33FF4500,
            },
            3 => StageConfig {
                number: 3,
                name: "STAGE 3: CYBER MEGACITY".to_string(),
                subtitle: "Ruinas Tecnológicas y Red Troncal".to_string(),
                scroll_speed: 190.0,
                stage_length: 3600.0,
                bg_color: 0xFF080015,
                grid_color: 0x339400D3,
            },
            4 => StageConfig {
                number: 4,
                name: "STAGE 4: BIO-ORGANIC HIVE".to_string(),
                subtitle: "Complejo Subterráneo Mutado (Tributo a Abadox)".to_string(),
                scroll_speed: 170.0,
                stage_length: 3800.0,
                bg_color: 0xFF0A1400,
                grid_color: 0x3332CD32,
            },
            5 => StageConfig {
                number: 5,
                name: "STAGE 5: HEPHAESTUS STATION".to_string(),
                subtitle: "Estación Militar Orbital de Defensa Láser".to_string(),
                scroll_speed: 210.0,
                stage_length: 4000.0,
                bg_color: 0xFF00121C,
                grid_color: 0x3300FFFF,
            },
            6 => StageConfig {
                number: 6,
                name: "STAGE 6: ASTEROID BELT".to_string(),
                subtitle: "Cinturón de Asteroides y Minas Autónomas".to_string(),
                scroll_speed: 230.0,
                stage_length: 4200.0,
                bg_color: 0xFF101018,
                grid_color: 0x33FFD700,
            },
            7 => StageConfig {
                number: 7,
                name: "STAGE 7: NEMESIS MOTHERSHIP".to_string(),
                subtitle: "Flota Matriz de la Armada IA".to_string(),
                scroll_speed: 240.0,
                stage_length: 4500.0,
                bg_color: 0xFF180010,
                grid_color: 0x33FF1493,
            },
            _ => StageConfig {
                number: 8,
                name: "STAGE 8: QUANTUM SINGULARITY".to_string(),
                subtitle: "Núcleo Central de la IA Rebelde Master".to_string(),
                scroll_speed: 260.0,
                stage_length: 4800.0,
                bg_color: 0xFF000000,
                grid_color: 0x33FF0033,
            },
        }
    }

    pub fn update(&mut self, dt: f32, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) -> bool {
        self.scroll_pos += self.config.scroll_speed * dt;
        self.stage_progress = self.scroll_pos / self.config.stage_length;

        // Comprueba si debe spawnear el Boss
        if self.stage_progress >= 1.0 && !self.boss_spawned {
            self.boss_spawned = true;
            return true; // Momento de invocar al Boss
        }

        // Oleadas periódicas de enemigos mientras avanza el nivel
        if !self.boss_spawned {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = match self.current_stage {
                    1..=2 => 1.8,
                    3..=5 => 1.4,
                    _ => 1.0,
                };
                self.spawn_wave(screen_w, screen_h, enemies);
            }
        }
        false
    }

    fn spawn_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let spawn_x = screen_w + 40.0;
        let p = (self.scroll_pos as usize / 120) % 6;

        match p {
            0 => {
                // Formación V de drones patrulleros
                enemies.push(Enemy::new(spawn_x, screen_h * 0.3, EnemyType::PatrolDrone));
                enemies.push(Enemy::new(spawn_x + 60.0, screen_h * 0.5, EnemyType::PatrolDrone));
                enemies.push(Enemy::new(spawn_x, screen_h * 0.7, EnemyType::PatrolDrone));
            }
            1 => {
                // Enjambre kamikaze rápido
                for i in 0..4 {
                    let y = screen_h * 0.2 + (i as f32 * 140.0);
                    enemies.push(Enemy::new(spawn_x + (i as f32 * 40.0), y, EnemyType::KamikazeWasp));
                }
            }
            2 => {
                // Torretas pesadas
                enemies.push(Enemy::new(spawn_x, screen_h * 0.25, EnemyType::LaserTurret));
                enemies.push(Enemy::new(spawn_x, screen_h * 0.75, EnemyType::LaserTurret));
            }
            3 => {
                // Criaturas biomecánicas / asteroides
                enemies.push(Enemy::new(spawn_x, screen_h * 0.5, EnemyType::CyberCrab));
                enemies.push(Enemy::new(spawn_x + 90.0, screen_h * 0.3, EnemyType::AsteroidLeech));
            }
            4 => {
                // Cazas furtivos
                enemies.push(Enemy::new(spawn_x, screen_h * 0.4, EnemyType::StealthStriker));
                enemies.push(Enemy::new(spawn_x + 80.0, screen_h * 0.6, EnemyType::StealthStriker));
            }
            _ => {
                enemies.push(Enemy::new(spawn_x, screen_h * 0.5, EnemyType::PatrolDrone));
            }
        }
    }
}
