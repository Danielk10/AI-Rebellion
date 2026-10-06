//! Módulo del Jugador y Mecánica de Satélites Orbitales (Inspirado en Final Mission)

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
            distance: 48.0,
            is_locked: false,
            fire_cooldown: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, lock_requested: bool) {
        self.is_locked = lock_requested;
        if !self.is_locked {
            self.angle += 3.5 * dt; // Rotación continua estilo Final Mission
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
}

impl Player {
    pub fn new(id: u8, start_x: f32, start_y: f32) -> Self {
        let color = match id {
            0 => 0xFF00D2FF, // Azul eléctrico (P1)
            1 => 0xFF3B3BFF, // Rojo cibernético (P2)
            2 => 0xFF3BFF3B, // Verde plasma (P3)
            _ => 0xFFFFD700, // Dorado solar (P4)
        };

        Self {
            id,
            x: start_x,
            y: start_y,
            vx: 0.0,
            vy: 0.0,
            speed: 420.0,
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
        }
    }

    pub fn update(&mut self, dt: f32, move_x: f32, move_y: f32, sat_lock: bool, screen_w: f32, screen_h: f32) {
        if !self.active {
            return;
        }

        // Movimiento suave con inercia controlada
        self.vx = move_x * self.speed;
        self.vy = move_y * self.speed;
        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Limitar dentro de la pantalla
        self.x = self.x.clamp(40.0, screen_w - 40.0);
        self.y = self.y.clamp(40.0, screen_h - 40.0);

        if self.invulnerable_timer > 0.0 {
            self.invulnerable_timer -= dt;
        }
        if self.fire_timer > 0.0 {
            self.fire_timer -= dt;
        }

        // Actualizar satélites
        for sat in self.satellites.iter_mut() {
            sat.update(dt, sat_lock);
        }
    }

    pub fn fire(&mut self, bullets: &mut Vec<Bullet>) {
        if self.fire_timer > 0.0 || !self.active {
            return;
        }

        self.fire_timer = match self.weapon {
            WeaponType::Vulcan => 0.12,
            WeaponType::Laser => 0.22,
            WeaponType::Spread => 0.18,
            WeaponType::Homing => 0.28,
        };

        let owner = BulletOwner::Player(self.id);

        // Disparo principal del caza
        match self.weapon {
            WeaponType::Vulcan => {
                bullets.push(Bullet::new(self.x + 25.0, self.y, 900.0, 0.0, 6.0, 25.0, owner, BulletType::NormalVulcan, 0xFF00FFFF));
                if self.weapon_power >= 2 {
                    bullets.push(Bullet::new(self.x + 20.0, self.y - 12.0, 880.0, -80.0, 5.0, 20.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                    bullets.push(Bullet::new(self.x + 20.0, self.y + 12.0, 880.0, 80.0, 5.0, 20.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                }
            }
            WeaponType::Laser => {
                bullets.push(Bullet::new(self.x + 35.0, self.y, 1400.0, 0.0, 10.0, 65.0, owner, BulletType::LaserBeam, 0xFFFF4500));
            }
            WeaponType::Spread => {
                let count = if self.weapon_power >= 2 { 5 } else { 3 };
                let spread_step = 160.0;
                let start_vy = -((count - 1) as f32 * spread_step * 0.5);
                for i in 0..count {
                    let vy = start_vy + (i as f32 * spread_step);
                    bullets.push(Bullet::new(self.x + 20.0, self.y, 820.0, vy, 7.0, 30.0, owner, BulletType::SpreadWave, 0xFF7CFC00));
                }
            }
            WeaponType::Homing => {
                bullets.push(Bullet::new(self.x + 20.0, self.y - 15.0, 500.0, -180.0, 6.0, 40.0, owner, BulletType::HomingMissile, 0xFFFF1493));
                bullets.push(Bullet::new(self.x + 20.0, self.y + 15.0, 500.0, 180.0, 6.0, 40.0, owner, BulletType::HomingMissile, 0xFFFF1493));
            }
        }

        // Disparo de apoyo de los Satélites Orbitales (Estilo Final Mission)
        for sat in self.satellites.iter_mut() {
            if sat.fire_cooldown <= 0.0 {
                sat.fire_cooldown = 0.16;
                let sx = self.x + sat.angle.cos() * sat.distance;
                let sy = self.y + sat.angle.sin() * sat.distance;
                let vx = sat.angle.cos() * 800.0 + 300.0;
                let vy = sat.angle.sin() * 800.0;
                bullets.push(Bullet::new(sx, sy, vx, vy, 5.0, 18.0, BulletOwner::Satellite(self.id), BulletType::NormalVulcan, 0xFF00FFFF));
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
            true // Murió o perdió vida
        } else {
            false
        }
    }
}
