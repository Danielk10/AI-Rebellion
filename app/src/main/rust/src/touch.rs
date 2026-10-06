//! Módulo de Gestión de Controles Táctiles Multi-touch
//! Incluye palanca virtual flotante, botones de acción (disparo, bomba, fijación de satélites)

#[derive(Clone, Copy, Debug, Default)]
pub struct TouchPointer {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub down: bool,
}

#[derive(Clone, Debug)]
pub struct TouchControls {
    pub pointers: [TouchPointer; 10],

    // Estado del joystick virtual
    pub joystick_active: bool,
    pub joystick_pointer_id: i32,
    pub joystick_base_x: f32,
    pub joystick_base_y: f32,
    pub joystick_current_x: f32,
    pub joystick_current_y: f32,

    // Entradas analógicas normalizadas (-1.0 a 1.0)
    pub move_x: f32,
    pub move_y: f32,

    // Estado de botones de acción
    pub btn_fire: bool,
    pub btn_special_bomb: bool,
    pub btn_satellite_lock: bool,
    pub btn_pause: bool,

    // Configuración visual de botones
    pub screen_width: f32,
    pub screen_height: f32,
}

impl TouchControls {
    pub fn new(w: f32, h: f32) -> Self {
        Self {
            pointers: [TouchPointer::default(); 10],
            joystick_active: false,
            joystick_pointer_id: -1,
            joystick_base_x: 200.0,
            joystick_base_y: h - 200.0,
            joystick_current_x: 200.0,
            joystick_current_y: h - 200.0,
            move_x: 0.0,
            move_y: 0.0,
            btn_fire: false,
            btn_special_bomb: false,
            btn_satellite_lock: false,
            btn_pause: false,
            screen_width: w,
            screen_height: h,
        }
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.screen_width = w;
        self.screen_height = h;
        if !self.joystick_active {
            self.joystick_base_x = 220.0;
            self.joystick_base_y = h - 220.0;
            self.joystick_current_x = self.joystick_base_x;
            self.joystick_current_y = self.joystick_base_y;
        }
    }

    pub fn on_touch_down(&mut self, id: i32, x: f32, y: f32) {
        let idx = (id.abs() as usize) % self.pointers.len();
        self.pointers[idx] = TouchPointer { id, x, y, down: true };

        // Mitad izquierda de la pantalla -> Joystick virtual
        if x < self.screen_width * 0.5 {
            if !self.joystick_active {
                self.joystick_active = true;
                self.joystick_pointer_id = id;
                self.joystick_base_x = x;
                self.joystick_base_y = y;
                self.joystick_current_x = x;
                self.joystick_current_y = y;
                self.update_joystick();
            }
        } else {
            // Mitad derecha -> Botones de acción
            self.check_action_buttons(x, y, true);
        }
    }

    pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) {
        let idx = (id.abs() as usize) % self.pointers.len();
        if self.pointers[idx].id == id {
            self.pointers[idx].x = x;
            self.pointers[idx].y = y;
        }

        if self.joystick_active && self.joystick_pointer_id == id {
            self.joystick_current_x = x;
            self.joystick_current_y = y;
            self.update_joystick();
        } else if x >= self.screen_width * 0.5 {
            self.check_action_buttons(x, y, true);
        }
    }

    pub fn on_touch_up(&mut self, id: i32, x: f32, y: f32) {
        let idx = (id.abs() as usize) % self.pointers.len();
        self.pointers[idx].down = false;

        if self.joystick_active && self.joystick_pointer_id == id {
            self.joystick_active = false;
            self.joystick_pointer_id = -1;
            self.move_x = 0.0;
            self.move_y = 0.0;
        }

        if x >= self.screen_width * 0.5 {
            self.check_action_buttons(x, y, false);
        }
    }

    fn update_joystick(&mut self) {
        let dx = self.joystick_current_x - self.joystick_base_x;
        let dy = self.joystick_current_y - self.joystick_base_y;
        let max_radius = 120.0f32;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 0.0 {
            let clamped_dist = dist.min(max_radius);
            self.move_x = (dx / dist) * (clamped_dist / max_radius);
            self.move_y = (dy / dist) * (clamped_dist / max_radius);
        } else {
            self.move_x = 0.0;
            self.move_y = 0.0;
        }
    }

    fn check_action_buttons(&mut self, x: f32, y: f32, is_down: bool) {
        let w = self.screen_width;
        let h = self.screen_height;

        // Botón Disparo (Gran botón circular abajo a la derecha)
        let fire_cx = w - 160.0;
        let fire_cy = h - 160.0;
        if ((x - fire_cx).powi(2) + (y - fire_cy).powi(2)).sqrt() < 100.0 {
            self.btn_fire = is_down;
        }

        // Botón Bomba Especial (Arriba del botón de disparo)
        let bomb_cx = w - 290.0;
        let bomb_cy = h - 230.0;
        if ((x - bomb_cx).powi(2) + (y - bomb_cy).powi(2)).sqrt() < 65.0 {
            self.btn_special_bomb = is_down;
        }

        // Botón Satélites: Bloqueo de Ángulo / Modo Libre (Estilo Final Mission)
        let sat_cx = w - 130.0;
        let sat_cy = h - 310.0;
        if ((x - sat_cx).powi(2) + (y - sat_cy).powi(2)).sqrt() < 65.0 {
            self.btn_satellite_lock = is_down;
        }

        // Botón Pausa (Esquina superior derecha)
        if x > w - 100.0 && y < 100.0 {
            self.btn_pause = is_down;
        }
    }
}
