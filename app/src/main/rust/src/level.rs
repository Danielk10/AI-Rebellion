//! Módulo de Gestión de los 8 Niveles de IA Rebellion
//! Controla el desplazamiento multidireccional lineal (Horizontal, Ascenso Vertical, Descenso Vertical),
//! oleadas de enemigos inspiradas en Final Mission / Abadox y activación del jefe.

use crate::enemy::{Enemy, EnemyAnchor, EnemyType, ItemType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StagePhase {
    HorizontalRight, // Sección 1 & 4: Vuelo horizontal hacia adelante (Megaciudad / Autopista aérea)
    AscendUp,        // Sección 2: Ascenso vertical (scrolling UP) por cajas de ascensores y rascacielos
    DescendDown,     // Sección 3: Descenso vertical (scrolling DOWN) a fundiciones y fosas subterráneas
    BossEncounter,   // Arena de combate contra el Jefe Colosal (velocidad estabilizada)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObstacleStyle {
    SteelGirder,      // Stage 1: Vigas Warren, tuberías y pilares urbanos
    SmeltingPress,    // Stage 2: Prensas hidráulicas y crisoles de fundición
    MarsCanyonRock,   // Stage 3: Formaciones de óxido marciano y pilones mineros
    CryoIcePillar,    // Stage 4: Columnas criogénicas de Europa y bio-nodos Abadox
    SolarShield,      // Stage 5: Blindajes térmicos solares y colectores Dyson
    MethaneTank,      // Stage 6: Silos de metano líquido y celosías de Titán
    ArmadaBulkhead,   // Stage 7: Mamparas acorazadas y torretas de la flota nodriza
    QuantumMonolith,  // Stage 8: Monolitos cuánticos y bobinas de singularidad
}

#[derive(Clone, Debug)]
pub struct StageObstacle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style: ObstacleStyle,
    pub destructible: bool,
    pub health: f32,
    pub max_health: f32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct StageConfig {
    pub name: String,
    pub subtitle: String,
    pub scroll_speed: f32,
    pub stage_length: f32,
    pub bg_color: u32,
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
    pub previous_phase: StagePhase,
    pub inside_chamber: bool,
    pub transition_timer: f32,
    pub transition_text: &'static str,
    pub distance_traveled: f32,
    pub stage_progress: f32,
    pub boss_spawned: bool,
    pub spawn_timer: f32,
    pub obstacle_timer: f32,
    pub obstacles: Vec<StageObstacle>,
    pub config: StageConfig,
    pub wave_index: usize,
    pub stage_carrier_count: usize,
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
            previous_phase: StagePhase::HorizontalRight,
            inside_chamber: false,
            transition_timer: 0.0,
            transition_text: "",
            distance_traveled: 0.0,
            stage_progress: 0.0,
            boss_spawned: false,
            spawn_timer: 1.0,
            obstacle_timer: 1.8,
            obstacles: Vec::with_capacity(24),
            config,
            wave_index: 0,
            stage_carrier_count: 0,
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
        self.previous_phase = StagePhase::HorizontalRight;
        self.inside_chamber = false;
        self.transition_timer = 0.0;
        self.transition_text = "";
        self.distance_traveled = 0.0;
        self.stage_progress = 0.0;
        self.boss_spawned = false;
        self.spawn_timer = 1.0;
        self.obstacle_timer = 1.8;
        self.obstacles.clear();
        self.wave_index = 0;
        self.stage_carrier_count = 0;
    }

    /// Progresión lógica de armamento y suministros estilo arcade clásico Final Mission:
    /// 'P' (Vulcan) -> 'L' (Laser) -> 'H' (Escudo) -> 'S' (Spread) -> 'M' (Homing) -> 'B' (Bomba EMP) -> '1UP' (Vida extra) -> 'H' (Escudo)
    pub fn next_stage_item(&mut self) -> ItemType {
        let item = match self.stage_carrier_count % 8 {
            0 => ItemType::WeaponVulcan, // 'P' - Aumento de poder de fuego / Vulcan
            1 => ItemType::WeaponLaser,  // 'L' - Rayo láser penetrante de alta potencia
            2 => ItemType::Shield,       // 'H' - Nano-médico (+40 HP)
            3 => ItemType::WeaponSpread, // 'S' - Escopeta de plasma en abanico
            4 => ItemType::WeaponHoming, // 'M' - Misiles teledirigidos swarm
            5 => ItemType::Bomb,         // 'B' - Recarga de Bomba Inteligente EMP (+1)
            6 => ItemType::ExtraLife,    // '1UP' - Vida extra de supervivencia
            _ => ItemType::Shield,       // 'H' - Recuperación de emergencia pre-Jefe
        };
        self.stage_carrier_count += 1;
        item
    }

    fn get_config(stage: u8) -> StageConfig {
        match stage {
            1 => StageConfig {
                name: "STAGE 1: RUINED NEW YORK CITY".to_string(),
                subtitle: "Earth Zero Zone - Ruined Bastion & Broken Skylines".to_string(),
                scroll_speed: 160.0,
                stage_length: 24000.0,
                bg_color: 0xFF000000,
            },
            2 => StageConfig {
                name: "STAGE 2: DRONE FORGE".to_string(),
                subtitle: "Earth - Automated Swarm Assembly & Smelting Vats".to_string(),
                scroll_speed: 175.0,
                stage_length: 24500.0,
                bg_color: 0xFF140D07,
            },
            3 => StageConfig {
                name: "STAGE 3: MARS CYBER-FOUNDRY".to_string(),
                subtitle: "Mars - Iron Oxide Canyons & Automated Mining Rigs".to_string(),
                scroll_speed: 185.0,
                stage_length: 25000.0,
                bg_color: 0xFF1A0802,
            },
            4 => StageConfig {
                name: "STAGE 4: EUROPA SUB-GLACIAL NETWORK".to_string(),
                subtitle: "Jupiter's Moon Europa - Sub-Zero Cryo Caverns".to_string(),
                scroll_speed: 180.0,
                stage_length: 25500.0,
                bg_color: 0xFF03141C,
            },
            5 => StageConfig {
                name: "STAGE 5: HEPHAESTUS SOLAR BASTION".to_string(),
                subtitle: "Mercury / Solar Orbit - Solar Prominence & Defense Grids".to_string(),
                scroll_speed: 210.0,
                stage_length: 26000.0,
                bg_color: 0xFF1F0F00,
            },
            6 => StageConfig {
                name: "STAGE 6: TITAN METHANE SPIRE".to_string(),
                subtitle: "Saturn's Moon Titan - Rings of Saturn & Methane Refineries".to_string(),
                scroll_speed: 220.0,
                stage_length: 26500.0,
                bg_color: 0xFF191004,
            },
            7 => StageConfig {
                name: "STAGE 7: NEMESIS MOTHERSHIP FLEET".to_string(),
                subtitle: "Deep Space - AI Armada Flagship & Flak Corridors".to_string(),
                scroll_speed: 240.0,
                stage_length: 27000.0,
                bg_color: 0xFF0D0614,
            },
            _ => StageConfig {
                name: "STAGE 8: QUANTUM SINGULARITY CORE".to_string(),
                subtitle: "The AI Overmind - Quantum Singularity & Master Core".to_string(),
                scroll_speed: 260.0,
                stage_length: 28000.0,
                bg_color: 0xFF000000,
            },
        }
    }

    pub fn update(&mut self, dt: f32, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) -> bool {
        // 1. Determinar fase y velocidades objetivo de trayectoria lineal estilo Final Mission
        let was_in_chamber = self.inside_chamber;
        let prev_phase = self.current_phase;
        self.update_phase_and_trajectory();

        if self.current_phase != prev_phase {
            self.previous_phase = prev_phase;
            self.handle_phase_transition(screen_w, screen_h, enemies);
        }

        // Entrar en cámara interior (último tramo antes del jefe) activa la alerta cinemática de brecha
        let now_in_chamber = self.stage_progress >= 0.78 && !self.boss_spawned;
        self.inside_chamber = now_in_chamber;

        if now_in_chamber && !was_in_chamber {
            self.transition_timer = 1.2;
            self.transition_text = match self.current_stage {
                1 => ">>> ENTERING INNER FORTRESS CHAMBER: TITAN-01 LAIR >>>",
                2 => ">>> ENTERING CENTRAL SMELTING REACTOR CHAMBER >>>",
                3 => ">>> ENTERING SUB-MARS CORE EXCAVATION CHAMBER >>>",
                4 => ">>> BREACHING ABADOX BIO-CYBERNETIC CHAMBER >>>",
                5 => ">>> ENTERING SOLAR CORE CONTAINMENT CHAMBER >>>",
                6 => ">>> ENTERING SATURN ORBITAL DOCKING CHAMBER >>>",
                7 => ">>> BREACHING MOTHERSHIP INNER COMMAND HANGAR >>>",
                _ => ">>> ENTERING NYX-OVERMIND SINGULARITY CHAMBER >>>",
            };
        }

        if self.transition_timer > 0.0 {
            self.transition_timer = (self.transition_timer - dt).max(0.0);
        }

        // 2. Transición lineal suave y fluida de la trayectoria de cámara (estilo Final Mission)
        let blend_rate = 2.8;
        self.scroll_vx += (self.target_vx - self.scroll_vx) * (blend_rate * dt).min(1.0);
        self.scroll_vy += (self.target_vy - self.scroll_vy) * (blend_rate * dt).min(1.0);

        // 3. Integración continua del vector de desplazamiento 2D
        self.scroll_x += self.scroll_vx * dt;
        self.scroll_y += self.scroll_vy * dt;

        // 4. Desplazar obstáculos físicos del escenario linealmente con la cámara
        for obs in self.obstacles.iter_mut() {
            if !obs.active {
                continue;
            }
            obs.x -= self.scroll_vx * dt;
            obs.y -= self.scroll_vy * dt;
            if obs.x < -160.0 || obs.x > screen_w + 220.0 || obs.y < -160.0 || obs.y > screen_h + 220.0 {
                obs.active = false;
            }
        }
        self.obstacles.retain(|o| o.active);

        // 5. Acumular distancia lineal recorrida a lo largo de la ruta
        let speed = (self.scroll_vx * self.scroll_vx + self.scroll_vy * self.scroll_vy).sqrt();
        self.distance_traveled += speed * dt;
        self.stage_progress = (self.distance_traveled / self.config.stage_length).clamp(0.0, 1.0);

        // 6. Comprueba si debe spawnear el Boss al culminar el trayecto
        if self.stage_progress >= 1.0 && !self.boss_spawned {
            self.boss_spawned = true;
            self.current_phase = StagePhase::BossEncounter;
            self.transition_timer = 0.0;
            self.transition_text = "";
            self.target_vx = 0.0;
            self.target_vy = 0.0;
            self.obstacles.clear();
            return true;
        }

        // 7. Oleadas periódicas de enemigos y obstáculos de escenario adaptados a la dirección lineal
        if !self.boss_spawned {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_timer = self.calculate_spawn_timer();
                self.spawn_wave(screen_w, screen_h, enemies);
            }

            self.obstacle_timer -= dt;
            if self.obstacle_timer <= 0.0 {
                self.obstacle_timer = 2.4;
                self.spawn_stage_obstacle(screen_w, screen_h, enemies.as_slice());
            }
        }
        false
    }

    /// Gestiona la transición limpia entre orientaciones de nivel:
    /// Banners cinemáticos compactos y de corta duración (transition_timer = 1.2s),
    /// sin banners sobre el jefe para no obstruir la visión del combate.
    pub fn handle_phase_transition(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        if self.current_phase == StagePhase::BossEncounter {
            self.transition_timer = 0.0;
            self.transition_text = "";
        } else {
            self.transition_timer = 1.2;
            self.transition_text = match self.current_phase {
                StagePhase::AscendUp => match self.current_stage {
                    1 => ">> WARNING: VERTICAL ASCENT VECTOR // SKYSCRAPER SHAFTS >>",
                    3 => ">> WARNING: VERTICAL ASCENT VECTOR // EXCAVATOR CHIMNEY >>",
                    5 => ">> WARNING: VERTICAL ASCENT VECTOR // DYSON SOLAR SPIRE >>",
                    6 => ">> WARNING: VERTICAL ASCENT VECTOR // TITAN SPACE ELEVATOR >>",
                    _ => ">> WARNING: VERTICAL ASCENT VECTOR ENGAGED >>",
                },
                StagePhase::DescendDown => match self.current_stage {
                    1 => ">> WARNING: VERTICAL DESCENT VECTOR // SUB-FOUNDRY SHAFT >>",
                    2 => ">> WARNING: VERTICAL DESCENT VECTOR // SMELTING VATS >>",
                    4 => ">> WARNING: VERTICAL DESCENT VECTOR // SUB-GLACIAL TRENCH >>",
                    7 => ">> WARNING: VERTICAL DESCENT VECTOR // DREADNOUGHT TRENCH >>",
                    8 => ">> WARNING: GRAVITATIONAL ANOMALY // EVENT HORIZON DIVE >>",
                    _ => ">> WARNING: VERTICAL DESCENT VECTOR ENGAGED >>",
                },
                StagePhase::HorizontalRight => match self.current_stage {
                    1 => ">> HORIZONTAL CRUISE // ROOFTOP RADAR & SKY-HIGHWAY >>",
                    _ => ">> RESUMING HORIZONTAL FLIGHT CORRIDOR // SENSORS ONLINE >>",
                },
                StagePhase::BossEncounter => "",
            };
        }

        match self.current_phase {
            StagePhase::AscendUp | StagePhase::DescendDown => {
                // Entrando a pozo vertical:
                // 1. Desactivar torretas de techo/suelo para que no queden flotando en el aire del pozo
                for e in enemies.iter_mut() {
                    if e.anchor == EnemyAnchor::Ceiling || e.anchor == EnemyAnchor::Floor {
                        e.active = false;
                    } else if e.anchor == EnemyAnchor::FreeFlying {
                        e.x = e.x.clamp(60.0 + e.radius, screen_w - 60.0 - e.radius);
                        e.vx = 0.0;
                        if self.current_phase == StagePhase::AscendUp {
                            e.vy = 120.0;
                        } else {
                            e.vy = -120.0;
                        }
                    }
                }
                // 2. Retirar obstáculos horizontales que no encajan en el pozo vertical
                self.obstacles.retain(|o| {
                    o.destructible && o.x >= 65.0 && o.x <= screen_w - 65.0
                });
            }
            StagePhase::HorizontalRight => {
                // Saliendo a corredor horizontal (azoteas / autopista aérea / túnel interior):
                // 1. Desactivar torretas de pared lateral para que no queden flotando en el aire
                for e in enemies.iter_mut() {
                    if e.anchor == EnemyAnchor::LeftWall || e.anchor == EnemyAnchor::RightWall {
                        e.active = false;
                    } else if e.anchor == EnemyAnchor::FreeFlying {
                        e.y = e.y.clamp(56.0 + e.radius, screen_h - 52.0 - e.radius);
                        e.vx = -150.0;
                        e.vy = 0.0;
                    }
                }
                // 2. Retirar obstáculos verticales residuales
                self.obstacles.retain(|o| {
                    o.destructible && o.y >= 80.0 && o.y <= screen_h - 80.0
                });
            }
            StagePhase::BossEncounter => {
                self.obstacles.clear();
            }
        }
    }

    fn update_phase_and_trajectory(&mut self) {
        if self.boss_spawned || self.stage_progress >= 1.0 {
            self.current_phase = StagePhase::BossEncounter;
            self.target_vx = 0.0;
            self.target_vy = 0.0;
            return;
        }

        let speed = self.config.scroll_speed;
        let p = self.stage_progress;

        match self.current_stage {
            1 => {
                // Stage 1: Calle baja -> Ascenso por rascacielos -> Techos -> Descenso a fosa -> Cámara Titan-01
                if p < 0.28 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.52 {
                    self.current_phase = StagePhase::AscendUp;
                    self.target_vx = 0.0;
                    self.target_vy = -speed;
                } else if p < 0.68 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.05;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::DescendDown;
                    self.target_vx = 0.0;
                    self.target_vy = speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.20;
                    self.target_vy = 0.0;
                }
            }
            2 => {
                // Stage 2: Complejo exterior -> Descenso a fundiciones subterráneas -> Cámara del reactor
                if p < 0.38 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::DescendDown;
                    self.target_vx = 0.0;
                    self.target_vy = speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.15;
                    self.target_vy = 0.0;
                }
            }
            3 => {
                // Stage 3: Cañones de óxido -> Ascenso por chimenea de excavadora -> Cámara interior
                if p < 0.45 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::AscendUp;
                    self.target_vx = 0.0;
                    self.target_vy = -speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.15;
                    self.target_vy = 0.0;
                }
            }
            4 => {
                // Stage 4: Superficie de hielo -> Inmersión en fosa hidrotermal -> Cámara Abadox
                if p < 0.45 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::DescendDown;
                    self.target_vx = 0.0;
                    self.target_vy = speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.15;
                    self.target_vy = 0.0;
                }
            }
            5 => {
                // Stage 5: Carrera orbital -> Slingshot ascendente por torre Dyson -> Bastión solar
                if p < 0.48 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::AscendUp;
                    self.target_vx = 0.0;
                    self.target_vy = -speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.15;
                    self.target_vy = 0.0;
                }
            }
            6 => {
                // Stage 6: Costas de metano -> Ascenso monumental por elevador espacial -> Dársena orbital
                if p < 0.40 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::AscendUp;
                    self.target_vx = 0.0;
                    self.target_vy = -speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.15;
                    self.target_vy = 0.0;
                }
            }
            7 => {
                // Stage 7: Cinturón de asteroides -> Trinchera del Dreadnought -> Hangar de mando
                if p < 0.45 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::DescendDown;
                    self.target_vx = 0.0;
                    self.target_vy = speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.15;
                    self.target_vy = 0.0;
                }
            }
            _ => {
                // Stage 8: Vector cuántico -> Fosa gravitacional -> Ascenso singularidad -> Núcleo Overmind
                if p < 0.35 {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed;
                    self.target_vy = 0.0;
                } else if p < 0.58 {
                    self.current_phase = StagePhase::DescendDown;
                    self.target_vx = 0.0;
                    self.target_vy = speed;
                } else if p < 0.78 {
                    self.current_phase = StagePhase::AscendUp;
                    self.target_vx = 0.0;
                    self.target_vy = -speed;
                } else {
                    self.current_phase = StagePhase::HorizontalRight;
                    self.target_vx = speed * 1.20;
                    self.target_vy = 0.0;
                }
            }
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
            StagePhase::HorizontalRight => {
                if self.inside_chamber {
                    (36.0, screen_w - 36.0, 52.0, screen_h - 52.0)
                } else {
                    (36.0, screen_w - 36.0, 40.0, screen_h - 52.0)
                }
            }
            StagePhase::AscendUp | StagePhase::DescendDown => {
                // Pozos verticales: delimitados por las paredes laterales de 56px
                (60.0, screen_w - 60.0, 32.0, screen_h - 32.0)
            }
            StagePhase::BossEncounter => (36.0, screen_w - 36.0, 44.0, screen_h - 44.0),
        }
    }

    fn spawn_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        self.wave_index += 1;
        if self.current_stage == 1 {
            self.spawn_stage1_multidirectional_wave(screen_w, screen_h, enemies);
            return;
        }
        self.spawn_planetary_wave(screen_w, screen_h, enemies);
    }

    /// Trayecto multifase de Stage 1 inspirado fielmente en Final Mission y Abadox
    fn spawn_stage1_multidirectional_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let spawn_x_right = screen_w + 30.0;
        let w_idx = self.wave_index;

        match self.current_phase {
            StagePhase::HorizontalRight => {
                if self.stage_progress < 0.28 {
                    // === SECCIÓN 1: Lower ruined street & broken skylines (Vuelo horizontal inicial, cielo abierto) ===
                    // Coherencia física: Las defensas terrestres van exclusivamente en la autopista/suelo inferior (screen_h - 82.0)
                    // Las amenazas aéreas en el cielo son cazas, drones patrulla y avispas kamikaze.
                    let step = w_idx % 8;
                    match step {
                        0 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.40, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 65.0, screen_h * 0.60, EnemyType::PatrolDrone));
                        }
                        1 => {
                            for i in 0..4 {
                                let y = screen_h * 0.25 + (i as f32 * 45.0);
                                enemies.push(Enemy::new(spawn_x_right + (i as f32 * 45.0), y, EnemyType::KamikazeWasp));
                            }
                        }
                        2 => {
                            // Carrier #1: Dron transportador dorado con Vulcan Lv2 ('P') accesible en vuelo medio
                            let item = self.next_stage_item();
                            enemies.push(Enemy::new_carrier(spawn_x_right, screen_h * 0.45, item));
                            enemies.push(Enemy::new(spawn_x_right + 80.0, screen_h * 0.62, EnemyType::PatrolDrone));
                        }
                        3 => {
                            // Batería antiaérea asentada firmemente sobre el asfalto de la autopista derruida
                            enemies.push(Enemy::new_anchored(spawn_x_right, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 65.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                            enemies.push(Enemy::new(spawn_x_right + 30.0, screen_h * 0.35, EnemyType::KamikazeWasp));
                        }
                        4 => {
                            for i in 0..3 {
                                let y = screen_h * 0.32 + (i as f32 * 60.0);
                                enemies.push(Enemy::new(spawn_x_right + (i as f32 * 40.0), y, EnemyType::PatrolDrone));
                            }
                        }
                        5 => {
                            // Torreta antiaérea en parapeto de hormigón inferior + cazas aéreos de apoyo
                            enemies.push(Enemy::new_anchored(spawn_x_right + 50.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.30, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 80.0, screen_h * 0.52, EnemyType::PatrolDrone));
                        }
                        6 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.40, EnemyType::StealthStriker));
                            enemies.push(Enemy::new(spawn_x_right + 70.0, screen_h * 0.65, EnemyType::PatrolDrone));
                        }
                        _ => {
                            // Torreta antiaérea en suelo + escolta aérea en abanico
                            enemies.push(Enemy::new_anchored(spawn_x_right + 60.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.28, EnemyType::KamikazeWasp));
                            enemies.push(Enemy::new(spawn_x_right + 90.0, screen_h * 0.50, EnemyType::PatrolDrone));
                        }
                    }
                } else if self.stage_progress < 0.72 {
                    // === SECCIÓN 3: Rooftops & Skyscraper Summits (Techos de rascacielos y antenas, cielo abierto) ===
                    let step = w_idx % 6;
                    match step {
                        0 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.30, EnemyType::StealthStriker));
                            enemies.push(Enemy::new(spawn_x_right + 70.0, screen_h * 0.60, EnemyType::PatrolDrone));
                        }
                        1 => {
                            // Carrier #5: Dron contenedor dorado sobre los techos (Homing 'M')
                            let item = self.next_stage_item();
                            enemies.push(Enemy::new_carrier(spawn_x_right, screen_h * 0.42, item));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 70.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                            enemies.push(Enemy::new(spawn_x_right + 40.0, screen_h * 0.26, EnemyType::PatrolDrone));
                        }
                        2 => {
                            // Cañonera aérea pesada CyberCrab + torreta antiaérea en azotea inferior
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.50, EnemyType::CyberCrab));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 60.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                        }
                        3 => {
                            for i in 0..3 {
                                let y = screen_h * 0.22 + (i as f32 * 55.0);
                                enemies.push(Enemy::new(spawn_x_right + (i as f32 * 50.0), y, EnemyType::KamikazeWasp));
                            }
                        }
                        4 => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.35, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h * 0.65, EnemyType::PatrolDrone));
                        }
                        _ => {
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.42, EnemyType::StealthStriker));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 70.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                        }
                    }
                } else {
                    // === SECCIÓN 5: Interior de la Fortaleza TITAN-01 (Cámara reforzada con techo y suelo blindados) ===
                    // Aquí, al ser túnel/cámara interior (inside_chamber), sí existen mamparas superiores de techo y suelo.
                    let step = w_idx % 4;
                    match step {
                        0 => {
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
                            // Fuego cruzado letal: Torreta en techo blindado + Torreta en suelo de acero + Cañonera central
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.50, EnemyType::CyberCrab));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 60.0, 78.0, EnemyType::LaserTurret, EnemyAnchor::Ceiling));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 60.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                        }
                        _ => {
                            // Carrier #7: Suministro crítico pre-Boss (Shield 'H' - Nano-Med)
                            let item = self.next_stage_item();
                            enemies.push(Enemy::new_carrier(spawn_x_right, screen_h * 0.46, item));
                            enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h * 0.28, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h * 0.68, EnemyType::PatrolDrone));
                        }
                    }
                }
            }
            StagePhase::AscendUp => {
                // === SECCIÓN 2: Vertical ASCENT (scrolling UP) through Skyscraper Shaft ===
                // Todas las torretas se montan exclusivamente en paredes laterales izquierda (52px) y derecha (screen_w - 52px)
                let spawn_y_top = -35.0;
                let left_wall_x = 52.0;
                let right_wall_x = screen_w - 52.0;
                let step = w_idx % 6;

                match step {
                    0 => {
                        let mut t_left = Enemy::new_anchored(left_wall_x, spawn_y_top, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        let mut t_right = Enemy::new_anchored(right_wall_x, spawn_y_top - 60.0, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(t_left);
                        enemies.push(t_right);
                    }
                    1 => {
                        for i in 0..4 {
                            let x = screen_w * 0.30 + (i as f32 * 50.0);
                            let mut wasp = Enemy::new(x, spawn_y_top - (i as f32 * 45.0), EnemyType::KamikazeWasp);
                            wasp.vx = 0.0;
                            wasp.vy = 180.0;
                            enemies.push(wasp);
                        }
                    }
                    2 => {
                        let mut leech1 = Enemy::new(screen_w * 0.38, spawn_y_top, EnemyType::AsteroidLeech);
                        leech1.vx = 0.0;
                        leech1.vy = 140.0;
                        let mut leech2 = Enemy::new(screen_w * 0.62, spawn_y_top - 50.0, EnemyType::AsteroidLeech);
                        leech2.vx = 0.0;
                        leech2.vy = 140.0;
                        enemies.push(leech1);
                        enemies.push(leech2);
                    }
                    3 => {
                        // Carrier #4: Dron contenedor que desciende suavemente por el centro del pozo (Spread 'S')
                        let item = self.next_stage_item();
                        let mut carrier = Enemy::new_carrier(screen_w * 0.50, spawn_y_top - 30.0, item);
                        carrier.vx = 0.0;
                        carrier.vy = 65.0;
                        let mut t_right = Enemy::new_anchored(right_wall_x, spawn_y_top - 40.0, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(carrier);
                        enemies.push(t_right);
                    }
                    4 => {
                        let mut crab = Enemy::new(screen_w * 0.50, spawn_y_top, EnemyType::CyberCrab);
                        crab.vx = 0.0;
                        crab.vy = 110.0;
                        enemies.push(crab);
                    }
                    _ => {
                        let mut striker = Enemy::new(screen_w * 0.45, spawn_y_top, EnemyType::StealthStriker);
                        striker.vx = 0.0;
                        striker.vy = 160.0;
                        let mut t_left = Enemy::new_anchored(left_wall_x, spawn_y_top - 40.0, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        enemies.push(striker);
                        enemies.push(t_left);
                    }
                }
            }
            StagePhase::DescendDown => {
                // === SECCIÓN 4: Vertical DESCENT (scrolling DOWN) into Subterranean Foundry ===
                // Todas las torretas se montan en paredes laterales izquierda (52px) y derecha (screen_w - 52px)
                let spawn_y_bottom = screen_h + 35.0;
                let left_wall_x = 52.0;
                let right_wall_x = screen_w - 52.0;
                let step = w_idx % 6;

                match step {
                    0 => {
                        let mut t_left = Enemy::new_anchored(left_wall_x, spawn_y_bottom, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        let mut t_right = Enemy::new_anchored(right_wall_x, spawn_y_bottom + 50.0, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(t_left);
                        enemies.push(t_right);
                    }
                    1 => {
                        for i in 0..4 {
                            let x = screen_w * 0.28 + (i as f32 * 55.0);
                            let mut wasp = Enemy::new(x, spawn_y_bottom + (i as f32 * 45.0), EnemyType::KamikazeWasp);
                            wasp.vx = 0.0;
                            wasp.vy = -170.0;
                            enemies.push(wasp);
                        }
                    }
                    2 => {
                        // Carrier #6: Dron transportador que asciende por el centro del pozo (Bomb 'B')
                        let item = self.next_stage_item();
                        let mut carrier = Enemy::new_carrier(screen_w * 0.48, spawn_y_bottom + 30.0, item);
                        carrier.vx = 0.0;
                        carrier.vy = -65.0;
                        let mut leech = Enemy::new(screen_w * 0.68, spawn_y_bottom + 40.0, EnemyType::AsteroidLeech);
                        leech.vx = 0.0;
                        leech.vy = -150.0;
                        enemies.push(carrier);
                        enemies.push(leech);
                    }
                    3 => {
                        let mut leech = Enemy::new(screen_w * 0.50, spawn_y_bottom, EnemyType::AsteroidLeech);
                        leech.vx = 0.0;
                        leech.vy = -140.0;
                        enemies.push(leech);
                    }
                    4 => {
                        let mut striker = Enemy::new(screen_w * 0.50, spawn_y_bottom, EnemyType::StealthStriker);
                        striker.vx = 0.0;
                        striker.vy = -190.0;
                        let mut t_left = Enemy::new_anchored(left_wall_x, spawn_y_bottom + 60.0, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        enemies.push(striker);
                        enemies.push(t_left);
                    }
                    _ => {
                        let mut crab = Enemy::new(screen_w * 0.50, spawn_y_bottom, EnemyType::CyberCrab);
                        crab.vx = 0.0;
                        crab.vy = -110.0;
                        enemies.push(crab);
                    }
                }
            }
            StagePhase::BossEncounter => {}
        }
    }

    /// Oleadas para escenarios planetarios (Fases 2 a 8) con soporte multidireccional
    fn spawn_planetary_wave(&mut self, screen_w: f32, screen_h: f32, enemies: &mut Vec<Enemy>) {
        let spawn_x_right = screen_w + 30.0;
        let w_idx = self.wave_index;

        match self.current_phase {
            StagePhase::HorizontalRight => {
                let step = w_idx % 6;
                match step {
                    0 => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.35, EnemyType::PatrolDrone));
                        enemies.push(Enemy::new(spawn_x_right + 60.0, screen_h * 0.55, EnemyType::PatrolDrone));
                    }
                    1 => {
                        for i in 0..4 {
                            let y = screen_h * 0.25 + (i as f32 * 100.0);
                            enemies.push(Enemy::new(spawn_x_right + (i as f32 * 45.0), y, EnemyType::KamikazeWasp));
                        }
                    }
                    2 => {
                        // Suministro ordenado de power-ups e ítems según progresión arcade
                        let item = self.next_stage_item();
                        enemies.push(Enemy::new_carrier(spawn_x_right, screen_h * 0.45, item));
                        enemies.push(Enemy::new(spawn_x_right + 70.0, screen_h * 0.65, EnemyType::PatrolDrone));
                    }
                    3 => {
                        if self.inside_chamber {
                            // Túnel/cámara interior: Torretas en techo reforzado y suelo blindado
                            enemies.push(Enemy::new_anchored(spawn_x_right, 78.0, EnemyType::LaserTurret, EnemyAnchor::Ceiling));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 70.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                        } else {
                            // Cielo abierto planetario: Amenaza aérea + Batería terrestre en suelo
                            enemies.push(Enemy::new(spawn_x_right, screen_h * 0.30, EnemyType::PatrolDrone));
                            enemies.push(Enemy::new_anchored(spawn_x_right + 70.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                        }
                    }
                    4 => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.50, EnemyType::CyberCrab));
                        enemies.push(Enemy::new(spawn_x_right + 80.0, screen_h * 0.35, EnemyType::StealthStriker));
                    }
                    _ => {
                        enemies.push(Enemy::new(spawn_x_right, screen_h * 0.40, EnemyType::PatrolDrone));
                        enemies.push(Enemy::new_anchored(spawn_x_right + 50.0, screen_h - 82.0, EnemyType::LaserTurret, EnemyAnchor::Floor));
                    }
                }
            }
            StagePhase::AscendUp => {
                let spawn_y_top = -35.0;
                let step = w_idx % 6;
                match step {
                    0 => {
                        let mut e1 = Enemy::new(screen_w * 0.35, spawn_y_top, EnemyType::PatrolDrone);
                        e1.vx = 0.0;
                        e1.vy = 150.0;
                        let mut e2 = Enemy::new(screen_w * 0.65, spawn_y_top - 40.0, EnemyType::AsteroidLeech);
                        e2.vx = 0.0;
                        e2.vy = 150.0;
                        enemies.push(e1);
                        enemies.push(e2);
                    }
                    1 => {
                        let mut t_left = Enemy::new_anchored(52.0, spawn_y_top, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        let mut t_right = Enemy::new_anchored(screen_w - 52.0, spawn_y_top - 50.0, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(t_left);
                        enemies.push(t_right);
                    }
                    2 => {
                        let item = self.next_stage_item();
                        let mut carrier = Enemy::new_carrier(screen_w * 0.50, spawn_y_top - 30.0, item);
                        carrier.vx = 0.0;
                        carrier.vy = 75.0;
                        enemies.push(carrier);
                    }
                    3 => {
                        for i in 0..3 {
                            let x = screen_w * 0.32 + (i as f32 * 60.0);
                            let mut wasp = Enemy::new(x, spawn_y_top - (i as f32 * 40.0), EnemyType::KamikazeWasp);
                            wasp.vx = 0.0;
                            wasp.vy = 170.0;
                            enemies.push(wasp);
                        }
                    }
                    4 => {
                        let mut t_right = Enemy::new_anchored(screen_w - 52.0, spawn_y_top, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        let mut crab = Enemy::new(screen_w * 0.40, spawn_y_top - 30.0, EnemyType::CyberCrab);
                        crab.vx = 0.0;
                        crab.vy = 120.0;
                        enemies.push(t_right);
                        enemies.push(crab);
                    }
                    _ => {
                        let mut t_left = Enemy::new_anchored(52.0, spawn_y_top, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        let mut striker = Enemy::new(screen_w * 0.60, spawn_y_top - 30.0, EnemyType::StealthStriker);
                        striker.vx = 0.0;
                        striker.vy = 150.0;
                        enemies.push(t_left);
                        enemies.push(striker);
                    }
                }
            }
            StagePhase::DescendDown => {
                let spawn_y_bottom = screen_h + 35.0;
                let step = w_idx % 6;
                match step {
                    0 => {
                        let mut e1 = Enemy::new(screen_w * 0.40, spawn_y_bottom, EnemyType::KamikazeWasp);
                        e1.vx = 0.0;
                        e1.vy = -160.0;
                        let mut e2 = Enemy::new(screen_w * 0.60, spawn_y_bottom + 40.0, EnemyType::CyberCrab);
                        e2.vx = 0.0;
                        e2.vy = -110.0;
                        enemies.push(e1);
                        enemies.push(e2);
                    }
                    1 => {
                        let mut t_left = Enemy::new_anchored(52.0, spawn_y_bottom, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        let mut t_right = Enemy::new_anchored(screen_w - 52.0, spawn_y_bottom + 50.0, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        enemies.push(t_left);
                        enemies.push(t_right);
                    }
                    2 => {
                        let item = self.next_stage_item();
                        let mut carrier = Enemy::new_carrier(screen_w * 0.50, spawn_y_bottom + 30.0, item);
                        carrier.vx = 0.0;
                        carrier.vy = -75.0;
                        enemies.push(carrier);
                    }
                    3 => {
                        let mut leech = Enemy::new(screen_w * 0.50, spawn_y_bottom, EnemyType::AsteroidLeech);
                        leech.vx = 0.0;
                        leech.vy = -140.0;
                        enemies.push(leech);
                    }
                    4 => {
                        let mut t_left = Enemy::new_anchored(52.0, spawn_y_bottom, EnemyType::LaserTurret, EnemyAnchor::LeftWall);
                        t_left.aim_angle = 0.0;
                        let mut striker = Enemy::new(screen_w * 0.55, spawn_y_bottom + 40.0, EnemyType::StealthStriker);
                        striker.vx = 0.0;
                        striker.vy = -160.0;
                        enemies.push(t_left);
                        enemies.push(striker);
                    }
                    _ => {
                        let mut t_right = Enemy::new_anchored(screen_w - 52.0, spawn_y_bottom, EnemyType::LaserTurret, EnemyAnchor::RightWall);
                        t_right.aim_angle = std::f32::consts::PI;
                        let mut wasp = Enemy::new(screen_w * 0.38, spawn_y_bottom + 30.0, EnemyType::KamikazeWasp);
                        wasp.vx = 0.0;
                        wasp.vy = -170.0;
                        enemies.push(t_right);
                        enemies.push(wasp);
                    }
                }
            }
            StagePhase::BossEncounter => {}
        }
    }

    /// Genera objetos físicos y estructuras del escenario con coherencia física:
    /// - En cielo abierto: apoyados en la autopista/suelo o vigas transversales Warren suspendidas
    /// - En cámaras interiores: prensas hidráulicas, mamparas de techo blindado y crisoles
    /// - En pozos verticales: vigas cantilever ancladas a las paredes laterales o pasarelas centrales
    fn spawn_stage_obstacle(&mut self, screen_w: f32, screen_h: f32, enemies: &[Enemy]) {
        let style = match self.current_stage {
            1 => ObstacleStyle::SteelGirder,
            2 => ObstacleStyle::SmeltingPress,
            3 => ObstacleStyle::MarsCanyonRock,
            4 => ObstacleStyle::CryoIcePillar,
            5 => ObstacleStyle::SolarShield,
            6 => ObstacleStyle::MethaneTank,
            7 => ObstacleStyle::ArmadaBulkhead,
            _ => ObstacleStyle::QuantumMonolith,
        };

        let pattern = (self.distance_traveled as usize / 180) % 4;

        match self.current_phase {
            StagePhase::HorizontalRight => {
                let sx = screen_w + 90.0;

                // Evitar solapamiento con torretas ancladas existentes en techo o suelo
                let overlaps_turret = enemies.iter().any(|e| {
                    e.active && e.is_anchored() && (e.x - sx).abs() < 75.0
                });
                if overlaps_turret {
                    return;
                }

                if !self.inside_chamber {
                    // === ZONA EXTERIOR / CIELO ABIERTO ===
                    // Físicamente coherente: estructuras apoyadas en el suelo/autopista o vigas puente suspendidas
                    match pattern {
                        0 => {
                            // Puente / Viga Warren transversal suspendida (ObstacleStyle::SteelGirder)
                            // A media altura, dejando amplio paso de vuelo arriba y abajo
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: screen_h * 0.38,
                                width: 92.0,
                                height: 32.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                        }
                        1 => {
                            // Bunker terrestre / Pilar de autopista derruida elevado desde el suelo
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: screen_h - 58.0,
                                width: 68.0,
                                height: 56.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                        }
                        2 => {
                            // Plataforma flotante / Barcaza tecnológica de suministros (destruible con disparos)
                            let py = if (self.distance_traveled as usize / 360) % 2 == 0 {
                                screen_h * 0.42
                            } else {
                                screen_h * 0.60
                            };
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: py,
                                width: 88.0,
                                height: 32.0,
                                style,
                                destructible: true,
                                health: 180.0,
                                max_health: 180.0,
                                active: true,
                            });
                        }
                        _ => {
                            // Parapeto de hormigón reforzado en tierra con torre de comunicaciones
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: screen_h - 68.0,
                                width: 60.0,
                                height: 72.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                        }
                    }
                } else {
                    // === CÁMARA INTERIOR / TÚNEL REFORZADO ===
                    // Techos y suelos de acero blindado con compuertas y prensas industriales
                    match pattern {
                        0 => {
                            // Prensa hidráulica / Mampara superior descendente desde el techo blindado
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: 52.0,
                                width: 56.0,
                                height: 50.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                        }
                        1 => {
                            // Crisol / Prensas de fundición o bunker inferior
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: screen_h - 56.0,
                                width: 60.0,
                                height: 52.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                        }
                        2 => {
                            // Nodo de blindaje o generador de campo cuántico central (destruible)
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: screen_h * 0.50,
                                width: 84.0,
                                height: 36.0,
                                style,
                                destructible: true,
                                health: 180.0,
                                max_health: 180.0,
                                active: true,
                            });
                        }
                        _ => {
                            // Compuerta de esclusa presurizada (estrechamiento superior e inferior)
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: 50.0,
                                width: 48.0,
                                height: 46.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                            self.obstacles.push(StageObstacle {
                                x: sx,
                                y: screen_h - 52.0,
                                width: 48.0,
                                height: 46.0,
                                style,
                                destructible: false,
                                health: 999.0,
                                max_health: 999.0,
                                active: true,
                            });
                        }
                    }
                }
            }
            StagePhase::AscendUp => {
                // El jugador asciende: las vigas entran por arriba y bajan
                let sy = -50.0;
                let (ox, ow, destructible) = match pattern % 3 {
                    0 => (80.0, 96.0, false),            // Viga cantilever anclada a pared izquierda
                    1 => (screen_w - 80.0, 96.0, false), // Viga cantilever anclada a pared derecha
                    _ => (screen_w * 0.5, 100.0, true),  // Pasarela central de mantenimiento destructible
                };

                let overlaps_turret = enemies.iter().any(|e| {
                    e.active && e.is_anchored() && (e.x - ox).abs() < (ow * 0.5 + 28.0) && (e.y - sy).abs() < 70.0
                });
                if overlaps_turret {
                    return;
                }

                self.obstacles.push(StageObstacle {
                    x: ox,
                    y: sy,
                    width: ow,
                    height: 30.0,
                    style,
                    destructible,
                    health: 160.0,
                    max_health: 160.0,
                    active: true,
                });
            }
            StagePhase::DescendDown => {
                // El jugador desciende: las vigas entran por abajo y suben
                let sy = screen_h + 50.0;
                let (ox, ow, destructible) = match pattern % 3 {
                    0 => (80.0, 96.0, false),            // Viga cantilever anclada a pared izquierda
                    1 => (screen_w - 80.0, 96.0, false), // Viga cantilever anclada a pared derecha
                    _ => (screen_w * 0.5, 100.0, true),  // Pasarela central de mantenimiento destructible
                };

                let overlaps_turret = enemies.iter().any(|e| {
                    e.active && e.is_anchored() && (e.x - ox).abs() < (ow * 0.5 + 28.0) && (e.y - sy).abs() < 70.0
                });
                if overlaps_turret {
                    return;
                }

                self.obstacles.push(StageObstacle {
                    x: ox,
                    y: sy,
                    width: ow,
                    height: 30.0,
                    style,
                    destructible,
                    health: 160.0,
                    max_health: 160.0,
                    active: true,
                });
            }
            StagePhase::BossEncounter => {}
        }
    }
}
