//! Módulo de Gestión Táctil para IA Rebellion (Inspirado en Final Mission NES)
//! Arquitectura 100% Nativa sin botones virtuales en pantalla:
//! - Arrastre relativo 1:1 de Jugador.java (sin tapar el personaje).
//! - Doble toque rápido (< 0.35s) para Bomba EMP.
//! - Multi-touch: toque rápido (< 0.28s) con segundo dedo conmuta orientación 180° (Voltear adelante <-> atrás).
//! - Gesto Flick: deslizamiento horizontal rápido en dirección contraria conmuta orientación.
//! - Hold & Drag con segundo dedo: bloquea y apunta satélites orbitales en 360 grados.

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

    // Estado del segundo dedo (Multi-touch: Conmutación vs Bloqueo Satelital)
    pub secondary_start_x: f32,
    pub secondary_start_y: f32,
    pub secondary_touch_time: f32,
    pub secondary_has_dragged: bool,

    // Detección estricta de Gesto Flick / Arrastre Rápido Horizontal
    pub touch_timer: f32,
    pub history: [TouchHistoryPoint; 8],
    pub history_len: usize,
    pub history_idx: usize,
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
            touch_timer: 0.0,
            history: [TouchHistoryPoint { x: 0.0, y: 0.0, time: 0.0 }; 8],
            history_len: 0,
            history_idx: 0,
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
        self.trigger_bomb = false;
        self.toggle_facing = false;
        self.flick_facing = None;

        if self.flick_cooldown > 0.0 {
            self.flick_cooldown -= dt;
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

            // Inicializar historial de tracking para detección de arrastre rápido (flick)
            self.history_len = 1;
            self.history_idx = 0;
            self.history[0] = TouchHistoryPoint { x, y, time: self.touch_timer };
            self.flick_cooldown = 0.0;

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

            let now = self.touch_timer;
            self.history_idx = (self.history_idx + 1) % 8;
            self.history[self.history_idx] = TouchHistoryPoint { x, y, time: now };
            if self.history_len < 8 {
                self.history_len += 1;
            }

            // Detección de Gesto Flick / Arrastre Rápido Horizontal (dedo primario)
            // Solo cambia la orientación si se realiza un latigazo rápido intencional,
            // garantizando que arrastrar el dedo hacia adelante/atrás para mover al jugador
            // NUNCA altere la dirección configurada.
            if self.flick_cooldown <= 0.0 && self.history_len >= 2 {
                for i in 1..self.history_len {
                    let past_idx = (self.history_idx + 8 - i) % 8;
                    let sample = self.history[past_idx];
                    let dt = now - sample.time;
                    if dt >= 0.025 && dt <= 0.12 {
                        let dx = x - sample.x;
                        let dy = y - sample.y;
                        let vx = dx / dt.max(0.016);

                        // Umbrales estrictos de arrastre rápido intencional:
                        // 1. Distancia horizontal mínima: >= 70 px
                        // 2. Velocidad explosiva de swipe: >= 1700 px/s
                        // 3. Dominancia horizontal clara: dx >= dy * 1.4
                        if dx.abs() >= 70.0 && vx.abs() >= 1700.0 && dx.abs() >= dy.abs() * 1.4 {
                            self.flick_facing = Some(dx > 0.0);
                            self.flick_cooldown = 0.28;
                            // Resetea el historial para evitar activaciones múltiples en el mismo trazo
                            self.history_len = 1;
                            self.history[self.history_idx] = TouchHistoryPoint { x, y, time: now };
                            break;
                        }
                    }
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
            self.secondary_id = -1;
            self.satellite_lock = false;
            self.satellite_target_angle = None;
            self.secondary_has_dragged = false;
            self.secondary_touch_time = 0.0;
        }
    }
}
