//! Módulo de los 8 Jefes Principales (Bosses) para los 8 Niveles de IA Rebellion
//! Basado en las mecánicas de jefes de Final Mission y Abadox de Natsume

use crate::bullet::{Bullet, BulletOwner, BulletType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BossId {
    Stage1ZeroAlpha,     // Órbita Terrestre
    Stage2BlazeColossus, // Fábrica de Drones
    Stage3SentinelHunter,// Megaciudad Desolada
    Stage4GaiaBioCore,   // Complejo Subterráneo Ciber-Orgánico (Abadox)
    Stage5LuxFortress,   // Estación Espacial Militar
    Stage6OrionStriker,  // Cinturón de Asteroides
    Stage7NebulaDread,   // Súper-Nodriza Matriz
    Stage8NyxOvermind,   // Núcleo Cuántico de la IA Rebelde (Final)
}

#[derive(Clone, Debug)]
pub struct Boss {
    pub id: BossId,
    pub name: String,
    pub stage: u8,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub health: f32,
    pub max_health: f32,
    pub phase: u8,
    pub attack_timer: f32,
    pub move_timer: f32,
    pub active: bool,
    pub defeated: bool,
}

impl Boss {
    pub fn new_for_stage(stage: u8, screen_w: f32, screen_h: f32) -> Self {
        let (id, name, hp, r) = match stage {
            1 => (BossId::Stage1ZeroAlpha, "ZERO-ALPHA GUARDIAN", 1200.0, 70.0),
            2 => (BossId::Stage2BlazeColossus, "BLAZE-COLOSSUS", 2000.0, 85.0),
            3 => (BossId::Stage3SentinelHunter, "SENTINEL-V HUNTER", 2600.0, 65.0),
            4 => (BossId::Stage4GaiaBioCore, "GAIA-BIOCORE", 3400.0, 95.0),
            5 => (BossId::Stage5LuxFortress, "LUX-PHOTON FORTRESS", 4200.0, 110.0),
            6 => (BossId::Stage6OrionStriker, "ORION-VOID STRIKER", 5000.0, 75.0),
            7 => (BossId::Stage7NebulaDread, "NEBULA-DREADNOUGHT", 6200.0, 125.0),
            _ => (BossId::Stage8NyxOvermind, "NYX-OVERMIND SINGULARITY", 8000.0, 100.0),
        };

        Self {
            id,
            name: name.to_string(),
            stage,
            x: screen_w + 120.0, // Entra deslizándose desde la derecha
            y: screen_h * 0.5,
            vx: -160.0,
            vy: 80.0,
            radius: r,
            health: hp,
            max_health: hp,
            phase: 1,
            attack_timer: 2.0,
            move_timer: 0.0,
            active: true,
            defeated: false,
        }
    }

    pub fn update(&mut self, dt: f32, player_x: f32, player_y: f32, screen_w: f32, screen_h: f32, bullets: &mut Vec<Bullet>) {
        if !self.active || self.defeated {
            return;
        }

        self.move_timer += dt;
        self.attack_timer -= dt;

        // Comportamiento de entrada inicial
        let target_entry_x = screen_w - 240.0;
        if self.x > target_entry_x {
            self.x -= 220.0 * dt;
        } else {
            // Movimiento oscilatorio vertical y acecho
            self.y += (self.move_timer * 1.8).sin() * 190.0 * dt;
            self.x += (self.move_timer * 1.1).cos() * 60.0 * dt;
            self.y = self.y.clamp(self.radius + 20.0, screen_h - self.radius - 20.0);
        }

        // Actualizar fases según porcentaje de vida
        let hp_pct = self.health / self.max_health;
        if hp_pct < 0.35 {
            self.phase = 3;
        } else if hp_pct < 0.70 {
            self.phase = 2;
        }

        // Patrones de ataque según el jefe de cada nivel
        if self.attack_timer <= 0.0 && self.x <= target_entry_x + 50.0 {
            self.execute_boss_attack(player_x, player_y, bullets);
        }
    }

    fn execute_boss_attack(&mut self, player_x: f32, player_y: f32, bullets: &mut Vec<Bullet>) {
        let owner = BulletOwner::Boss;
        self.attack_timer = match self.phase {
            3 => 1.1,
            2 => 1.5,
            _ => 2.0,
        };

        match self.id {
            // Jefe 1: Ráfaga triple y láser de pulso
            BossId::Stage1ZeroAlpha => {
                for i in -2..=2 {
                    let angle = (i as f32) * 0.18;
                    bullets.push(Bullet::new(self.x - 30.0, self.y, -480.0 * angle.cos(), 480.0 * angle.sin(), 7.0, 20.0, owner, BulletType::EnemyPlasma, 0xFFFF0055));
                }
            }
            // Jefe 2: Salvas industriales de mortero
            BossId::Stage2BlazeColossus => {
                bullets.push(Bullet::new(self.x - 40.0, self.y - 30.0, -520.0, -120.0, 9.0, 25.0, owner, BulletType::EnemyPlasma, 0xFFFF8C00));
                bullets.push(Bullet::new(self.x - 40.0, self.y + 30.0, -520.0, 120.0, 9.0, 25.0, owner, BulletType::EnemyPlasma, 0xFFFF8C00));
            }
            // Jefe 3: Láseres dirigidos de alta velocidad
            BossId::Stage3SentinelHunter => {
                let dx = player_x - self.x;
                let dy = player_y - self.y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                bullets.push(Bullet::new(self.x - 30.0, self.y, (dx / dist) * 750.0, (dy / dist) * 750.0, 10.0, 30.0, owner, BulletType::EnemyLaser, 0xFF00FFFF));
            }
            // Jefe 4: Tentáculos de bio-ácido orgánico (Inspirado en Abadox)
            BossId::Stage4GaiaBioCore => {
                for a in 0..8 {
                    let rad = (a as f32) * (std::f32::consts::PI / 4.0) + self.move_timer;
                    bullets.push(Bullet::new(self.x, self.y, rad.cos() * 320.0, rad.sin() * 320.0, 8.0, 22.0, owner, BulletType::BioAcid, 0xFF76EE00));
                }
            }
            // Jefe 5: Anillo de fotones reflectantes
            BossId::Stage5LuxFortress => {
                for i in 0..6 {
                    let vy = ((i as f32) - 2.5) * 110.0;
                    bullets.push(Bullet::new(self.x - 40.0, self.y, -540.0, vy, 8.0, 24.0, owner, BulletType::EnemyPlasma, 0xFFFFD700));
                }
            }
            // Jefe 6: Misiles cuánticos teledirigidos
            BossId::Stage6OrionStriker => {
                bullets.push(Bullet::new(self.x, self.y - 40.0, -380.0, -180.0, 7.0, 32.0, owner, BulletType::EnemyHoming, 0xFF8A2BE2));
                bullets.push(Bullet::new(self.x, self.y + 40.0, -380.0, 180.0, 7.0, 32.0, owner, BulletType::EnemyHoming, 0xFF8A2BE2));
            }
            // Jefe 7: Andanada pesada de nave nodriza
            BossId::Stage7NebulaDread => {
                for i in -3..=3 {
                    let vy = (i as f32) * 90.0;
                    bullets.push(Bullet::new(self.x - 60.0, self.y, -600.0, vy, 8.0, 26.0, owner, BulletType::EnemyPlasma, 0xFFFF1493));
                }
            }
            // Jefe 8: Singularidad cuántica de la IA Master (Bullet Hell épico)
            BossId::Stage8NyxOvermind => {
                let count = if self.phase == 3 { 12 } else { 8 };
                for i in 0..count {
                    let angle = (i as f32) * (std::f32::consts::PI * 2.0 / count as f32) + self.move_timer * 2.0;
                    bullets.push(Bullet::new(self.x, self.y, angle.cos() * 450.0, angle.sin() * 450.0, 9.0, 28.0, owner, BulletType::EnemyPlasma, 0xFFFF0000));
                }
            }
        }
    }

    pub fn take_damage(&mut self, dmg: f32) -> bool {
        self.health -= dmg;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.defeated = true;
            true
        } else {
            false
        }
    }
}
