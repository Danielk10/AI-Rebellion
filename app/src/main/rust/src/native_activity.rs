//! Arquitectura 100% Nativa de Android en Rust (C ABI / ANativeActivity)
//! Implementación directa del motor para Android sin intermediación de Java/JNI.
//!
//! Controla el ciclo de vida de ANativeActivity, renderizado directo sobre
//! ANativeWindow respetando el stride de hardware, y eventos táctiles de baja
//! latencia vía AInputQueue para controles 1:1, auto-disparo, EMP y satélites.

use std::ffi::{c_char, c_int, c_void, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::game::Game;

// ==============================================================================
// Constantes de Resolución Virtual y Temporización
// ==============================================================================
pub const VIRTUAL_WIDTH: usize = 960;
pub const VIRTUAL_HEIGHT: usize = 540;
pub const TARGET_FPS: f32 = 60.0;
pub const TARGET_FRAME_DURATION_NS: u64 = 16_666_667; // ~16.67 ms (60 FPS)

// ==============================================================================
// Android NDK C ABI: Constantes del Sistema
// ==============================================================================
pub const ANDROID_LOG_INFO: c_int = 4;
pub const ANDROID_LOG_WARN: c_int = 5;
pub const ANDROID_LOG_ERROR: c_int = 6;

pub const WINDOW_FORMAT_RGBA_8888: i32 = 1;
pub const WINDOW_FORMAT_RGBX_8888: i32 = 2;
pub const WINDOW_FORMAT_RGB_565: i32 = 4;

pub const AINPUT_EVENT_TYPE_KEY: i32 = 1;
pub const AINPUT_EVENT_TYPE_MOTION: i32 = 2;

pub const AMOTION_EVENT_ACTION_MASK: i32 = 0x00ff;
pub const AMOTION_EVENT_ACTION_POINTER_INDEX_MASK: i32 = 0xff00;
pub const AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT: i32 = 8;

pub const AMOTION_EVENT_ACTION_DOWN: i32 = 0;
pub const AMOTION_EVENT_ACTION_UP: i32 = 1;
pub const AMOTION_EVENT_ACTION_MOVE: i32 = 2;
pub const AMOTION_EVENT_ACTION_CANCEL: i32 = 3;
pub const AMOTION_EVENT_ACTION_OUTSIDE: i32 = 4;
pub const AMOTION_EVENT_ACTION_POINTER_DOWN: i32 = 5;
pub const AMOTION_EVENT_ACTION_POINTER_UP: i32 = 6;
pub const AMOTION_EVENT_ACTION_HOVER_MOVE: i32 = 7;
pub const AMOTION_EVENT_ACTION_SCROLL: i32 = 8;

// ==============================================================================
// Android NDK C ABI: Tipos y Estructuras
// ==============================================================================

#[repr(C)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct ANativeActivityCallbacks {
    pub on_start: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_resume: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_save_instance_state: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, out_size: *mut usize) -> *mut c_void>,
    pub on_pause: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_stop: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_destroy: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_window_focus_changed: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, has_focus: c_int)>,
    pub on_native_window_created: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow)>,
    pub on_native_window_resized: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow)>,
    pub on_native_window_redraw_needed: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow)>,
    pub on_native_window_destroyed: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow)>,
    pub on_input_queue_created: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue)>,
    pub on_input_queue_destroyed: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue)>,
    pub on_content_rect_changed: Option<unsafe extern "C" fn(activity: *mut ANativeActivity, rect: *const ARect)>,
    pub on_configuration_changed: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_low_memory: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
}

#[repr(C)]
pub struct ANativeActivity {
    pub callbacks: *mut ANativeActivityCallbacks,
    pub vm: *mut c_void,
    pub env: *mut c_void,
    pub clazz: *mut c_void,
    pub internal_data_path: *const c_char,
    pub external_data_path: *const c_char,
    pub sdk_version: i32,
    pub instance: *mut c_void,
    pub asset_manager: *mut c_void,
    pub obb_path: *const c_char,
}

