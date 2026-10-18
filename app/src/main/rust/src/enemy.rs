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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnemyAnchor {
    FreeFlying,  // Drones, avispas voladoras, cazas, leviatán (espacio libre)
    Ceiling,     // Torreta/planta anclada rígidamente al techo (fase horizontal)
    Floor,       // Torreta/planta anclada rígidamente al suelo (fase horizontal)
    LeftWall,    // Torreta anclada rígidamente a la pared izquierda (fase vertical)
    RightWall,   // Torreta anclada rígidamente a la pared derecha (fase vertical)
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
            vx: -175.0,
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
        // Desplazamiento horizontal dinámico a través de la pantalla estilo Final Mission NES (vx = -175.0)
        self.x += self.vx * dt;
        // Suave ondulación vertical
        self.y += (self.time_alive * 3.5).sin() * 35.0 * dt;

        // Despawn definitivo al salir de la pantalla por la izquierda
        if self.x < -40.0 {
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
    pub is_ceiling: bool, // Montada en techo o invertida (retrocompatibilidad)
    pub anchor: EnemyAnchor, // Tipo de anclaje (FreeFlying, Ceiling, Floor, LeftWall, RightWall)
    pub item_drop: Option<ItemType>, // Ítem garantizado que suelta al ser destruido
}

impl Enemy {
    pub fn new(x: f32, y: f32, e_type: EnemyType) -> Self {
        let anchor = match e_type {
            EnemyType::LaserTurret | EnemyType::BioPlantFlowerTrap | EnemyType::BioPlantSporePod => {
                if y < 140.0 {
                    EnemyAnchor::Ceiling
                } else {
                    EnemyAnchor::Floor
                }
            }
            _ => EnemyAnchor::FreeFlying,
        };
        Self::new_anchored(x, y, e_type, anchor)
    }

    pub fn new_carrier(x: f32, y: f32, item: ItemType) -> Self {
        let mut e = Self::new(x, y, EnemyType::ItemCarrier);
        e.item_drop = Some(item);
        e
    }

    pub fn new_anchored(x: f32, y: f32, e_type: EnemyType, anchor: EnemyAnchor) -> Self {
        let (hp, r, mut vx, mut vy) = match e_type {
            EnemyType::SynthKatana | EnemyType::StealthStriker => (75.0, 19.0, -170.0, 0.0),
            EnemyType::SynthRifle => (85.0, 20.0, -140.0, 0.0),
            EnemyType::BioPlantVine | EnemyType::AsteroidLeech => (90.0, 22.0, -100.0, 0.0),
            EnemyType::BioPlantSporePod => (130.0, 25.0, 0.0, 0.0),
            EnemyType::BioPlantFlowerTrap | EnemyType::LaserTurret => (140.0, 26.0, 0.0, 0.0),
            EnemyType::PredatoryDrone | EnemyType::PatrolDrone => (45.0, 18.0, -210.0, 0.0),
            EnemyType::KamikazeWasp => (30.0, 16.0, -250.0, 0.0),
            EnemyType::BioMechLeviathan | EnemyType::CyberCrab => (220.0, 32.0, -75.0, 25.0),
            EnemyType::ItemCarrier => (48.0, 22.0, -95.0, 0.0), // Dron transportador dorado accesible
        };

        if anchor != EnemyAnchor::FreeFlying {
            vx = 0.0;
            vy = 0.0;
        }

        let is_ceiling = anchor == EnemyAnchor::Ceiling;
        let aim_angle = match anchor {
            EnemyAnchor::LeftWall => 0.0,
            EnemyAnchor::RightWall => std::f32::consts::PI,
            EnemyAnchor::Ceiling => std::f32::consts::FRAC_PI_2,
            EnemyAnchor::Floor => -std::f32::consts::FRAC_PI_2,
            EnemyAnchor::FreeFlying => std::f32::consts::PI,
        };

        let item_drop = if e_type == EnemyType::ItemCarrier {
            Some(ItemType::WeaponVulcan)
        } else {
            None
        };

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
            aim_angle,
            is_ceiling,
            anchor,
            item_drop,
        }
    }

    #[inline(always)]
    pub fn is_anchored(&self) -> bool {
        self.anchor != EnemyAnchor::FreeFlying
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

        // Movimiento según patrón táctico y tipo de anclaje:
        // Las torretas ancladas son desplazadas exclusivamente por el vector de scroll en game.rs.
        if !self.is_anchored() {
            let is_vertical_flight = self.vy.abs() > 20.0;

            match self.enemy_type {
                EnemyType::KamikazeWasp => {
                    if is_vertical_flight {
                        self.y += self.vy * dt;
                        self.x += self.vx * dt + (self.time_alive * 6.0).sin() * 140.0 * dt;
                    } else {
                        self.x += self.vx * dt;
                        self.y += self.vy * dt + (self.time_alive * 5.0).sin() * 160.0 * dt;
                    }
                }
                EnemyType::PredatoryDrone | EnemyType::PatrolDrone => {
                    if is_vertical_flight {
                        self.y += self.vy * dt;
                        self.x += self.vx * dt + (self.time_alive * 4.0).sin() * 70.0 * dt;
                    } else {
                        self.x += self.vx * dt;
                        self.y += self.vy * dt + (self.time_alive * 3.5).sin() * 70.0 * dt;
                    }
                }
                EnemyType::ItemCarrier => {
                    if is_vertical_flight {
                        self.y += self.vy * dt;
                        self.x += self.vx * dt + (self.time_alive * 3.0).sin() * 45.0 * dt;
                    } else {
                        self.x += self.vx * dt;
                        self.y += self.vy * dt + (self.time_alive * 2.5).sin() * 45.0 * dt;
                    }
                }
                EnemyType::CyberCrab | EnemyType::BioMechLeviathan => {
                    if is_vertical_flight {
                        self.y += self.vy * dt;
                        self.x += self.vx * dt + (self.time_alive * 2.5).cos() * 80.0 * dt;
                    } else {
                        self.x += self.vx * dt;
                        self.y += self.vy * dt + (self.time_alive * 2.2).cos() * 90.0 * dt;
                    }
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
        }

        // Disparo enemigo dirigido (ItemCarrier no dispara)
        if self.enemy_type != EnemyType::ItemCarrier && self.fire_timer <= 0.0 && self.x > 30.0 && self.x < 1800.0 {
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

        if self.x < -120.0 || self.x > 2600.0 || self.y < -140.0 || self.y > 2000.0 {
            self.active = false;
        }
    }
}
