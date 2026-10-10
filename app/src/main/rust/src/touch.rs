//! Módulo de Gestión Táctil para IA Rebellion (Inspirado en Final Mission NES)
//! Arquitectura 100% Nativa sin botones virtuales en pantalla:
//! - Primer dedo (movimiento): Arrastre relativo 1:1 de Jugador.java (sin tapar el personaje).
//!   NO cambia nunca la dirección del personaje, garantizando esquivas y desplazamientos limpios.
//! - Segundo dedo (dedo libre - Exclusión Mutua):
//!   * Modo Satélites: Al arrastrar y rotar en 360°, la dirección del jugador queda totalmente bloqueada,
//!     garantizando que apuntar o girar satélites NUNCA cambie la orientación del personaje.
//!   * Cambio de Dirección (Flick): Latigazo horizontal rápido que se suelta (Swipe & Release < 0.25s).
//! - Ataque especial (Bomba EMP):
//!   * Toque rápido simultáneo de dos dedos (two-finger tap) o tap seco del segundo dedo (< 0.25s sin arrastre).
//!   * Doble toque rápido repetido (< 0.35s).

#[derive(Clone, Copy, Debug)]
pub struct TouchHistoryPoint {
    pub x: f32,
    pub y: f32,
    pub time: f32,
}

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

    // Tiempo de pulsación del primer dedo
    pub primary_down_time: f32,

    // Estado del segundo dedo (dedo libre: satélites + dirección + flick)
    pub secondary_start_x: f32,
    pub secondary_start_y: f32,
    pub secondary_touch_time: f32,
    pub secondary_has_dragged: bool,

    pub touch_timer: f32,
    pub flick_cooldown: f32,
    pub flick_facing: Option<bool>, // Some(true) = derecha, Some(false) = izquierda

    // Acciones de juego
    pub is_touching: bool,
    pub is_firing: bool,
    pub satellite_lock: bool,
    pub satellite_target_angle: Option<f32>,
    pub trigger_bomb: bool,
    pub toggle_facing: bool,

    // Temporizador para doble toque rápido
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
            primary_down_time: 0.0,
            secondary_start_x: 0.0,
            secondary_start_y: 0.0,
            secondary_touch_time: 0.0,
            secondary_has_dragged: false,
            touch_timer: 0.0,
            flick_cooldown: 0.0,
            flick_facing: None,
            is_touching: false,
            is_firing: false,
            satellite_lock: false,
            satellite_target_angle: None,
            trigger_bomb: false,
            toggle_facing: false,
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
        self.touch_timer += dt;
        self.time_since_last_tap += dt;
        self.toggle_facing = false;
        self.flick_facing = None;

        if self.flick_cooldown > 0.0 {
            self.flick_cooldown -= dt;
        }

        // Si el segundo dedo se mantiene presionado:
        if self.secondary_id != -1 {
            self.secondary_touch_time += dt;
            // Confirmación de apuntado sostenido de satélites
            if self.secondary_touch_time >= 0.16 && !self.secondary_has_dragged {
                let dx = self.secondary_x - self.secondary_start_x;
                let dy = self.secondary_y - self.secondary_start_y;
                if dx * dx + dy * dy > 36.0 {
                    self.secondary_has_dragged = true;
                    self.satellite_lock = true;
                    if self.satellite_target_angle.is_none() {
                        self.satellite_target_angle = Some(dy.atan2(dx));
                    }
                }
            }
        }
    }

    /// Evento toquePresionado (AMOTION_EVENT_ACTION_DOWN / POINTER_DOWN)
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
            self.primary_down_time = self.touch_timer;

            // Detección de doble toque rápido (< 0.35s) con un solo dedo para lanzar Bomba EMP
            if self.time_since_last_tap < 0.35 {
                self.trigger_bomb = true;
                self.time_since_last_tap = 10.0;
            } else {
                self.time_since_last_tap = 0.0;
            }
        } else if self.secondary_id == -1 && id != self.primary_id {
            // Segundo dedo detectado (dedo libre)
            self.secondary_id = id;
            self.secondary_x = x;
            self.secondary_y = y;
            self.secondary_start_x = x;
            self.secondary_start_y = y;
            self.secondary_touch_time = 0.0;
            self.secondary_has_dragged = false;

            // Ataque especial por pulsación rápida con los dos dedos (two-finger tap simultáneo):
            // Si el segundo dedo cae casi al mismo tiempo que el primero (< 0.20s)
            if (self.touch_timer - self.primary_down_time).abs() < 0.20 {
                self.trigger_bomb = true;
            }
        }
    }

    /// Evento toqueDeslizando (AMOTION_EVENT_ACTION_MOVE)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.primary_id == id {
            self.primary_x = x;
            self.primary_y = y;
            self.is_touching = true;
            self.is_firing = true;

            // El dedo primario SOLO desplaza al jugador por la pantalla.
            // NO evalúa gestos flick ni cambia jamás la dirección del personaje.
            if self.touch_initialized {
                let new_x = x - self.delta_x_tactil;
                let new_y = y - self.delta_y_tactil;
                return Some((new_x, new_y));
            }
        } else if self.secondary_id == id {
            self.secondary_x = x;
            self.secondary_y = y;

            let mut dx = x - self.secondary_start_x;
            let mut dy = y - self.secondary_start_y;
            let dist_sq = dx * dx + dy * dy;

            // Al superar 14 px de desplazamiento o 0.14s, se entra en MODO SATÉLITES:
            // Al activarse satellite_lock, cualquier cambio de dirección queda 100% bloqueado.
            if dist_sq > 196.0 || self.secondary_touch_time > 0.14 {
                self.secondary_has_dragged = true;
                self.satellite_lock = true;

                // Clamping analógico para radio de 60px
                let dist = dist_sq.sqrt();
                let max_radius = 60.0;
                if dist > max_radius {
                    self.secondary_start_x = x - (dx / dist) * max_radius;
                    self.secondary_start_y = y - (dy / dist) * max_radius;
                    dx = x - self.secondary_start_x;
                    dy = y - self.secondary_start_y;
                }

                self.satellite_target_angle = Some(dy.atan2(dx));
            }
        }
        None
    }

    /// Evento toqueLevantado (AMOTION_EVENT_ACTION_UP / POINTER_UP)
    pub fn on_touch_up(&mut self, id: i32, x: f32, y: f32) {
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
                self.secondary_has_dragged = false;
                self.secondary_touch_time = 0.0;
            } else {
                self.primary_id = -1;
                self.touch_initialized = false;
                self.is_touching = false;
                self.is_firing = false;
                self.satellite_lock = false;
                self.satellite_target_angle = None;
            }
        } else if self.secondary_id == id {
            let dx = x - self.secondary_start_x;
            let dy = y - self.secondary_start_y;
            let dist_sq = dx * dx + dy * dy;

            // ==============================================================
            // REGLA DE EXCLUSIÓN MUTUA:
            // Si el dedo estuvo en MODO SATÉLITES (arrastró o apuntó en 360°),
            // NUNCA cambia la dirección de la nave ni lanza bomba accidental.
            // ==============================================================
            if !self.secondary_has_dragged && !self.satellite_lock && self.secondary_touch_time <= 0.25 {
                // Gesto Flick / Swipe & Release deliberado:
                // Latigazo horizontal rápido en menos de 0.25s con desplazamiento claro
                if dx.abs() >= 40.0 && dx.abs() >= dy.abs() * 1.25 {
                    self.flick_facing = Some(dx > 0.0);
                    self.flick_cooldown = 0.25;
                } else if dist_sq < 324.0 {
                    // Toque seco en un punto (< 18 px de desplazamiento): Bomba especial EMP
                    self.trigger_bomb = true;
                }
            }

            // Liberar estado del segundo dedo
            self.secondary_id = -1;
            self.satellite_lock = false;
            self.satellite_target_angle = None;
            self.secondary_has_dragged = false;
            self.secondary_touch_time = 0.0;
        }
    }
}