#[repr(C)]
pub struct ANativeWindow_Buffer {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub bits: *mut c_void,
    pub reserved: [u32; 6],
}

#[derive(Copy, Clone, Debug)]
pub enum ANativeWindow {}
#[derive(Copy, Clone, Debug)]
pub enum AInputQueue {}
#[derive(Copy, Clone, Debug)]
pub enum AInputEvent {}

// ==============================================================================
// Android NDK C ABI: Enlaces a libandroid.so y liblog.so
// ==============================================================================
#[link(name = "log")]
extern "C" {
    pub fn __android_log_print(prio: c_int, tag: *const c_char, fmt: *const c_char, ...) -> c_int;
}

pub const AWINDOW_FLAG_FULLSCREEN: u32 = 0x00000400;
pub const AWINDOW_FLAG_KEEP_SCREEN_ON: u32 = 0x00000080;
pub const AWINDOW_FLAG_LAYOUT_IN_SCREEN: u32 = 0x00000100;
pub const AWINDOW_FLAG_LAYOUT_NO_LIMITS: u32 = 0x00000200;
pub const AWINDOW_FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS: u32 = 0x80000000;

#[link(name = "android")]
extern "C" {
    pub fn ANativeActivity_setWindowFlags(
        activity: *mut ANativeActivity,
        add_flags: u32,
        remove_flags: u32,
    );
    pub fn ANativeWindow_acquire(window: *mut ANativeWindow);
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
    pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_getFormat(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_setBuffersGeometry(
        window: *mut ANativeWindow,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
    pub fn ANativeWindow_lock(
        window: *mut ANativeWindow,
        out_buffer: *mut ANativeWindow_Buffer,
        in_out_dirty_bounds: *mut ARect,
    ) -> i32;
    pub fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;

    pub fn AInputQueue_hasEvents(queue: *mut AInputQueue) -> i32;
    pub fn AInputQueue_getEvent(queue: *mut AInputQueue, out_event: *mut *mut AInputEvent) -> i32;
    pub fn AInputQueue_preDispatchEvent(queue: *mut AInputQueue, event: *mut AInputEvent) -> i32;
    pub fn AInputQueue_finishEvent(queue: *mut AInputQueue, event: *mut AInputEvent, handled: i32);

    pub fn AInputEvent_getType(event: *const AInputEvent) -> i32;
    pub fn AInputEvent_getDeviceId(event: *const AInputEvent) -> i32;
    pub fn AInputEvent_getSource(event: *const AInputEvent) -> i32;

    pub fn AMotionEvent_getAction(event: *const AInputEvent) -> i32;
    pub fn AMotionEvent_getFlags(event: *const AInputEvent) -> i32;
    pub fn AMotionEvent_getPointerCount(event: *const AInputEvent) -> usize;
    pub fn AMotionEvent_getPointerId(event: *const AInputEvent, pointer_index: usize) -> i32;
    pub fn AMotionEvent_getX(event: *const AInputEvent, pointer_index: usize) -> f32;
    pub fn AMotionEvent_getY(event: *const AInputEvent, pointer_index: usize) -> f32;
}

// ==============================================================================
// Macros de Logging Nativo (Logcat)
// ==============================================================================
macro_rules! log_info {
    ($($arg:tt)*) => {{
        if let Ok(c_tag) = CString::new("AIRebellionNative") {
            let msg = format!($($arg)*);
            if let Ok(c_msg) = CString::new(msg) {
                if let Ok(c_fmt) = CString::new("%s") {
                    unsafe {
                        __android_log_print(ANDROID_LOG_INFO, c_tag.as_ptr(), c_fmt.as_ptr(), c_msg.as_ptr());
                    }
                }
            }
        }
    }};
}

macro_rules! log_error {
    ($($arg:tt)*) => {{
        if let Ok(c_tag) = CString::new("AIRebellionNative") {
            let msg = format!($($arg)*);
            if let Ok(c_msg) = CString::new(msg) {
                if let Ok(c_fmt) = CString::new("%s") {
                    unsafe {
                        __android_log_print(ANDROID_LOG_ERROR, c_tag.as_ptr(), c_fmt.as_ptr(), c_msg.as_ptr());
                    }
                }
            }
        }
    }};
}

// ==============================================================================
// Envoltorios Send/Sync para Punteros Crudos NDK
// ==============================================================================
#[derive(PartialEq, Eq, Debug)]
pub struct SendPtr<T>(pub *mut T);

impl<T> Copy for SendPtr<T> {}
impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}
unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

// ==============================================================================
// Motor Nativo: Estado y Sincronización de Hilos
// ==============================================================================
pub struct NativeEngine {
    pub running: Arc<AtomicBool>,
    pub has_focus: Arc<AtomicBool>,
    pub is_paused: Arc<AtomicBool>,
    pub window: Arc<Mutex<Option<SendPtr<ANativeWindow>>>>,
    pub window_dims: Arc<Mutex<(f32, f32)>>,
    pub input_queue: Arc<Mutex<Option<SendPtr<AInputQueue>>>>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    audio_thread_handle: Mutex<Option<JoinHandle<()>>>,
}

unsafe impl Send for NativeEngine {}
unsafe impl Sync for NativeEngine {}

impl NativeEngine {
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let has_focus = Arc::new(AtomicBool::new(true));
        let is_paused = Arc::new(AtomicBool::new(false));
        let window = Arc::new(Mutex::new(None));
        let window_dims = Arc::new(Mutex::new((VIRTUAL_WIDTH as f32, VIRTUAL_HEIGHT as f32)));
        let input_queue = Arc::new(Mutex::new(None));

        let audio_engine = Arc::new(Mutex::new(crate::audio::AudioEngine::new()));

        let t_running = Arc::clone(&running);
        let t_focus = Arc::clone(&has_focus);
        let t_paused = Arc::clone(&is_paused);
        let t_window = Arc::clone(&window);
        let t_dims = Arc::clone(&window_dims);
        let t_input = Arc::clone(&input_queue);
        let t_audio = Arc::clone(&audio_engine);

        let thread_handle = thread::Builder::new()
            .name("NativeRenderThread".to_string())
            .spawn(move || {
                render_loop(t_running, t_focus, t_paused, t_window, t_dims, t_input, t_audio);
            })
            .ok();

        let a_running = Arc::clone(&running);
        let a_paused = Arc::clone(&is_paused);
        let a_audio = Arc::clone(&audio_engine);

        let audio_thread_handle = thread::Builder::new()
            .name("NativeAudioThread".to_string())
            .spawn(move || {
                crate::audio::native_playback_loop(a_running, a_paused, a_audio);
            })
            .ok();

        Self {
            running,
            has_focus,
            is_paused,
            window,
            window_dims,
            input_queue,
            thread_handle: Mutex::new(thread_handle),
            audio_thread_handle: Mutex::new(audio_thread_handle),
        }
    }

    pub fn set_window(&self, window: *mut ANativeWindow) {
        if window.is_null() {
            return;
        }
        unsafe {
            let phys_w = ANativeWindow_getWidth(window) as f32;
            let phys_h = ANativeWindow_getHeight(window) as f32;

            if let Ok(mut dims) = self.window_dims.lock() {
                *dims = (phys_w.max(1.0), phys_h.max(1.0));
            }

            // Calcular resolución virtual proporcional al aspect ratio real del dispositivo
            // para cubrir el 100% de la pantalla sin franjas negras laterales ni superiores
            let aspect = phys_w / phys_h.max(1.0);
            let virt_h = VIRTUAL_HEIGHT;
            let virt_w = ((virt_h as f32 * aspect).round() as usize).max(VIRTUAL_WIDTH);

            ANativeWindow_setBuffersGeometry(
                window,
                virt_w as i32,
                virt_h as i32,
                WINDOW_FORMAT_RGBA_8888,
            );
        }

        if let Ok(mut win_lock) = self.window.lock() {
            if let Some(old_win) = *win_lock {
                if old_win.0 == window {
                    return;
                }
                unsafe {
                    ANativeWindow_release(old_win.0);
                }
            }
            unsafe {
                ANativeWindow_acquire(window);
            }
            *win_lock = Some(SendPtr(window));
        }
    }

    pub fn destroy_window(&self, window: *mut ANativeWindow) {
        // Al bloquear self.window garantizamos que el hilo de render no esté activo en lockCanvas
        if let Ok(mut win_lock) = self.window.lock() {
            if let Some(curr_win) = *win_lock {
                if curr_win.0 == window || window.is_null() {
                    *win_lock = None;
                    unsafe {
                        ANativeWindow_release(curr_win.0);
                    }
                }
            }
        }
    }

    pub fn set_input_queue(&self, queue: *mut AInputQueue) {
        if let Ok(mut q_lock) = self.input_queue.lock() {
            *q_lock = Some(SendPtr(queue));
        }
    }

    pub fn destroy_input_queue(&self, queue: *mut AInputQueue) {
        if let Ok(mut q_lock) = self.input_queue.lock() {
            if let Some(curr_q) = *q_lock {
                if curr_q.0 == queue || queue.is_null() {
                    *q_lock = None;
                }
            }
        }
    }

    pub fn shutdown(self: Box<Self>) {
        self.running.store(false, Ordering::SeqCst);

        // Desvincular ventana y cola de entrada
        if let Ok(mut win_lock) = self.window.lock() {
            if let Some(win) = win_lock.take() {
                unsafe {
                    ANativeWindow_release(win.0);
                }
            }
        }
        if let Ok(mut q_lock) = self.input_queue.lock() {
            *q_lock = None;
        }

        // Esperar finalización del hilo de renderizado
        if let Ok(mut handle) = self.thread_handle.lock() {
            if let Some(th) = handle.take() {
                let _ = th.join();
            }
        }

        // Esperar finalización del hilo de audio nativo
        if let Ok(mut handle) = self.audio_thread_handle.lock() {
            if let Some(th) = handle.take() {
                let _ = th.join();
            }
        }
    }
}

