//! Módulo de Gestión Táctil para IA Rebellion (Inspirado en Final Mission NES)
//! Arquitectura 100% Nativa sin botones virtuales en pantalla:
//! - Primer dedo (dedo de movimiento - `primary_id`): Arrastre relativo 1:1 de Jugador.java (sin tapar el personaje).
//!   NO cambia nunca la dirección del personaje ni activa jamás el ataque especial, garantizando desplazamientos limpios.
//!   Si se cambia de dedo para mover, el nuevo dedo asume el movimiento suavemente y el otro pasa a ser el dedo libre.
//! - Segundo dedo (dedo libre - `secondary_id` - Exclusión Mutua):
//!   * Modo Satélites: Al mantener y arrastrar en 360°, bloquea y orienta los satélites sin cambiar la dirección del jugador.
//!   * Cambio de Dirección (Flick): Deslizamiento horizontal rápido hacia derecha o izquierda (< 0.40s).
//!   * Ataque Especial (Bomba EMP): Doble toque rápido (toques rápidos repetidos < 0.40s) exclusivamente con el dedo libre.

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

    // Estado del segundo dedo (dedo libre: satélites + dirección + doble toque de bomba)
    pub secondary_origin_x: f32,
    pub secondary_origin_y: f32,
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

    // Temporizador entre toques rápidos del dedo libre (secondary_id) para Bomba EMP
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
            secondary_origin_x: 0.0,
            secondary_origin_y: 0.0,
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

        if self.secondary_id != -1 {
            self.secondary_touch_time += dt;
        }
    }

    /// Evento toquePresionado (AMOTION_EVENT_ACTION_DOWN / POINTER_DOWN)
    pub fn on_touch_down(&mut self, id: i32, x: f32, y: f32, px: f32, py: f32) {
        if self.primary_id == -1 {
            // Dedo de movimiento: NUNCA activa el ataque especial (Bomba EMP)
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
            self.time_since_last_tap = 10.0;
        } else if self.secondary_id == -1 && id != self.primary_id {
            // Segundo dedo detectado (dedo libre):
            // Encargado de orientar el personaje (flick), apuntar satélites (arrastre)
            // y activar el ataque especial mediante toques rápidos (doble toque rápido).
            self.secondary_id = id;
            self.secondary_x = x;
            self.secondary_y = y;
            self.secondary_origin_x = x;
            self.secondary_origin_y = y;
            self.secondary_start_x = x;
            self.secondary_start_y = y;
            self.secondary_touch_time = 0.0;
            self.secondary_has_dragged = false;
        }
    }

    /// Evento toqueDeslizando (AMOTION_EVENT_ACTION_MOVE)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.primary_id == id {
            self.primary_x = x;
            self.primary_y = y;
            self.is_touching = true;
            self.is_firing = true;

            // El dedo de movimiento SOLO desplaza al jugador por la pantalla.
            // NO evalúa gestos flick ni activa jamás el ataque especial.
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

            // Control y apuntado de Satélites en 360 grados:
            // Al arrastrar, orienta los satélites en tiempo real hacia el ángulo del vector táctil.
            // IMPORTANTE: Durante on_touch_move NO se evalúa flick para garantizar
            // que girar satélites en círculos o a cualquier ángulo NUNCA cambie la dirección del jugador.
            if dist_sq > 100.0 {
                self.secondary_has_dragged = true;
                self.satellite_lock = true;

                // Clamping flotante para radio de 60px
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
                // Cuando se cambia de dedo para movimiento:
                // El segundo dedo pasa a ser el nuevo dedo de movimiento (primary_id)
                // y el dedo que quedó libre será ahora el dedo libre (secondary_id) al tocar la pantalla.
                self.primary_id = self.secondary_id;
                self.primary_x = self.secondary_x;
                self.primary_y = self.secondary_y;
                self.secondary_id = -1;
                self.touch_initialized = false;
                self.satellite_lock = false;
                self.satellite_target_angle = None;
                self.secondary_has_dragged = false;
                self.secondary_touch_time = 0.0;
                self.time_since_last_tap = 10.0;
            } else {
                self.primary_id = -1;
                self.touch_initialized = false;
                self.is_touching = false;
                self.is_firing = false;
                self.satellite_lock = false;
                self.satellite_target_angle = None;
                self.time_since_last_tap = 10.0;
            }
        } else if self.secondary_id == id {
            let total_dx = x - self.secondary_origin_x;
            let total_dy = y - self.secondary_origin_y;
            let total_dist_sq = total_dx * total_dx + total_dy * total_dy;
            let touch_time = self.secondary_touch_time;

            // ==============================================================
            // ACCIONES EXCLUSIVAS DEL DEDO LIBRE (secondary_id):
            // 1. Gesto rápido (< 0.40s):
            //    - Desplazamiento horizontal (Flick / Swipe >= 25px): cambia la orientación (derecha/izquierda).
            //    - Toques rápidos sin desplazamiento (< 20px): doble toque rápido (< 0.40s entre toques)
            //      lanza la Bomba Especial EMP. Un solo toque aislado NO dispara la bomba por accidente.
            // 2. Toque sostenido (>= 0.40s) o arrastre de satélites:
            //    - Libera el bloqueo de satélites sin cambiar dirección ni activar bomba.
            // ==============================================================
            if touch_time < 0.40 {
                if total_dx.abs() >= 25.0 && total_dx.abs() >= total_dy.abs() * 1.1 {
                    // Latigazo horizontal rápido del dedo libre (Flick) -> Derecha / Izquierda
                    self.flick_facing = Some(total_dx > 0.0);
                    self.flick_cooldown = 0.20;
                    self.time_since_last_tap = 10.0;
                } else if total_dist_sq < 400.0 {
                    // Toque rápido en el sitio (< 20 px) con el dedo libre:
                    // Si es el segundo toque rápido consecutivo (< 0.40s), activa el ataque especial EMP
                    if self.time_since_last_tap < 0.40 {
                        self.trigger_bomb = true;
                        self.time_since_last_tap = 10.0;
                    } else {
                        self.time_since_last_tap = 0.0;
                    }
                } else {
                    self.time_since_last_tap = 10.0;
                }
            } else {
                self.time_since_last_tap = 10.0;
            }

            // Liberar estado del segundo dedo (dedo libre)
            self.secondary_id = -1;
            self.satellite_lock = false;
            self.satellite_target_angle = None;
            self.secondary_has_dragged = false;
            self.secondary_touch_time = 0.0;
        }
    }
}
