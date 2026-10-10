//! Módulo de Gestión Táctil para IA Rebellion (Inspirado en Final Mission NES)
//! Arquitectura 100% Nativa sin botones virtuales en pantalla:
//! - Primer dedo (movimiento): Arrastre relativo 1:1 de Jugador.java (sin tapar el personaje).
//!   NO cambia nunca la dirección del personaje, garantizando esquivas y desplazamientos limpios.
//! - Segundo dedo (dedo libre):
//!   * Arrastre sostenido: Controla y apunta satélites orbitales en 360 grados.
//!   * Arrastre rápido (flick horizontal): Conmuta la dirección del jugador (izquierda/derecha).
//! - Ataque especial (Bomba EMP):
//!   * Toque rápido simultáneo de dos dedos (o toque seco del segundo dedo < 0.35s).
//!   * Doble toque rápido repetido en pantalla (< 0.35s).

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

    // Historial del segundo dedo para detección de Flick / Arrastre Rápido Horizontal
    pub secondary_history: [TouchHistoryPoint; 8],
    pub secondary_history_len: usize,
    pub secondary_history_idx: usize,

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
            secondary_history: [TouchHistoryPoint { x: 0.0, y: 0.0, time: 0.0 }; 8],
            secondary_history_len: 0,
            secondary_history_idx: 0,
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

        // Si el segundo dedo se mantiene presionado sin soltarlo:
        if self.secondary_id != -1 {
            self.secondary_touch_time += dt;
            // Al superar 0.20s de presión continua con desplazamiento, se confirma modo Hold de satélites
            if self.secondary_touch_time >= 0.20 && !self.secondary_has_dragged {
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

            // Inicializar historial del segundo dedo para swipe rápido (flick)
            self.secondary_history_len = 1;
            self.secondary_history_idx = 0;
            self.secondary_history[0] = TouchHistoryPoint { x, y, time: self.touch_timer };

            // Ataque especial por pulsación rápida con los dos dedos (two-finger press):
            // Si el segundo dedo cae casi simultáneamente con el primero (< 0.25s)
            if (self.touch_timer - self.primary_down_time).abs() < 0.25 {
                self.trigger_bomb = true;
            }
        }
    }

    /// Evento toqueDeslizando de Jugador.java (AMOTION_EVENT_ACTION_MOVE)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.primary_id == id {
            self.primary_x = x;
            self.primary_y = y;
            self.is_touching = true;
            self.is_firing = true;

            // El dedo primario SOLO desplaza al jugador por la pantalla.
            // NO evalúa gestos flick ni cambia jamás la dirección del personaje,
            // permitiendo evasión y movimiento a máxima velocidad sin riesgo de giros involuntarios.
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

            let now = self.touch_timer;
            self.secondary_history_idx = (self.secondary_history_idx + 1) % 8;
            self.secondary_history[self.secondary_history_idx] = TouchHistoryPoint { x, y, time: now };
            if self.secondary_history_len < 8 {
                self.secondary_history_len += 1;
            }

            // Gesto Flick / Arrastre Rápido Horizontal del segundo dedo (dedo libre):
            // Controla deliberadamente la orientación (izquierda/derecha) del jugador
            if self.flick_cooldown <= 0.0 && self.secondary_history_len >= 2 {
                for i in 1..self.secondary_history_len {
                    let past_idx = (self.secondary_history_idx + 8 - i) % 8;
                    let sample = self.secondary_history[past_idx];
                    let dt = now - sample.time;
                    if dt >= 0.020 && dt <= 0.18 {
                        let dx = x - sample.x;
                        let dy = y - sample.y;
                        let vx = dx / dt.max(0.016);

                        // Umbral de arrastre rápido del segundo dedo:
                        // Distancia horizontal >= 50 px, velocidad >= 1200 px/s, dominancia horizontal
                        if dx.abs() >= 50.0 && vx.abs() >= 1200.0 && dx.abs() >= dy.abs() * 1.2 {
                            self.flick_facing = Some(dx > 0.0);
                            self.flick_cooldown = 0.25;
                            self.secondary_history_len = 1;
                            self.secondary_history[self.secondary_history_idx] = TouchHistoryPoint { x, y, time: now };
                            break;
                        }
                    }
                }
            }

            // Desplazamiento relativo desde el ancla de pulsación del segundo dedo para Satélites
            let mut dx = x - self.secondary_start_x;
            let mut dy = y - self.secondary_start_y;
            let dist_sq = dx * dx + dy * dy;

            // Umbral de arrastre (> 12 px) activa bloqueo y apuntado de satélites en 360°
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
                self.secondary_history_len = 0;
            } else {
                self.primary_id = -1;
                self.touch_initialized = false;
                self.is_touching = false;
                self.is_firing = false;
                self.satellite_lock = false;
                self.satellite_target_angle = None;
            }
        } else if self.secondary_id == id {
            // Ataque especial por toque rápido seco del segundo dedo:
            // Si el jugador presiona y suelta rápidamente el segundo dedo (< 0.35s) sin arrastrar satélites
            let dx = x - self.secondary_start_x;
            let dy = y - self.secondary_start_y;
            if self.secondary_touch_time < 0.35 && (dx * dx + dy * dy) < 400.0 && !self.secondary_has_dragged {
                self.trigger_bomb = true;
            }

            self.secondary_id = -1;
            self.satellite_lock = false;
            self.satellite_target_angle = None;
            self.secondary_has_dragged = false;
            self.secondary_touch_time = 0.0;
            self.secondary_history_len = 0;
        }
    }
}
