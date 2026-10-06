//! Módulo del Jugador: Soldado Humano con Traje Avanzado y Jetpack
//! Inspirado en Final Mission (Famicom/NES Japón) y Abadox de Natsume.
//! Cuenta con vuelo multidireccional, orientación dinámica, retroceso de disparo,
//! propulsor jetpack animado y 2 satélites orbitales de protección y asistencia táctica.

use crate::bullet::{Bullet, BulletOwner, BulletType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeaponType {
    Vulcan,
    Laser,
    Spread,
    Homing,
}

#[derive(Clone, Debug)]
pub struct Satellite {
    pub angle: f32,
    pub distance: f32,
    pub is_locked: bool,
    pub fire_cooldown: f32,
}

impl Satellite {
    pub fn new(initial_angle: f32) -> Self {
        Self {
            angle: initial_angle,
            distance: 46.0,
            is_locked: false,
            fire_cooldown: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, lock_requested: bool, target_angle: Option<f32>) {
        self.is_locked = lock_requested;
        if self.is_locked {
            if let Some(target) = target_angle {
                // Orientar suavemente hacia el ángulo deseado
                let diff = target - self.angle;
                self.angle += diff * (8.0 * dt).min(1.0);
            }
        } else {
            self.angle += 3.8 * dt; // Rotación continua estilo Final Mission
            if self.angle > std::f32::consts::PI * 2.0 {
                self.angle -= std::f32::consts::PI * 2.0;
            }
        }
        if self.fire_cooldown > 0.0 {
            self.fire_cooldown -= dt;
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub id: u8,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub speed: f32,
    pub health: f32,
    pub max_health: f32,
    pub lives: i32,
    pub score: u32,
    pub bombs: u8,
    pub weapon: WeaponType,
    pub weapon_power: u8, // 1..=3
    pub satellites: [Satellite; 2],
    pub invulnerable_timer: f32,
    pub fire_timer: f32,
    pub active: bool,
    pub color: u32,
    pub facing_right: bool,
    pub anim_timer: f32,
    pub jetpack_active: bool,
}

impl Player {
    pub fn new(id: u8, start_x: f32, start_y: f32) -> Self {
        let color = match id {
            0 => 0xFF00D2FF, // Azul cobalto / cian (P1 - Arnold)
            1 => 0xFFFF3344, // Rojo carmesí / rubí (P2 - Sigourney)
            2 => 0xFF00FF77, // Verde plasma / esmeralda (P3 - Jax)
            _ => 0xFFFFD700, // Dorado solar / titanio (P4 - Orion)
        };

        Self {
            id,
            x: start_x,
            y: start_y,
            vx: 0.0,
            vy: 0.0,
            speed: 460.0,
            health: 100.0,
            max_health: 100.0,
            lives: 3,
            score: 0,
            bombs: 3,
            weapon: WeaponType::Vulcan,
            weapon_power: 1,
            satellites: [Satellite::new(0.0), Satellite::new(std::f32::consts::PI)],
            invulnerable_timer: 2.0,
            fire_timer: 0.0,
            active: true,
            color,
            facing_right: true,
            anim_timer: 0.0,
            jetpack_active: true,
        }
    }

    /// Desplazamiento por arrastre táctil directo (Algoritmo de Jugador.java)
    pub fn apply_touch_drag(&mut self, dx: f32, dy: f32, screen_w: f32, screen_h: f32) {
        if !self.active {
            return;
        }

        self.x = (self.x + dx).clamp(36.0, screen_w - 36.0);
        self.y = (self.y + dy).clamp(36.0, screen_h - 36.0);

        if dx > 0.8 {
            self.facing_right = true;
        } else if dx < -0.8 {
            self.facing_right = false;
        }

        self.jetpack_active = dx.abs() > 0.2 || dy.abs() > 0.2;
    }

    pub fn update(
        &mut self,
        dt: f32,
        sat_lock: bool,
        target_angle: Option<f32>,
    ) {
        if !self.active {
            return;
        }

        self.anim_timer += dt;

        if self.invulnerable_timer > 0.0 {
            self.invulnerable_timer -= dt;
        }
        if self.fire_timer > 0.0 {
            self.fire_timer -= dt;
        }

        // Actualizar satélites orbitales
        for sat in self.satellites.iter_mut() {
            sat.update(dt, sat_lock, target_angle);
        }
    }

    pub fn fire(&mut self, bullets: &mut Vec<Bullet>) {
        if self.fire_timer > 0.0 || !self.active {
            return;
        }

        self.fire_timer = match self.weapon {
            WeaponType::Vulcan => 0.11,
            WeaponType::Laser => 0.20,
            WeaponType::Spread => 0.17,
            WeaponType::Homing => 0.26,
        };

        let owner = BulletOwner::Player(self.id);
        let dir = if self.facing_right { 1.0 } else { -1.0 };
        let gun_x = self.x + dir * 28.0;
        let gun_y = self.y - 2.0;

        // Disparo principal del rifle de asalto del soldado humano
        match self.weapon {
            WeaponType::Vulcan => {
                bullets.push(Bullet::new(gun_x, gun_y, dir * 950.0, 0.0, 6.0, 26.0, owner, BulletType::NormalVulcan, 0xFF00FFFF));
                if self.weapon_power >= 2 {
                    bullets.push(Bullet::new(gun_x, gun_y - 10.0, dir * 920.0, -90.0, 5.0, 22.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                    bullets.push(Bullet::new(gun_x, gun_y + 10.0, dir * 920.0, 90.0, 5.0, 22.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                }
            }
            WeaponType::Laser => {
                bullets.push(Bullet::new(gun_x, gun_y, dir * 1500.0, 0.0, 11.0, 70.0, owner, BulletType::LaserBeam, 0xFFFF4500));
            }
            WeaponType::Spread => {
                let count = if self.weapon_power >= 2 { 5 } else { 3 };
                let spread_step = 160.0;
                let start_vy = -((count - 1) as f32 * spread_step * 0.5);
                for i in 0..count {
                    let vy = start_vy + (i as f32 * spread_step);
                    bullets.push(Bullet::new(gun_x, gun_y, dir * 850.0, vy, 7.0, 32.0, owner, BulletType::SpreadWave, 0xFF7CFC00));
                }
            }
            WeaponType::Homing => {
                bullets.push(Bullet::new(gun_x, gun_y - 12.0, dir * 550.0, -180.0, 6.0, 42.0, owner, BulletType::HomingMissile, 0xFFFF1493));
                bullets.push(Bullet::new(gun_x, gun_y + 12.0, dir * 550.0, 180.0, 6.0, 42.0, owner, BulletType::HomingMissile, 0xFFFF1493));
            }
        }

        // Disparo de apoyo de los Satélites Orbitales
        for sat in self.satellites.iter_mut() {
            if sat.fire_cooldown <= 0.0 {
                sat.fire_cooldown = 0.15;
                let sx = self.x + sat.angle.cos() * sat.distance;
                let sy = self.y + sat.angle.sin() * sat.distance;
                let (vx, vy) = if sat.is_locked {
                    (sat.angle.cos() * 900.0, sat.angle.sin() * 900.0)
                } else {
                    (dir * 850.0, sat.angle.sin() * 400.0)
                };
                bullets.push(Bullet::new(sx, sy, vx, vy, 5.0, 19.0, BulletOwner::Satellite(self.id), BulletType::NormalVulcan, 0xFF00FFFF));
            }
        }
    }

    pub fn take_damage(&mut self, dmg: f32) -> bool {
        if self.invulnerable_timer > 0.0 || !self.active {
            return false;
        }

        self.health -= dmg;
        self.invulnerable_timer = 1.6;

        if self.health <= 0.0 {
            self.lives -= 1;
            if self.lives > 0 {
                self.health = self.max_health;
                self.weapon_power = 1.max(self.weapon_power.saturating_sub(1));
            } else {
                self.active = false;
            }
            true
        } else {
            false
        }
    }
}
