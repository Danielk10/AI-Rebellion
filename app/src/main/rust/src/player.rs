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

    pub fn update(&mut self, dt: f32, lock_requested: bool, target_angle: Option<f32>, forward_angle: f32) {
        self.is_locked = lock_requested;
        if self.is_locked {
            // Modo bloqueado (tactical lock): mantiene su posición fija y orienta el cañón al ángulo fijado
            if let Some(target) = target_angle {
                // Camino angular más corto (evita saltos bruscos en +/- PI)
                let mut diff = (target - self.aim_angle) % (std::f32::consts::PI * 2.0);
                if diff > std::f32::consts::PI {
                    diff -= std::f32::consts::PI * 2.0;
                } else if diff < -std::f32::consts::PI {
                    diff += std::f32::consts::PI * 2.0;
                }
                self.aim_angle += diff * (16.0 * dt).min(1.0);
            }
        } else {
            // Modo libre (unlocked): rotación continua orbital alrededor del jugador (Final Mission style)
            self.angle += 3.8 * dt;
            if self.angle > std::f32::consts::PI * 2.0 {
                self.angle -= std::f32::consts::PI * 2.0;
            }
            // En modo sin bloqueo, orientar siempre hacia el frente de vuelo del jugador
            self.aim_angle = forward_angle;
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
    pub facing_right: bool,
    pub facing_scale: f32, // -1.0 (izquierda) .. +1.0 (derecha) para giro 3D continuo sin saltos bruscos
    pub flight_pitch: f32, // Inclinación aerodinámica del cuerpo en radianes
    pub flight_thrust: f32, // Intensidad de empuje de los reactores (0.0 .. 1.0)
    pub is_backpedaling: bool, // Postura táctica de vuelo hacia atrás disparando hacia el frente (Final Mission)
    pub measured_vx: f32,
    pub measured_vy: f32,
    pub prev_x: f32,
    pub prev_y: f32,
    pub anim_timer: f32,
    pub jetpack_active: bool,
    pub recoil_anim: f32, // Micro-animación de retroceso del rifle (1.0 = disparo, decae a 0.0)
    pub bank_angle: f32,  // Micro-animación de inclinación de vuelo (-15°..+15°)
}

impl Player {
    pub fn new(id: u8, start_x: f32, start_y: f32) -> Self {
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
            facing_right: true,
            facing_scale: 1.0,
            flight_pitch: 0.0,
            flight_thrust: 0.4,
            is_backpedaling: false,
            measured_vx: 0.0,
            measured_vy: 0.0,
            prev_x: start_x,
            prev_y: start_y,
            anim_timer: 0.0,
            jetpack_active: true,
            recoil_anim: 0.0,
            bank_angle: 0.0,
        }
    }

    pub fn set_facing(&mut self, right: bool) {
        self.facing_right = right;
    }

    pub fn reset_pose(&mut self) {
        self.facing_scale = if self.facing_right { 1.0 } else { -1.0 };
        self.flight_pitch = 0.0;
        self.bank_angle = 0.0;
        self.flight_thrust = 0.4;
        self.is_backpedaling = false;
        self.measured_vx = 0.0;
        self.measured_vy = 0.0;
        self.prev_x = self.x;
        self.prev_y = self.y;
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

        // Medir velocidad real de desplazamiento (incluye arrastre táctil)
        let raw_vx = (self.x - self.prev_x) / dt.max(0.0001);
        let raw_vy = (self.y - self.prev_y) / dt.max(0.0001);
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.measured_vx += (raw_vx - self.measured_vx) * (18.0 * dt).min(1.0);
        self.measured_vy += (raw_vy - self.measured_vy) * (18.0 * dt).min(1.0);

        // Giro 3D continuo suave (~0.16s) evitando cambios bruscos
        let target_scale = if self.facing_right { 1.0 } else { -1.0 };
        self.facing_scale += (target_scale - self.facing_scale) * (11.5 * dt).min(1.0);

        // Amortiguación del retroceso del arma (recoil kickback)
        if self.recoil_anim > 0.0 {
            self.recoil_anim = (self.recoil_anim - dt * 14.0).max(0.0);
        }

        // Dinámica de vuelo coherente con el movimiento estilo Final Mission
        let dir_sign = if self.facing_right { 1.0 } else { -1.0 };
        let forward_speed = self.measured_vx * dir_sign;
        let is_moving = self.measured_vx.abs() > 15.0 || self.measured_vy.abs() > 15.0;
        self.jetpack_active = is_moving;
        self.is_backpedaling = forward_speed < -20.0;

        let mut target_pitch = if self.is_backpedaling {
            0.14 * dir_sign
        } else {
            -0.28 * dir_sign * (forward_speed.abs() / 420.0).clamp(0.0, 1.0)
        };
        target_pitch += (self.measured_vy * 0.0008).clamp(-0.25, 0.25) * dir_sign;
        if !is_moving {
            target_pitch += (self.anim_timer * 3.5).sin() * 0.04;
        }
        self.flight_pitch += (target_pitch - self.flight_pitch) * (14.0 * dt).min(1.0);
        self.bank_angle = self.flight_pitch.to_degrees();

        let speed_mag = (self.measured_vx * self.measured_vx + self.measured_vy * self.measured_vy).sqrt();
        self.flight_thrust = if is_moving {
            (speed_mag / 380.0).clamp(0.4, 1.0)
        } else {
            0.25 + (self.anim_timer * 6.0).sin().abs() * 0.1
        };

        // Amortiguación inercial para retorno suave al hover neutro
        self.vx *= (1.0 - dt * 4.0).max(0.0);
        self.vy *= (1.0 - dt * 4.0).max(0.0);

        // Actualizar satélites tácticos bivalvos
        let forward_angle = if self.facing_right { 0.0 } else { std::f32::consts::PI };
        for sat in self.satellites.iter_mut() {
            sat.update(dt, sat_lock, target_angle, forward_angle);
        }
    }

    pub fn fire(&mut self, bullets: &mut Vec<Bullet>) {
        if self.fire_timer > 0.0 || !self.active {
            return;
        }

        // Durante el corte exacto de giro corporal (de frente a la cámara), suprimir el disparo
        if self.facing_scale.abs() < 0.20 {
            return;
        }

        self.fire_timer = match self.weapon {
            WeaponType::Vulcan => 0.11,
            WeaponType::Laser => 0.20,
            WeaponType::Spread => 0.17,
            WeaponType::Homing => 0.35,
        };

        // Gatilla el retroceso físico del arma instantáneamente
        self.recoil_anim = 1.0;

        let owner = BulletOwner::Player(self.id);
        let dir = if self.facing_scale >= 0.0 { 1.0 } else { -1.0 };
        let cos_p = self.flight_pitch.cos();
        let sin_p = self.flight_pitch.sin();
        let rot_gx = 28.0 * cos_p - (-2.0) * sin_p;
        let rot_gy = 28.0 * sin_p + (-2.0) * cos_p;
        let gun_x = self.x + rot_gx * dir;
        let gun_y = self.y + rot_gy;

        // Disparo principal del fusil de asalto pesado anti-IA (Inspirado en Final Mission)
        match self.weapon {
            WeaponType::Vulcan => {
                // Fusil de asalto pesado: en Nivel 1 dispara limpio y directo hacia adelante (dir * 960.0, 0.0)
                bullets.push(Bullet::new(gun_x, gun_y, dir * 960.0, 0.0, 6.0, 28.0, owner, BulletType::NormalVulcan, 0xFF00FFFF));
                match self.weapon_power {
                    1 => {
                        // Nivel 1: disparo único frontal directo sin proyectiles dispersos
                    }
                    2 => {
                        // Nivel 2: 3 proyectiles en abanico controlado y cerrado
                        bullets.push(Bullet::new(gun_x, gun_y - 8.0, dir * 940.0, -110.0, 5.5, 24.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                        bullets.push(Bullet::new(gun_x, gun_y + 8.0, dir * 940.0, 110.0, 5.5, 24.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                    }
                    _ => {
                        // Nivel 3: 5 proyectiles en abanico controlado sin dispersión vertical descontrolada
                        bullets.push(Bullet::new(gun_x, gun_y - 7.0, dir * 940.0, -100.0, 5.5, 24.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                        bullets.push(Bullet::new(gun_x, gun_y + 7.0, dir * 940.0, 100.0, 5.5, 24.0, owner, BulletType::NormalVulcan, 0xFF00E5FF));
                        bullets.push(Bullet::new(gun_x, gun_y - 14.0, dir * 910.0, -200.0, 5.0, 22.0, owner, BulletType::NormalVulcan, 0xFF38BDF8));
                        bullets.push(Bullet::new(gun_x, gun_y + 14.0, dir * 910.0, 200.0, 5.0, 22.0, owner, BulletType::NormalVulcan, 0xFF38BDF8));
                    }
                }
            }
            WeaponType::Laser => {
                // Rayo perforante denso y destructivo que atraviesa enemigos y estructuras destructibles
                let (beam_r, beam_dmg, beam_pierce) = match self.weapon_power {
                    1 => (12.0, 85.0, 4),
                    2 => (16.0, 130.0, 6),
                    _ => (20.0, 180.0, 9),
                };
                let mut core_beam = Bullet::new(gun_x, gun_y, dir * 1650.0, 0.0, beam_r, beam_dmg, owner, BulletType::LaserBeam, 0xFFFF4500);
                core_beam.pierce_count = beam_pierce;
                bullets.push(core_beam);

                if self.weapon_power >= 2 {
                    let mut beam_top = Bullet::new(gun_x, gun_y - 7.0, dir * 1650.0, 0.0, 8.0, 40.0, owner, BulletType::LaserBeam, 0xFFFF6B35);
                    beam_top.pierce_count = beam_pierce.saturating_sub(1);
                    bullets.push(beam_top);

                    let mut beam_bot = Bullet::new(gun_x, gun_y + 7.0, dir * 1650.0, 0.0, 8.0, 40.0, owner, BulletType::LaserBeam, 0xFFFF6B35);
                    beam_bot.pierce_count = beam_pierce.saturating_sub(1);
                    bullets.push(beam_bot);
                }

                if self.weapon_power >= 3 {
                    let mut beam_far_top = Bullet::new(gun_x, gun_y - 14.0, dir * 1650.0, 0.0, 7.0, 35.0, owner, BulletType::LaserBeam, 0xFFFF8C00);
                    beam_far_top.pierce_count = beam_pierce.saturating_sub(2);
                    bullets.push(beam_far_top);

                    let mut beam_far_bot = Bullet::new(gun_x, gun_y + 14.0, dir * 1650.0, 0.0, 7.0, 35.0, owner, BulletType::LaserBeam, 0xFFFF8C00);
                    beam_far_bot.pierce_count = beam_pierce.saturating_sub(2);
                    bullets.push(beam_far_bot);
                }
            }
            WeaponType::Spread => {
                // Escopeta de plasma con arco limpio angular frontal (estilo Wave/Spread de Final Mission)
                let angles: &[f32] = match self.weapon_power {
                    1 => &[-18.0, 0.0, 18.0],
                    2 => &[-30.0, -15.0, 0.0, 15.0, 30.0],
                    _ => &[-40.0, -26.0, -13.0, 0.0, 13.0, 26.0, 40.0],
                };
                let speed = 880.0;
                let (r, dmg) = match self.weapon_power {
                    1 => (7.0, 32.0),
                    2 => (7.5, 34.0),
                    _ => (8.0, 36.0),
                };
                for &deg in angles.iter() {
                    let rad = deg.to_radians();
                    let vx = dir * speed * rad.cos();
                    let vy = speed * rad.sin();
                    bullets.push(Bullet::new(gun_x, gun_y, vx, vy, r, dmg, owner, BulletType::SpreadWave, 0xFF7CFC00));
                }
            }
            WeaponType::Homing => {
                // Misiles tácticos teledirigidos precisos y contundentes (sin enjambre caótico)
                match self.weapon_power {
                    1 => {
                        // Nivel 1: exactamente 2 misiles teledirigidos (superior e inferior en gun_y +/- 12.0), limpios, directos, daño 65.0
                        bullets.push(Bullet::new(gun_x, gun_y - 12.0, dir * 600.0, -100.0, 6.0, 65.0, owner, BulletType::HomingMissile, 0xFFFF1493));
                        bullets.push(Bullet::new(gun_x, gun_y + 12.0, dir * 600.0, 100.0, 6.0, 65.0, owner, BulletType::HomingMissile, 0xFFFF1493));
                    }
                    2 => {
                        // Nivel 2: 2 misiles más pesados y rápidos, daño 95.0
                        bullets.push(Bullet::new(gun_x, gun_y - 12.0, dir * 680.0, -90.0, 7.0, 95.0, owner, BulletType::HomingMissile, 0xFFFF3366));
                        bullets.push(Bullet::new(gun_x, gun_y + 12.0, dir * 680.0, 90.0, 7.0, 95.0, owner, BulletType::HomingMissile, 0xFFFF3366));
                    }
                    _ => {
                        // Nivel 3: máximo 4 misiles (2 pares escalonados limpios), daño 120.0
                        bullets.push(Bullet::new(gun_x, gun_y - 12.0, dir * 720.0, -80.0, 7.5, 120.0, owner, BulletType::HomingMissile, 0xFFFF0055));
                        bullets.push(Bullet::new(gun_x, gun_y + 12.0, dir * 720.0, 80.0, 7.5, 120.0, owner, BulletType::HomingMissile, 0xFFFF0055));
                        bullets.push(Bullet::new(gun_x - 10.0, gun_y - 20.0, dir * 680.0, -180.0, 7.0, 120.0, owner, BulletType::HomingMissile, 0xFFFF3377));
                        bullets.push(Bullet::new(gun_x - 10.0, gun_y + 20.0, dir * 680.0, 180.0, 7.0, 120.0, owner, BulletType::HomingMissile, 0xFFFF3377));
                    }
                }
            }
        }

        // Disparo táctico de los Satélites Orbitales (Mecánica Final Mission)
        for sat in self.satellites.iter_mut() {
            if sat.fire_cooldown <= 0.0 {
                sat.fire_cooldown = 0.15;
                let sx = self.x + sat.angle.cos() * sat.distance;
                let sy = self.y + sat.angle.sin() * sat.distance;
                // En modo libre (unlocked), los satélites barren con su radio orbital vertical (Final Mission)
                // En modo bloqueado (locked), disparan en el ángulo táctico fijado (360°).
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
