//! Módulo de Proyectiles para IA Rebellion
//! Soporta múltiples tipos de armas del jugador (inspiradas en Final Mission y Abadox)
//! y patrones de disparos enemigos.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BulletOwner {
    Player(u8), // ID del jugador (0..=3)
    Satellite(u8),
    Enemy,
    Boss,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BulletType {
    NormalVulcan,
    LaserBeam,
    SpreadWave,
    HomingMissile,
    PlasmaBomb,
    EnemyPlasma,
    EnemyHoming,
    EnemyLaser,
    BioAcid, // Inspiración Abadox
}

#[derive(Clone, Debug)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub radius: f32,
    pub damage: f32,
    pub owner: BulletOwner,
    pub b_type: BulletType,
    pub life: f32,
    pub active: bool,
    pub color: u32, // RGBA en formato 0xAABBGGRR
}

impl Bullet {
    pub fn new(
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        radius: f32,
        damage: f32,
        owner: BulletOwner,
        b_type: BulletType,
        color: u32,
    ) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            radius,
            damage,
            owner,
            b_type,
            life: 6.0,
            active: true,
            color,
        }
    }

    pub fn update(&mut self, dt: f32, target_x: f32, target_y: f32) {
        if !self.active {
            return;
        }

        // Comportamiento de misiles teledirigidos
        if self.b_type == BulletType::HomingMissile || self.b_type == BulletType::EnemyHoming {
            let dx = target_x - self.x;
            let dy = target_y - self.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 5.0 {
                let speed = (self.vx * self.vx + self.vy * self.vy).sqrt().max(350.0);
                let turn_rate = 6.0 * dt;
                let target_vx = (dx / dist) * speed;
                let target_vy = (dy / dist) * speed;
                self.vx += (target_vx - self.vx) * turn_rate;
                self.vy += (target_vy - self.vy) * turn_rate;
            }
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.life -= dt;

        if self.life <= 0.0 || self.x < -100.0 || self.x > 2100.0 || self.y < -100.0 || self.y > 1200.0 {
            self.active = false;
        }
    }
}