// ==============================================================================
// Bucle Principal de Renderizado y Entrada (Native Game Loop)
// ==============================================================================
fn render_loop(
    running: Arc<AtomicBool>,
    has_focus: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    window_arc: Arc<Mutex<Option<SendPtr<ANativeWindow>>>>,
    dims_arc: Arc<Mutex<(f32, f32)>>,
    input_arc: Arc<Mutex<Option<SendPtr<AInputQueue>>>>,
    audio_engine: Arc<Mutex<crate::audio::AudioEngine>>,
) {
    log_info!("🎮 Bucle de renderizado nativo en Rust iniciado (60 FPS objetivo)");

    let mut game = Game::new_with_audio(VIRTUAL_WIDTH, VIRTUAL_HEIGHT, audio_engine);
    let mut pixel_buffer = vec![0u32; VIRTUAL_WIDTH * VIRTUAL_HEIGHT];

    let target_frame_duration = Duration::from_nanos(TARGET_FRAME_DURATION_NS);
    let mut last_frame_instant = Instant::now();

    while running.load(Ordering::SeqCst) {
        let frame_start = Instant::now();

        // 1. Obtener ventana activa si existe (sin pánico si mutex está envenenado)
        let active_window = window_arc.lock().ok().and_then(|g| *g);

        if active_window.is_none() {
            // Sin ventana válida (actividad en segundo plano), reposo térmico de 100 ms
            thread::sleep(Duration::from_millis(100));
            last_frame_instant = Instant::now();
            continue;
        }

        // Si la actividad está en pausa, reposo térmico sin actualizar física de juego
        if is_paused.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
            last_frame_instant = Instant::now();
            continue;
        }

        let win = active_window.unwrap().0;

        // 2. Procesar eventos táctiles nativos de AInputQueue sin bloqueo
        {
            let (win_w, win_h) = dims_arc
                .lock()
                .ok()
                .map(|dims| *dims)
                .unwrap_or((VIRTUAL_WIDTH as f32, VIRTUAL_HEIGHT as f32));

            // Adaptar resolución interna del juego al aspecto físico del dispositivo (100% de pantalla)
            let aspect = win_w / win_h.max(1.0);
            let target_virt_h = VIRTUAL_HEIGHT;
            let target_virt_w = ((target_virt_h as f32 * aspect).round() as usize).max(VIRTUAL_WIDTH);

            if game.width != target_virt_w || game.height != target_virt_h {
                game.resize(target_virt_w, target_virt_h);
            }

            if let Ok(q_lock) = input_arc.lock() {
                if let Some(queue_ptr) = *q_lock {
                    let queue = queue_ptr.0;
                    unsafe {
                        while AInputQueue_hasEvents(queue) > 0 {
                            let mut event: *mut AInputEvent = std::ptr::null_mut();
                            if AInputQueue_getEvent(queue, &mut event) >= 0 && !event.is_null() {
                                // Si preDispatchEvent retorna != 0, el evento fue pre-despachado
                                // y NO se debe llamar a finishEvent sobre él.
                                if AInputQueue_preDispatchEvent(queue, event) != 0 {
                                    continue;
                                }
                                let event_type = AInputEvent_getType(event);

                                let handled = if event_type == AINPUT_EVENT_TYPE_MOTION {
                                    let gw = game.width;
                                    let gh = game.height;
                                    handle_input_event(
                                        event,
                                        &mut game,
                                        win_w,
                                        win_h,
                                        gw,
                                        gh,
                                    );
                                    1
                                } else {
                                    // Permitir que Android procese teclas de sistema (Volumen, Atrás)
                                    0
                                };

                                AInputQueue_finishEvent(queue, event, handled);
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 3. Actualizar simulación física y lógica de juego
        let now = Instant::now();
        let dt = now.duration_since(last_frame_instant).as_secs_f32().clamp(0.0001, 0.1);
        last_frame_instant = now;

        let focused = has_focus.load(Ordering::SeqCst);
        if focused {
            game.update(dt);
        }

        // 4. Rasterizar el fotograma completo a pixel_buffer ARGB8888
        let required_len = game.width * game.height;
        if pixel_buffer.len() != required_len {
            pixel_buffer.resize(required_len, 0);
        }
        game.render(&mut pixel_buffer);

        // 5. Bloquear ANativeWindow y copiar directamente los píxeles respetando el stride
        if let Ok(win_lock) = window_arc.lock() {
            if let Some(curr_win) = *win_lock {
                if curr_win.0 == win {
                    unsafe {
                        let mut out_buffer: ANativeWindow_Buffer = std::mem::zeroed();
                        let lock_res = ANativeWindow_lock(win, &mut out_buffer, std::ptr::null_mut());
                        if lock_res == 0
                            && !out_buffer.bits.is_null()
                            && out_buffer.width > 0
                            && out_buffer.height > 0
                            && out_buffer.stride >= out_buffer.width
                        {
                            let stride = out_buffer.stride as usize;
                            let copy_w = (out_buffer.width as usize).min(game.width);
                            let copy_h = (out_buffer.height as usize).min(game.height);
                            let dst_ptr = out_buffer.bits as *mut u32;

                            // Copia de píxeles respetando el stride de hardware:
                            // ARGB (0xAARRGGBB) se convierte a RGBA_8888 (0xAABBGGRR en little-endian)
                            // mediante permutación de canales R y B en tiempo de volcado.
                            for y in 0..copy_h {
                                let src_offset = y * game.width;
                                let dst_row = dst_ptr.add(y * stride);
                                for x in 0..copy_w {
                                    let argb = pixel_buffer[src_offset + x];
                                    let rgba = (argb & 0xFF00FF00)
                                        | ((argb & 0x00FF0000) >> 16)
                                        | ((argb & 0x000000FF) << 16);
                                    *dst_row.add(x) = rgba;
                                }
                            }

                            ANativeWindow_unlockAndPost(win);
                        }
                    }
                }
            }
        }

        // 6. Regular fotogramas a 60 FPS
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_duration {
            thread::sleep(target_frame_duration - elapsed);
        }
    }

    log_info!("🛑 Bucle de renderizado nativo finalizado");
}

// ==============================================================================
// Procesamiento de Eventos Táctiles Nativos (AInputQueue -> touch_controls)
// ==============================================================================
unsafe fn handle_input_event(
    event: *mut AInputEvent,
    game: &mut Game,
    win_w: f32,
    win_h: f32,
    virt_w: usize,
    virt_h: usize,
) {
    let action_full = AMotionEvent_getAction(event);
    let action = action_full & AMOTION_EVENT_ACTION_MASK;
    let pointer_index = ((action_full & AMOTION_EVENT_ACTION_POINTER_INDEX_MASK)
        >> AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT) as usize;

    let scale_x = if win_w > 0.0 { (virt_w as f32) / win_w } else { 1.0 };
    let scale_y = if win_h > 0.0 { (virt_h as f32) / win_h } else { 1.0 };

    match action {
        AMOTION_EVENT_ACTION_DOWN | AMOTION_EVENT_ACTION_POINTER_DOWN => {
            let count = AMotionEvent_getPointerCount(event);
            if pointer_index < count {
                let pointer_id = AMotionEvent_getPointerId(event, pointer_index);
                let raw_x = AMotionEvent_getX(event, pointer_index);
                let raw_y = AMotionEvent_getY(event, pointer_index);
                let x = (raw_x * scale_x).clamp(0.0, virt_w as f32);
                let y = (raw_y * scale_y).clamp(0.0, virt_h as f32);

                game.on_touch_down(pointer_id, x, y);
            }
        }

        AMOTION_EVENT_ACTION_MOVE => {
            let count = AMotionEvent_getPointerCount(event);
            for i in 0..count {
                let pointer_id = AMotionEvent_getPointerId(event, i);
                let raw_x = AMotionEvent_getX(event, i);
                let raw_y = AMotionEvent_getY(event, i);
                let x = (raw_x * scale_x).clamp(0.0, virt_w as f32);
                let y = (raw_y * scale_y).clamp(0.0, virt_h as f32);

                game.on_touch_move(pointer_id, x, y);
            }
        }

        AMOTION_EVENT_ACTION_UP | AMOTION_EVENT_ACTION_POINTER_UP => {
            let count = AMotionEvent_getPointerCount(event);
            if pointer_index < count {
                let pointer_id = AMotionEvent_getPointerId(event, pointer_index);
                let raw_x = AMotionEvent_getX(event, pointer_index);
                let raw_y = AMotionEvent_getY(event, pointer_index);
                let x = (raw_x * scale_x).clamp(0.0, virt_w as f32);
                let y = (raw_y * scale_y).clamp(0.0, virt_h as f32);

                game.on_touch_up(pointer_id, x, y);
            }
        }

        AMOTION_EVENT_ACTION_CANCEL => {
            let count = AMotionEvent_getPointerCount(event);
            for i in 0..count {
                let pointer_id = AMotionEvent_getPointerId(event, i);
                let raw_x = AMotionEvent_getX(event, i);
                let raw_y = AMotionEvent_getY(event, i);
                let x = (raw_x * scale_x).clamp(0.0, virt_w as f32);
                let y = (raw_y * scale_y).clamp(0.0, virt_h as f32);

                game.on_touch_up(pointer_id, x, y);
            }
        }

        _ => {}
    }
}

// ==============================================================================
// Callbacks C ABI de ANativeActivity
// ==============================================================================
unsafe fn get_engine<'a>(activity: *mut ANativeActivity) -> Option<&'a NativeEngine> {
    if activity.is_null() || (*activity).instance.is_null() {
        None
    } else {
        Some(&*((*activity).instance as *mut NativeEngine))
    }
}

unsafe extern "C" fn on_start(_activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onStart");
}

unsafe extern "C" fn on_resume(activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onResume");
    if let Some(engine) = get_engine(activity) {
        engine.is_paused.store(false, Ordering::SeqCst);
    }
}

unsafe extern "C" fn on_save_instance_state(
    _activity: *mut ANativeActivity,
    out_size: *mut usize,
) -> *mut c_void {
    log_info!("ANativeActivity::onSaveInstanceState");
    if !out_size.is_null() {
        *out_size = 0;
    }
    std::ptr::null_mut()
}

unsafe extern "C" fn on_pause(activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onPause");
    if let Some(engine) = get_engine(activity) {
        engine.is_paused.store(true, Ordering::SeqCst);
    }
}

unsafe extern "C" fn on_stop(_activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onStop");
}

unsafe extern "C" fn on_destroy(activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onDestroy");
    if activity.is_null() || (*activity).instance.is_null() {
        return;
    }
    let engine = Box::from_raw((*activity).instance as *mut NativeEngine);
    (*activity).instance = std::ptr::null_mut();
    engine.shutdown();
}

unsafe extern "C" fn on_window_focus_changed(activity: *mut ANativeActivity, has_focus: c_int) {
    log_info!("ANativeActivity::onWindowFocusChanged(hasFocus={})", has_focus);
    if has_focus != 0 && !activity.is_null() {
        let add_flags = AWINDOW_FLAG_FULLSCREEN
            | AWINDOW_FLAG_KEEP_SCREEN_ON
            | AWINDOW_FLAG_LAYOUT_IN_SCREEN
            | AWINDOW_FLAG_LAYOUT_NO_LIMITS
            | AWINDOW_FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS;
        ANativeActivity_setWindowFlags(activity, add_flags, 0);
    }
    if let Some(engine) = get_engine(activity) {
        engine.has_focus.store(has_focus != 0, Ordering::SeqCst);
    }
}

unsafe extern "C" fn on_native_window_created(
    activity: *mut ANativeActivity,
    window: *mut ANativeWindow,
) {
    log_info!("ANativeActivity::onNativeWindowCreated({:p})", window);
    if let Some(engine) = get_engine(activity) {
        engine.set_window(window);
    }
}

unsafe extern "C" fn on_native_window_resized(
    activity: *mut ANativeActivity,
    window: *mut ANativeWindow,
) {
    log_info!("ANativeActivity::onNativeWindowResized({:p})", window);
    if let Some(engine) = get_engine(activity) {
        let w = ANativeWindow_getWidth(window) as f32;
        let h = ANativeWindow_getHeight(window) as f32;
        if let Ok(mut dims) = engine.window_dims.lock() {
            *dims = (w.max(1.0), h.max(1.0));
        }

        let aspect = w / h.max(1.0);
        let virt_h = VIRTUAL_HEIGHT;
        let virt_w = ((virt_h as f32 * aspect).round() as usize).max(VIRTUAL_WIDTH);
        ANativeWindow_setBuffersGeometry(
            window,
            virt_w as i32,
            virt_h as i32,
            WINDOW_FORMAT_RGBA_8888,
        );
    }
}

unsafe extern "C" fn on_native_window_redraw_needed(
    _activity: *mut ANativeActivity,
    window: *mut ANativeWindow,
) {
    log_info!("ANativeActivity::onNativeWindowRedrawNeeded({:p})", window);
}

unsafe extern "C" fn on_native_window_destroyed(
    activity: *mut ANativeActivity,
    window: *mut ANativeWindow,
) {
    log_info!("ANativeActivity::onNativeWindowDestroyed({:p})", window);
    if let Some(engine) = get_engine(activity) {
        engine.destroy_window(window);
    }
}

unsafe extern "C" fn on_input_queue_created(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    log_info!("ANativeActivity::onInputQueueCreated({:p})", queue);
    if let Some(engine) = get_engine(activity) {
        engine.set_input_queue(queue);
    }
}

unsafe extern "C" fn on_input_queue_destroyed(
    activity: *mut ANativeActivity,
    queue: *mut AInputQueue,
) {
    log_info!("ANativeActivity::onInputQueueDestroyed({:p})", queue);
    if let Some(engine) = get_engine(activity) {
        engine.destroy_input_queue(queue);
    }
}

unsafe extern "C" fn on_content_rect_changed(
    _activity: *mut ANativeActivity,
    _rect: *const ARect,
) {
}

unsafe extern "C" fn on_configuration_changed(_activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onConfigurationChanged");
}

unsafe extern "C" fn on_low_memory(_activity: *mut ANativeActivity) {
    log_info!("ANativeActivity::onLowMemory");
}

// ==============================================================================
// Punto de Entrada Nativo Principal: ANativeActivity_onCreate
// ==============================================================================
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    _saved_state: *mut c_void,
    _saved_state_size: usize,
) {
    log_info!("🚀 ANativeActivity_onCreate: Inicializando arquitectura 100% nativa en Rust");

    if activity.is_null() || (*activity).callbacks.is_null() {
        log_error!("❌ Error fatal: Puntero activity o callbacks nulo en ANativeActivity_onCreate");
        return;
    }

    // Configurar banderas de ventana para cubrir el 100% de la pantalla (notch / display cutout)
    let add_flags = AWINDOW_FLAG_FULLSCREEN
        | AWINDOW_FLAG_KEEP_SCREEN_ON
        | AWINDOW_FLAG_LAYOUT_IN_SCREEN
        | AWINDOW_FLAG_LAYOUT_NO_LIMITS
        | AWINDOW_FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS;
    ANativeActivity_setWindowFlags(activity, add_flags, 0);

    let callbacks = &mut *(*activity).callbacks;
    callbacks.on_start = Some(on_start);
    callbacks.on_resume = Some(on_resume);
    callbacks.on_save_instance_state = Some(on_save_instance_state);
    callbacks.on_pause = Some(on_pause);
    callbacks.on_stop = Some(on_stop);
    callbacks.on_destroy = Some(on_destroy);
    callbacks.on_window_focus_changed = Some(on_window_focus_changed);
    callbacks.on_native_window_created = Some(on_native_window_created);
    callbacks.on_native_window_resized = Some(on_native_window_resized);
    callbacks.on_native_window_redraw_needed = Some(on_native_window_redraw_needed);
    callbacks.on_native_window_destroyed = Some(on_native_window_destroyed);
    callbacks.on_input_queue_created = Some(on_input_queue_created);
    callbacks.on_input_queue_destroyed = Some(on_input_queue_destroyed);
    callbacks.on_content_rect_changed = Some(on_content_rect_changed);
    callbacks.on_configuration_changed = Some(on_configuration_changed);
    callbacks.on_low_memory = Some(on_low_memory);

    let engine = NativeEngine::new();
    (*activity).instance = Box::into_raw(Box::new(engine)) as *mut c_void;

    log_info!("✅ ANativeActivity_onCreate configurado con éxito. Motor nativo listo.");
}
