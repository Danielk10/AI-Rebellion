//! Módulo de Enemigos Comunes y Cápsulas de Mejoras (Items)
//! Diseñado con la estética de Final Mission, Abadox y la Rebelión de la IA:
//! Sintéticos humanoides de combate, plantas biomecánicas y drones depredadores.

use crate::bullet::{Bullet, BulletOwner, BulletType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyType {
    // Sintéticos Humanoides de la IA
    SynthKatana,         // Sintético bípedo con katana de energía y dash táctico
    SynthRifle,          // Sintético bípedo francotirador con railgun
    
    // Ciber-Plantas Biomecánicas (Abadox + IA de terraformación)
    BioPlantVine,        // Zarcillo ondulante con espinas y savia luminosa
    BioPlantSporePod,    // Torreta de esporas que se dilata ("respira")
    BioPlantFlowerTrap,  // Flor trampa con pétalos navaja y estambre láser
    
    // Drones y Leviatanes
    PredatoryDrone,      // Dron interceptor con alas en flecha invertida
    BioMechLeviathan,    // Leviatán blindado con pinzas trituradoras
    ItemCarrier,         // Dron transportador blindado con cápsula de suministro (estilo Final Mission)

    // Mapeos retrocompatibles para las oleadas de level.rs:
    PatrolDrone,         // Mapea a PredatoryDrone
    KamikazeWasp,        // Mapea a PredatoryDrone / BioPlantSporePod
    LaserTurret,         // Mapea a BioPlantFlowerTrap / SporePod
    CyberCrab,           // Mapea a BioMechLeviathan
    AsteroidLeech,       // Mapea a BioPlantVine
    StealthStriker,      // Mapea a SynthKatana / SynthRifle
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ItemType {
    WeaponVulcan,  // 'P': Potencia de Fuego / Power-Up Vulcan
    WeaponLaser,   // 'L': Rayo Láser Penetrante
    WeaponSpread,  // 'S': Escopeta de Plasma en Abanico
    WeaponHoming,  // 'M': Misiles Teledirigidos
    Shield,        // 'H': Kit Nano-Médico (+40 HP)
    Bomb,          // 'B': Recarga de Bomba Especial EMP (+1 Smart Bomb)
    ExtraLife,     // '1UP': Vida Extra
}

#[derive(Clone, Debug)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub item_type: ItemType,
    pub active: bool,
    pub radius: f32,
    pub time_alive: f32,
}

impl Item {
    pub fn new(x: f32, y: f32, item_type: ItemType) -> Self {
        Self {
            x,
            y,
            vx: -35.0,
            vy: 0.0,
            item_type,
            active: true,
            radius: 18.0,
            time_alive: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        self.time_alive += dt;
        self.x += self.vx * dt;
        // Flotación sinusoidal suave estilo arcade clásico
        self.y += (self.time_alive * 2.8).sin() * 30.0 * dt;

        if self.x < -50.0 {
            self.active = false;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub health: f32,
    pub max_health: f32,
    pub enemy_type: EnemyType,
    pub fire_timer: f32,
    pub active: bool,
    pub time_alive: f32,
    pub aim_angle: f32,   // Orientación del cañón o sensor hacia el jugador
    pub is_ceiling: bool, // Montada en techo o invertida
}

impl Enemy {
    pub fn new(x: f32, y: f32, e_type: EnemyType) -> Self {
        let (hp, r, vx, vy) = match e_type {
            EnemyType::SynthKatana | EnemyType::StealthStriker => (75.0, 19.0, -170.0, 0.0),
            EnemyType::SynthRifle => (85.0, 20.0, -140.0, 0.0),
            EnemyType::BioPlantVine | EnemyType::AsteroidLeech => (90.0, 22.0, -100.0, 0.0),
            EnemyType::BioPlantSporePod => (130.0, 25.0, -80.0, 0.0),
            EnemyType::BioPlantFlowerTrap | EnemyType::LaserTurret => (140.0, 26.0, -90.0, 0.0),
            EnemyType::PredatoryDrone | EnemyType::PatrolDrone => (45.0, 18.0, -210.0, 0.0),
            EnemyType::KamikazeWasp => (30.0, 16.0, -250.0, 0.0),
            EnemyType::BioMechLeviathan | EnemyType::CyberCrab => (220.0, 32.0, -75.0, 25.0),
            EnemyType::ItemCarrier => (48.0, 22.0, -115.0, 0.0), // Dron contenedor dorado
        };

        let is_ceiling = y < 140.0;

        Self {
            x,
            y,
            vx,
            vy,
            radius: r,
            health: hp,
            max_health: hp,
            enemy_type: e_type,
            fire_timer: 0.8,
            active: true,
            time_alive: 0.0,
            aim_angle: std::f32::consts::PI, // Hacia la izquierda por defecto
            is_ceiling,
        }
    }

    pub fn take_damage(&mut self, dmg: f32) -> bool {
        self.health -= dmg;
        if self.health <= 0.0 {
            self.active = false;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32, bullets: &mut Vec<Bullet>) {
        if !self.active {
            return;
        }

        self.time_alive += dt;
        self.fire_timer -= dt;

        // Calcular ángulo continuo hacia el jugador (tracking continuo)
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        self.aim_angle = dy.atan2(dx);

        // Movimiento según patrón táctico
        match self.enemy_type {
            EnemyType::KamikazeWasp => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 5.0).sin() * 160.0 * dt;
            }
            EnemyType::PredatoryDrone | EnemyType::PatrolDrone => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 3.5).sin() * 70.0 * dt;
            }
            EnemyType::ItemCarrier => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 2.5).sin() * 45.0 * dt;
            }
            EnemyType::CyberCrab | EnemyType::BioMechLeviathan => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 2.2).cos() * 90.0 * dt;
            }
            EnemyType::LaserTurret | EnemyType::BioPlantFlowerTrap | EnemyType::BioPlantSporePod => {
                self.x += self.vx * dt;
            }
            EnemyType::SynthKatana | EnemyType::StealthStriker => {
                let dash = if ((self.time_alive * 2.0) as usize) % 2 == 0 { 1.5 } else { 0.7 };
                self.x += self.vx * dash * dt;
                self.y += self.vy * dt;
            }
            _ => {
                self.x += self.vx * dt;
                self.y += self.vy * dt;
            }
        }

        // Disparo enemigo dirigido (ItemCarrier no dispara)
        if self.enemy_type != EnemyType::ItemCarrier && self.fire_timer <= 0.0 && self.x > 40.0 && self.x < 1800.0 {
            self.fire_timer = match self.enemy_type {
                EnemyType::LaserTurret | EnemyType::BioPlantFlowerTrap => 1.7,
                EnemyType::CyberCrab | EnemyType::BioMechLeviathan => 2.0,
                EnemyType::StealthStriker | EnemyType::SynthRifle => 1.8,
                EnemyType::BioPlantSporePod => 2.2,
                _ => 2.5,
            };

            let b_speed = 370.0;
            let vx = self.aim_angle.cos() * b_speed;
            let vy = self.aim_angle.sin() * b_speed;

            let muzzle_dist = self.radius + 6.0;
            let spawn_x = self.x + self.aim_angle.cos() * muzzle_dist;
            let spawn_y = self.y + self.aim_angle.sin() * muzzle_dist;

            let b_type = match self.enemy_type {
                EnemyType::BioPlantSporePod | EnemyType::BioPlantVine => BulletType::BioAcid,
                _ => BulletType::EnemyPlasma,
            };

            let b_col = match self.enemy_type {
                EnemyType::BioPlantSporePod | EnemyType::BioPlantVine => 0xFF39FF14,
                EnemyType::SynthKatana | EnemyType::SynthRifle => 0xFFFF0055,
                _ => 0xFFFF2828,
            };

            bullets.push(Bullet::new(
                spawn_x,
                spawn_y,
                vx,
                vy,
                6.0,
                15.0,
                BulletOwner::Enemy,
                b_type,
                b_col,
            ));
        }

        if self.x < -80.0 {
            self.active = false;
        }
    }
}
