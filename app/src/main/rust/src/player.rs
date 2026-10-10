//! Módulo del Jugador: Comando Cibernético Humano con Traje Avanzado y Jetpack
//! Inspirado en Final Mission (Famicom/NES Japón) de Natsume en la Rebelión de la IA.
//! Cuenta con vuelo multidireccional 1:1, orientación independiente adelante/atrás,
//! propulsor jetpack animado de plasma, micro-animaciones de retroceso e inclinación de vuelo,
//! y 2 satélites tácticos bivalvos orbitales.

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
    pub angle: f32,        // Posición orbital alrededor del jugador
    pub aim_angle: f32,    // Dirección de disparo independiente (360°)
    pub distance: f32,
    pub is_locked: bool,
    pub fire_cooldown: f32,
}

impl Satellite {
    pub fn new(initial_angle: f32) -> Self {
        Self {
            angle: initial_angle,
            aim_angle: initial_angle,
            distance: 46.0,
            is_locked: false,
            fire_cooldown: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, lock_requested: bool, target_angle: Option<f32>) {
        self.is_locked = lock_requested;
        if self.is_locked {
            if let Some(target) = target_angle {
                // Camino angular más corto (evita saltos bruscos en +/- PI)
                let mut diff = (target - self.aim_angle) % (std::f32::consts::PI * 2.0);
                if diff > std::f32::consts::PI {
                    diff -= std::f32::consts::PI * 2.0;
                } else if diff < -std::f32::consts::PI {
                    diff += std::f32::consts::PI * 2.0;
                }
                self.aim_angle += diff * (14.0 * dt).min(1.0);
            }
        } else {
            self.angle += 3.8 * dt; // Rotación continua estilo Final Mission
            if self.angle > std::f32::consts::PI * 2.0 {
                self.angle -= std::f32::consts::PI * 2.0;
            }
            self.aim_angle = self.angle;
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
    pub recoil_anim: f32, // Micro-animación de retroceso del rifle (1.0 = disparo, decae a 0.0)
    pub bank_angle: f32,  // Micro-animación de inclinación de vuelo (-15°..+15°)
}

impl Player {
    pub fn new(id: u8, start_x: f32, start_y: f32) -> Self {
        let color = match id {
            0 => 0xFF00D2FF, // Arnold (P1): Cobalto con acento cian
            1 => 0xFFFF2233, // Sigourney (P2): Carmesí escarlata con filigrana dorada
            2 => 0xFF00FF77, // Jax: Verde plasma esmeralda
            _ => 0xFFFFD700, // Orion: Titanio solar dorado
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
            recoil_anim: 0.0,
            bank_angle: 0.0,
        }
    }

    pub fn toggle_facing(&mut self) {
        self.facing_right = !self.facing_right;
    }

    pub fn set_facing(&mut self, right: bool) {
        self.facing_right = right;
    }

    /// Desplazamiento por arrastre táctil directo 1:1 (Algoritmo de Jugador.java)
    pub fn apply_touch_drag(&mut self, dx: f32, dy: f32, screen_w: f32, screen_h: f32) {
        if !self.active {
            return;
        }

        self.x = (self.x + dx).clamp(36.0, screen_w - 36.0);
        self.y = (self.y + dy).clamp(36.0, screen_h - 36.0);

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

        // Amortiguación del retroceso del arma (recoil kickback)
        if self.recoil_anim > 0.0 {
            self.recoil_anim = (self.recoil_anim - dt * 14.0).max(0.0);
        }

        // Micro-animación de inclinación de vuelo (Flight Banking)
        let dir_sign = if self.facing_right { 1.0 } else { -1.0 };
        let pitch_target = (self.vy * 0.045).clamp(-14.0, 14.0);
        let surge_target = (self.vx * dir_sign * 0.018).clamp(-6.0, 6.0);
        let desired_bank = pitch_target + surge_target;
        self.bank_angle += (desired_bank - self.bank_angle) * (16.0 * dt).min(1.0);

        // Amortiguación inercial para retorno suave al hover neutro
        self.vx *= (1.0 - dt * 4.0).max(0.0);
        self.vy *= (1.0 - dt * 4.0).max(0.0);

        // Actualizar satélites tácticos bivalvos
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

        // Gatilla el retroceso físico del arma instantáneamente
        self.recoil_anim = 1.0;

        let owner = BulletOwner::Player(self.id);
        let dir = if self.facing_right { 1.0 } else { -1.0 };
        let gun_x = self.x + dir * 28.0;
        let gun_y = self.y - 2.0;

        // Disparo principal del fusil de asalto pesado anti-IA
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

        // Disparo táctico de los Satélites Orbitales en 360°
        for sat in self.satellites.iter_mut() {
            if sat.fire_cooldown <= 0.0 {
                sat.fire_cooldown = 0.15;
                let sx = self.x + sat.angle.cos() * sat.distance;
                let sy = self.y + sat.angle.sin() * sat.distance;
                let (vx, vy) = if sat.is_locked {
                    (sat.aim_angle.cos() * 900.0, sat.aim_angle.sin() * 900.0)
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
                self.facing_right = true;
            } else {
                self.active = false;
            }
            true
        } else {
            false
        }
    }
}
