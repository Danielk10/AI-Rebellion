//! Módulo de Enemigos Comunes y Cápsulas de Mejoras (Items)
//! Diseñado con la estética biomecánica de Abadox y ciber-rebelión

use crate::bullet::{Bullet, BulletOwner, BulletType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EnemyType {
    PatrolDrone,
    KamikazeWasp,
    LaserTurret,
    CyberCrab,
    AsteroidLeech,
    StealthStriker,
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
}

impl Enemy {
    pub fn new(x: f32, y: f32, e_type: EnemyType) -> Self {
        let (hp, r, vx, vy) = match e_type {
            EnemyType::PatrolDrone => (35.0, 18.0, -180.0, 0.0),
            EnemyType::KamikazeWasp => (20.0, 14.0, -290.0, 0.0),
            EnemyType::LaserTurret => (90.0, 24.0, -110.0, 0.0),
            EnemyType::CyberCrab => (150.0, 28.0, -90.0, 30.0),
            EnemyType::AsteroidLeech => (75.0, 22.0, -140.0, 0.0),
            EnemyType::StealthStriker => (60.0, 19.0, -240.0, 0.0),
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
            fire_timer: 1.0,
            active: true,
            time_alive: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32, bullets: &mut Vec<Bullet>) {
        if !self.active {
            return;
        }

        self.time_alive += dt;
        self.fire_timer -= dt;

        // Movimiento según patrón
        match self.enemy_type {
            EnemyType::KamikazeWasp => {
                let dy = player_y - self.y;
                self.y += dy.signum() * 140.0 * dt;
                self.x += self.vx * dt;
            }
            EnemyType::PatrolDrone => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 4.0).sin() * 80.0 * dt;
            }
            EnemyType::CyberCrab => {
                self.x += self.vx * dt;
                self.y += (self.time_alive * 2.5).cos() * 95.0 * dt;
            }
            _ => {
                self.x += self.vx * dt;
                self.y += self.vy * dt;
            }
        }

        // Disparo enemigo
        if self.fire_timer <= 0.0 && self.x > 50.0 && self.x < 1800.0 {
            self.fire_timer = match self.enemy_type {
                EnemyType::LaserTurret => 1.8,
                EnemyType::CyberCrab => 2.2,
                _ => 2.6,
            };

            let dx = player_x - self.x;
            let dy = player_y - self.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let b_speed = 360.0;
            let vx = (dx / dist) * b_speed;
            let vy = (dy / dist) * b_speed;

            bullets.push(Bullet::new(
                self.x - 10.0,
                self.y,
                vx,
                vy,
                6.0,
                15.0,
                BulletOwner::Enemy,
                BulletType::EnemyPlasma,
                0xFFFF3030,
            ));
        }

        if self.x < -80.0 {
            self.active = false;
        }
    }
}
