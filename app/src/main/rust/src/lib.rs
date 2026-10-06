pub mod audio;
pub mod boss;
pub mod bullet;
pub mod enemy;
pub mod game;
pub mod level;
pub mod multiplayer;
pub mod native_activity;
pub mod player;
pub mod renderer;
pub mod touch;

pub use native_activity::ANativeActivity_onCreate;

use std::sync::Mutex;
use jni::errors::Error;
use jni::objects::{JByteArray, JClass, JIntArray, JShortArray};
use jni::sys::{jboolean, jfloat, jint};
use jni::{EnvUnowned, Outcome};

use crate::game::Game;

static GAME: Mutex<Option<Game>> = Mutex::new(None);
static RENDER_BUFFER: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static AUDIO_BUFFER: Mutex<Vec<i16>> = Mutex::new(Vec::new());

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeInit<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    width: jint,
    height: jint,
) {
    let w = width.max(1) as usize;
    let h = height.max(1) as usize;

    if let Ok(mut g) = GAME.lock() {
        *g = Some(Game::new(w, h));
    }
    if let Ok(mut rb) = RENDER_BUFFER.lock() {
        rb.resize(w * h, 0);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeResize<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    width: jint,
    height: jint,
) {
    let w = width.max(1) as usize;
    let h = height.max(1) as usize;

    if let Ok(mut g) = GAME.lock() {
        if let Some(game) = g.as_mut() {
            game.resize(w, h);
        }
    }
    if let Ok(mut rb) = RENDER_BUFFER.lock() {
        rb.resize(w * h, 0);
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeUpdateAndRender<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    pixel_buffer: JIntArray<'local>,
    dt: jfloat,
) {
    let outcome = unowned_env
        .with_env(|env| -> Result<(), Error> {
            let mut g_lock = GAME.lock().unwrap();
            let mut rb_lock = RENDER_BUFFER.lock().unwrap();

            if let Some(game) = g_lock.as_mut() {
                let required_size = game.width * game.height;
                if rb_lock.len() != required_size {
                    rb_lock.resize(required_size, 0);
                }

                // 1. Actualizar lógica de juego
                game.update(dt.max(0.0001).min(0.1));

                // 2. Renderizar cuadro a búfer RGBA
                game.render(&mut rb_lock);

                // 3. Copiar píxeles a array Java
                let i32_slice: &[jint] = unsafe {
                    std::slice::from_raw_parts(rb_lock.as_ptr() as *const jint, rb_lock.len())
                };

                pixel_buffer.set_region(env, 0, i32_slice)?;
            }
            Ok(())
        })
        .into_outcome();

    let _ = outcome;
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeTouchDown<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    pointer_id: jint,
    x: jfloat,
    y: jfloat,
) {
    if let Ok(mut g) = GAME.lock() {
        if let Some(game) = g.as_mut() {
            game.on_touch_down(pointer_id, x, y);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeTouchMove<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    pointer_id: jint,
    x: jfloat,
    y: jfloat,
) {
    if let Ok(mut g) = GAME.lock() {
        if let Some(game) = g.as_mut() {
            game.on_touch_move(pointer_id, x, y);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeTouchUp<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    pointer_id: jint,
    x: jfloat,
    y: jfloat,
) {
    if let Ok(mut g) = GAME.lock() {
        if let Some(game) = g.as_mut() {
            game.on_touch_up(pointer_id, x, y);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeGetAudioSamples<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    audio_buffer: JShortArray<'local>,
    count: jint,
) {
    let outcome = unowned_env
        .with_env(|env| -> Result<(), Error> {
            let n = count.max(0) as usize;
            let mut ab_lock = AUDIO_BUFFER.lock().unwrap();
            if ab_lock.len() != n {
                ab_lock.resize(n, 0);
            }

            if let Ok(mut g) = GAME.lock() {
                if let Some(game) = g.as_mut() {
                    game.audio_engine.render_samples(&mut ab_lock);
                }
            }

            env.set_short_array_region(&audio_buffer, 0, &ab_lock)?;
            Ok(())
        })
        .into_outcome();

    let _ = outcome;
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeBluetoothReceive<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    data: JByteArray<'local>,
) {
    let outcome = unowned_env
        .with_env(|env| -> Result<(), Error> {
            let bytes = env.convert_byte_array(&data)?;
            if let Ok(mut g) = GAME.lock() {
                if let Some(game) = g.as_mut() {
                    game.process_bluetooth_data(&bytes);
                }
            }
            Ok(())
        })
        .into_outcome();

    let _ = outcome;
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeBluetoothGetOutgoing<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jni::sys::jbyteArray {
    let outcome = unowned_env
        .with_env(|env| -> Result<jni::sys::jbyteArray, Error> {
            let outgoing = if let Ok(mut g) = GAME.lock() {
                if let Some(game) = g.as_mut() {
                    game.get_bluetooth_outgoing()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            let jarray = env.byte_array_from_slice(&outgoing)?;
            Ok(jarray.into_raw())
        })
        .into_outcome();

    match outcome {
        Outcome::Ok(raw) => raw,
        _ => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_diamon_iarebellion_GameBridge_nativeSetMultiplayerMode<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    is_host: jboolean,
    player_id: jint,
) {
    if let Ok(mut g) = GAME.lock() {
        if let Some(game) = g.as_mut() {
            game.set_multiplayer(is_host, player_id as u8);
        }
    }
}
