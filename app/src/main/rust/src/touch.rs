//! Módulo de Gestión Táctil para IA Rebellion (Inspirado en Final Mission NES)
//! Arquitectura 100% Nativa sin botones virtuales en pantalla:
//! - Arrastre relativo 1:1 de Jugador.java (sin tapar el personaje).
//! - Doble toque rápido (< 0.35s) para Bomba EMP.
//! - Multi-touch: toque rápido (< 0.28s) con segundo dedo conmuta orientación 180° (Voltear adelante <-> atrás).
//! - Gesto Flick: deslizamiento horizontal rápido en dirección contraria conmuta orientación.
//! - Hold & Drag con segundo dedo: bloquea y apunta satélites orbitales en 360 grados.

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

    // Estado del segundo dedo (Multi-touch: Conmutación vs Bloqueo Satelital)
    pub secondary_start_x: f32,
    pub secondary_start_y: f32,
    pub secondary_touch_time: f32,
    pub secondary_has_dragged: bool,

    // Detección de Gesto Flick / Deslizamiento Rápido con dedo primario
    pub flick_anchor_x: f32,
    pub flick_anchor_time: f32,
    pub flick_cooldown: f32,
    pub flick_facing: Option<bool>, // Some(true) = derecha, Some(false) = izquierda

    // Acciones de juego
    pub is_touching: bool,
    pub is_firing: bool,
    pub satellite_lock: bool,
    pub satellite_target_angle: Option<f32>,
    pub trigger_bomb: bool,
    pub toggle_facing: bool, // Disparo único de conmutación 180°

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
            secondary_start_x: 0.0,
            secondary_start_y: 0.0,
            secondary_touch_time: 0.0,
            secondary_has_dragged: false,
            flick_anchor_x: 0.0,
            flick_anchor_time: 0.0,
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
        self.time_since_last_tap += dt;
        self.trigger_bomb = false;
        self.toggle_facing = false;
        self.flick_facing = None;

        if self.flick_cooldown > 0.0 {
            self.flick_cooldown -= dt;
        }

        self.flick_anchor_time += dt;
        if self.flick_anchor_time >= 0.08 {
            self.flick_anchor_x = self.primary_x;
            self.flick_anchor_time = 0.0;
        }

        // Si el segundo dedo se mantiene presionado sin soltarlo:
        if self.secondary_id != -1 {
            self.secondary_touch_time += dt;
            // Al superar 0.22s de presión continua, se confirma modo Hold de satélites
            if self.secondary_touch_time >= 0.22 && !self.secondary_has_dragged {
                self.secondary_has_dragged = true;
                self.satellite_lock = true;
                if self.satellite_target_angle.is_none() {
                    let dx = self.secondary_x - self.secondary_start_x;
                    let dy = self.secondary_y - self.secondary_start_y;
                    if dx * dx + dy * dy > 16.0 {
                        self.satellite_target_angle = Some(dy.atan2(dx));
                    }
                }
            }
        }
    }

    /// Evento toquePresionado de Jugador.java (AMOTION_EVENT_ACTION_DOWN / POINTER_DOWN)
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

            // Inicializar detección de flick horizontal
            self.flick_anchor_x = x;
            self.flick_anchor_time = 0.0;

            // Detección de doble toque rápido (< 0.35s) para lanzar Bomba EMP
            if self.time_since_last_tap < 0.35 {
                self.trigger_bomb = true;
                self.time_since_last_tap = 10.0;
            } else {
                self.time_since_last_tap = 0.0;
            }
        } else if self.secondary_id == -1 && id != self.primary_id {
            // Segundo dedo detectado: Registrar ancla inicial y tiempo
            // No activar satellite_lock de inmediato para permitir Quick Tap (volteo 180°)
            self.secondary_id = id;
            self.secondary_x = x;
            self.secondary_y = y;
            self.secondary_start_x = x;
            self.secondary_start_y = y;
            self.secondary_touch_time = 0.0;
            self.secondary_has_dragged = false;
        }
    }

    /// Evento toqueDeslizando de Jugador.java (AMOTION_EVENT_ACTION_MOVE)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.primary_id == id {
            self.primary_x = x;
            self.primary_y = y;
            self.is_touching = true;
            self.is_firing = true;

            // Detección de Gesto Flick / Swipe Rápido Horizontal (dedo primario)
            if self.flick_cooldown <= 0.0 {
                let dx_flick = x - self.flick_anchor_x;
                let dt_flick = self.flick_anchor_time.max(0.016);
                let vx_flick = dx_flick / dt_flick;

                // Si supera umbral de velocidad (> 650 px/s) y distancia mínima (> 35 px)
                if dx_flick.abs() > 35.0 && vx_flick.abs() > 650.0 {
                    self.flick_facing = Some(dx_flick > 0.0);
                    self.flick_cooldown = 0.28;
                    self.flick_anchor_x = x;
                    self.flick_anchor_time = 0.0;
                }
            }

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

            // Desplazamiento relativo desde el ancla de pulsación del segundo dedo
            let mut dx = x - self.secondary_start_x;
            let mut dy = y - self.secondary_start_y;
            let dist_sq = dx * dx + dy * dy;

            // Umbral de arrastre (> 12 px) activa bloqueo y apuntado en 360°
            if dist_sq > 144.0 {
                self.secondary_has_dragged = true;
                self.satellite_lock = true;

                // Clamping flotante (radio máx 60px) para mantener control analógico ágil
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

    /// Evento toqueLevantado de Jugador.java (AMOTION_EVENT_ACTION_UP / POINTER_UP)
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
            // Si el segundo dedo se levantó rápidamente sin arrastre significativo:
            // -> TOQUE RÁPIDO (Quick Tap): Conmuta orientación 180° estilo Final Mission NES!
            if !self.secondary_has_dragged && self.secondary_touch_time < 0.28 {
                self.toggle_facing = true;
            }

            self.secondary_id = -1;
            self.satellite_lock = false;
            self.satellite_target_angle = None;
            self.secondary_has_dragged = false;
            self.secondary_touch_time = 0.0;
        }
    }
}
