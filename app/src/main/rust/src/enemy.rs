//! Módulo de Enemigos Comunes y Cápsulas de Mejoras (Items)
//! Diseñado con la estética de Final Mission y la Rebelión de la IA:
//! Autómatas rebeldes, torretas S-400 orientables, drones cruciformes y cápsulas bivalvas.

use crate::bullet::{Bullet, BulletOwner, BulletType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyType {
    PatrolDrone,    // Dron cruciforme centinela de la IA (+)
    KamikazeWasp,   // Dron cazador bivalvo (almeja) en trayectoria senoidal
    LaserTurret,    // Torreta S-400Phalanx orientable montada en tuberías o suelo
    CyberCrab,      // Androide / Mecha pesado de asalto
    AsteroidLeech,  // Mina magnética o sonda parasitaria
    StealthStriker, // Cañonera aérea militar subvertida
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ItemType {
    WeaponVulcan,
    WeaponLaser,
    WeaponSpread,
    WeaponHoming,
    Shield,
    Bomb,
    ExtraLife,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub item_type: ItemType,
    pub active: bool,
    pub radius: f32,
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
    pub aim_angle: f32,   // Orientación del cañón hacia el jugador (estilo Final Mission)
    pub is_ceiling: bool, // Montada en tubería del techo o invertida
}

impl Enemy {
    pub fn new(x: f32, y: f32, e_type: EnemyType) -> Self {
        let (hp, r, vx, vy) = match e_type {
            EnemyType::PatrolDrone => (35.0, 18.0, -180.0, 0.0),
            EnemyType::KamikazeWasp => (24.0, 15.0, -260.0, 0.0),
            EnemyType::LaserTurret => (95.0, 24.0, -110.0, 0.0),
            EnemyType::CyberCrab => (160.0, 28.0, -90.0, 30.0),
            EnemyType::AsteroidLeech => (75.0, 20.0, -140.0, 0.0),
            EnemyType::StealthStriker => (65.0, 20.0, -230.0, 0.0),
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

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32, bullets: &mut Vec<Bullet>) {
        if !self.active {
            return;
        }

        self.time_alive += dt;
        self.fire_timer -= dt;

        // Calcular ángulo continuo hacia el jugador (tracking estilo Final Mission)
        let dx = player_x - self.x;
        let dy = player_y - self.y;
        self.aim_angle = dy.atan2(dx);

        // Movimiento según patrón táctico
        match self.enemy_type {
            EnemyType::KamikazeWasp => {
                // Vuelo ondulatorio senoidal de enjambre (estilo capturas fm_08)
                self.x += self.vx * dt;
                self.y += (self.time_alive * 5.0).sin() * 160.0 * dt;
            }
            EnemyType::PatrolDrone => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 3.5).sin() * 70.0 * dt;
            }
            EnemyType::CyberCrab => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 2.2).cos() * 90.0 * dt;
            }
            EnemyType::LaserTurret => {
                // Sigue la velocidad de desplazamiento del escenario
                self.x += self.vx * dt;
            }
            _ => {
                self.x += self.vx * dt;
                self.y += self.vy * dt;
            }
        }

        // Disparo enemigo dirigido
        if self.fire_timer <= 0.0 && self.x > 40.0 && self.x < 1800.0 {
            self.fire_timer = match self.enemy_type {
                EnemyType::LaserTurret => 1.7,
                EnemyType::CyberCrab => 2.0,
                EnemyType::StealthStriker => 1.9,
                _ => 2.5,
            };

            let b_speed = 370.0;
            let vx = self.aim_angle.cos() * b_speed;
            let vy = self.aim_angle.sin() * b_speed;

            let muzzle_dist = self.radius + 6.0;
            let spawn_x = self.x + self.aim_angle.cos() * muzzle_dist;
            let spawn_y = self.y + self.aim_angle.sin() * muzzle_dist;

            bullets.push(Bullet::new(
                spawn_x,
                spawn_y,
                vx,
                vy,
                6.0,
                15.0,
                BulletOwner::Enemy,
                BulletType::EnemyPlasma,
                0xFFFF2828,
            ));
        }

        if self.x < -80.0 {
            self.active = false;
        }
    }
}
