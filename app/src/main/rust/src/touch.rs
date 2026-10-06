//! Módulo de Gestión Táctil para IA Rebellion
//! Implementa fielmente el algoritmo de control táctil de Jugador.java:
//! - deltaXTactil = xPantalla - this.x;
//! - deltaYTactil = yPantalla - this.y;
//! - this.x = xPantalla - deltaXTactil;
//! - this.y = yPantalla - deltaYTactil;
//!
//! Permite arrastre 1:1 directo desde cualquier punto de la pantalla sin tapar el personaje.
//! Disparo automático continuo mientras se mantenga el toque.
//! Doble toque rápido (< 0.35s) para Bomba EMP.
//! Soporte multi-touch con segundo dedo para orientar y bloquear satélites (Final Mission).

#[derive(Clone, Debug)]
pub struct TouchControls {
    pub primary_id: i32,
    pub secondary_id: i32,

    // Algoritmo original exacto de Jugador.java (toquePresionado / toqueDeslizando)
    pub delta_x_tactil: f32,
    pub delta_y_tactil: f32,
    pub touch_initialized: bool,

    // Coordenadas actuales de los dedos
    pub primary_x: f32,
    pub primary_y: f32,
    pub secondary_x: f32,
    pub secondary_y: f32,

    // Acciones de juego
    pub is_touching: bool,
    pub is_firing: bool,
    pub satellite_lock: bool,
    pub satellite_target_angle: Option<f32>,
    pub trigger_bomb: bool,

    // Temporizador para doble toque rápido (Bomba EMP)
    pub time_since_last_tap: f32,

    pub screen_width: f32,
    pub screen_height: f32,
}

impl TouchControls {
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            primary_id: -1,
            secondary_id: -1,
            delta_x_tactil: 0.0,
            delta_y_tactil: 0.0,
            touch_initialized: false,
            primary_x: 0.0,
            primary_y: 0.0,
            secondary_x: 0.0,
            secondary_y: 0.0,
            is_touching: false,
            is_firing: false,
            satellite_lock: false,
            satellite_target_angle: None,
            trigger_bomb: false,
            time_since_last_tap: 10.0,
            screen_width: w,
            screen_height: h,
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.screen_width = w;
        self.screen_height = h;
    }

    pub fn update(&mut self, dt: f32) {
        self.time_since_last_tap += dt;
        self.trigger_bomb = false;
    }

    /// Evento toquePresionado de Jugador.java
    pub fn on_touch_down(&mut self, id: i32, x: f32, y: f32, px: f32, py: f32) {
        if self.primary_id == -1 {
            self.primary_id = id;
            self.primary_x = x;
            self.primary_y = y;

            // Algoritmo exacto de Jugador.java:
            // deltaXTactil = xPantalla - this.x;
            // deltaYTactil = yPantalla - this.y;
            self.delta_x_tactil = x - px;
            self.delta_y_tactil = y - py;
            self.touch_initialized = true;

            self.is_touching = true;
            self.is_firing = true;

            // Detección de doble toque rápido (< 0.35s) para lanzar Bomba EMP
            if self.time_since_last_tap < 0.35 {
                self.trigger_bomb = true;
                self.time_since_last_tap = 10.0;
            } else {
                self.time_since_last_tap = 0.0;
            }
        } else if self.secondary_id == -1 && id != self.primary_id {
            // Segundo dedo -> orientar y bloquear satélites (Final Mission Satellite Lock)
            self.secondary_id = id;
            self.secondary_x = x;
            self.secondary_y = y;
            self.satellite_lock = true;
            let dy = y - self.primary_y;
            let dx = x - self.primary_x;
            self.satellite_target_angle = Some(dy.atan2(dx));
        }
    }

    /// Evento toqueDeslizando de Jugador.java
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.primary_id == id {
            self.primary_x = x;
            self.primary_y = y;
            self.is_touching = true;
            self.is_firing = true;

            if self.touch_initialized {
                // Algoritmo exacto de Jugador.java:
                // x1 = xPantalla - deltaXTactil;
                // y1 = yPantalla - deltaYTactil;
                let new_x = x - self.delta_x_tactil;
                let new_y = y - self.delta_y_tactil;
                return Some((new_x, new_y));
            }
        } else if self.secondary_id == id {
            self.secondary_x = x;
            self.secondary_y = y;
            self.satellite_lock = true;
            let dy = y - self.primary_y;
            let dx = x - self.primary_x;
            self.satellite_target_angle = Some(dy.atan2(dx));
        }
        None
    }

    /// Evento toqueLevantado de Jugador.java
    pub fn on_touch_up(&mut self, id: i32, _x: f32, _y: f32) {
        if self.primary_id == id {
            if self.secondary_id != -1 {
                // Promover segundo dedo a primario
                self.primary_id = self.secondary_id;
                self.primary_x = self.secondary_x;
                self.primary_y = self.secondary_y;
                self.secondary_id = -1;
                self.touch_initialized = false;
                self.satellite_lock = false;
                self.satellite_target_angle = None;
            } else {
                self.primary_id = -1;
                self.touch_initialized = false;
                self.is_touching = false;
                self.is_firing = false;
                self.satellite_lock = false;
                self.satellite_target_angle = None;
            }
        } else if self.secondary_id == id {
            self.secondary_id = -1;
            self.satellite_lock = false;
            self.satellite_target_angle = None;
        }
    }
}
