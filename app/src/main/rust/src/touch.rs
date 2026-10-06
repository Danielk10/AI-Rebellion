//! Módulo de Gestión de Experiencia Táctil Pura (Touch Puro, Sin Botones Virtuales)
//! Inspirado en el algoritmo del código Java original (toquePresionado, toqueDeslizando, toqueLevantado)
//! El jugador se desplaza suavemente por arrastre directo en cualquier punto de la pantalla,
//! dispara automáticamente al mantener el toque, fija satélites con un segundo dedo y
//! activa la bomba EMP con doble toque rápido (Double-Tap).

#[derive(Clone, Copy, Debug, Default)]
pub struct ActivePointer {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub start_x: f32,
    pub start_y: f32,
    pub last_x: f32,
    pub last_y: f32,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct TouchControls {
    pub pointers: [ActivePointer; 10],
    pub primary_id: i32,
    pub secondary_id: i32,

    // Acumulador de desplazamiento relativo de arrastre (delta touch)
    pub delta_x: f32,
    pub delta_y: f32,

    // Estado de acciones de juego
    pub is_touching: bool,
    pub is_firing: bool,
    pub satellite_lock: bool,
    pub satellite_target_angle: Option<f32>,
    pub trigger_bomb: bool,

    // Detección de doble toque (Double-Tap) para activar Bomba EMP
    pub last_tap_time: f32,
    pub time_since_last_tap: f32,

    pub screen_width: f32,
    pub screen_height: f32,
}

impl TouchControls {
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            pointers: [ActivePointer::default(); 10],
            primary_id: -1,
            secondary_id: -1,
            delta_x: 0.0,
            delta_y: 0.0,
            is_touching: false,
            is_firing: false,
            satellite_lock: false,
            satellite_target_angle: None,
            trigger_bomb: false,
            last_tap_time: 0.0,
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
        // Reset de bomba de un solo cuadro si ya fue consumida
        self.trigger_bomb = false;

        // Reset de deltas de movimiento para el cuadro actual
        self.delta_x = 0.0;
        self.delta_y = 0.0;
    }

    /// Evento: dedo presiona la pantalla
    pub fn on_touch_down(&mut self, id: i32, x: f32, y: f32) {
        let idx = (id.abs() as usize) % self.pointers.len();
        self.pointers[idx] = ActivePointer {
            id,
            x,
            y,
            start_x: x,
            start_y: y,
            last_x: x,
            last_y: y,
            active: true,
        };

        let active_count = self.count_active_pointers();

        // 1. Asignar puntero primario (control de movimiento y auto-fire)
        if self.primary_id == -1 {
            self.primary_id = id;
            self.is_touching = true;
            self.is_firing = true;

            // Detección de doble toque rápido (< 0.35s) para lanzar Bomba EMP
            if self.time_since_last_tap < 0.35 {
                self.trigger_bomb = true;
                self.time_since_last_tap = 10.0; // Reset
            } else {
                self.time_since_last_tap = 0.0;
            }
        } else if self.secondary_id == -1 && id != self.primary_id {
            // 2. Segundo dedo presionado -> Activar Satellite Lock y orientar satélites
            self.secondary_id = id;
            self.satellite_lock = true;
            self.update_satellite_aim();
        }

        if active_count >= 2 {
            self.satellite_lock = true;
        }
    }

    /// Evento: dedo se desliza por la pantalla (Algoritmo de arrastre relativo de Jugador.java)
    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) {
        let idx = (id.abs() as usize) % self.pointers.len();
        if self.pointers[idx].id == id && self.pointers[idx].active {
            let dx = x - self.pointers[idx].last_x;
            let dy = y - self.pointers[idx].last_y;

            self.pointers[idx].x = x;
            self.pointers[idx].y = y;
            self.pointers[idx].last_x = x;
            self.pointers[idx].last_y = y;

            // Si es el dedo principal, trasladar el desplazamiento al jugador
            if self.primary_id == id {
                self.delta_x += dx;
                self.delta_y += dy;
                self.is_firing = true;
            } else if self.secondary_id == id {
                // Segundo dedo ajusta el ángulo de fuego de los satélites
                self.update_satellite_aim();
            }
        }
    }

    /// Evento: dedo se levanta de la pantalla
    pub fn on_touch_up(&mut self, id: i32, _x: f32, _y: f32) {
        let idx = (id.abs() as usize) % self.pointers.len();
        self.pointers[idx].active = false;

        if self.primary_id == id {
            // Buscar si queda otro dedo activo para promoverlo a primario
            self.primary_id = -1;
            for p in self.pointers.iter() {
                if p.active {
                    self.primary_id = p.id;
                    break;
                }
            }
            if self.primary_id == -1 {
                self.is_touching = false;
                self.is_firing = false;
            }
        }

        if self.secondary_id == id {
            self.secondary_id = -1;
        }

        if self.count_active_pointers() < 2 {
            self.satellite_lock = false;
            self.satellite_target_angle = None;
        }
    }

    fn count_active_pointers(&self) -> usize {
        self.pointers.iter().filter(|p| p.active).count()
    }

    fn update_satellite_aim(&mut self) {
        let mut p_pos = None;
        let mut s_pos = None;

        for p in self.pointers.iter() {
            if p.active && p.id == self.primary_id {
                p_pos = Some((p.x, p.y));
            }
            if p.active && p.id == self.secondary_id {
                s_pos = Some((p.x, p.y));
            }
        }

        if let (Some((px, py)), Some((sx, sy))) = (p_pos, s_pos) {
            let dx = sx - px;
            let dy = sy - py;
            if dx.abs() > 5.0 || dy.abs() > 5.0 {
                self.satellite_target_angle = Some(dy.atan2(dx));
            }
        }
    }
}
