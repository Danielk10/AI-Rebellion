# Guía de Desarrollo, Compilación y Configuración: IA Rebellion (Rust + Android 2026)

> **IA Rebellion** es un shoot 'em up (shmup) de acción arcade espacial de **nueva generación (2026)** programado 100% en **Rust nativo** (`android.app.NativeActivity`), inspirado en las mecánicas clásicas de **Final Mission** (versión japonesa de Natsume para Famicom/NES) y **Abadox**. Integra una **experiencia táctil pura sin botones virtuales en pantalla** con lectura directa desde `AInputQueue`, gráficos con **luz ambiental dinámica**, simulación de **resplandor aditivo (bloom)**, volcado directo a `ANativeWindow` respetando el stride de hardware a 60 FPS estables y síntesis de audio procedural chiptune PCM en Rust sin intermediación de Java.

![Captura de Gameplay en Dispositivo Real (TECNO BF7)](docs_gameplay_screenshot.png)

---

## 1. Características Principales y Filosofía 2026

1. **100% Rust Nativo Puro (Android NativeActivity - Arquitectura Zero-Java estilo Whisk3D)**:
   - Toda la física del juego, máquina de estados, detección de colisiones, cálculo de iluminación dinámica puntual, sistema de partículas y protocolo de red Bluetooth están programados en **Rust** puro (`cdylib`).
   - **Cero sobrecarga de Java por cuadro**: Arranca directamente desde el sistema operativo mediante `android.app.NativeActivity` con el punto de entrada `ANativeActivity_onCreate` exportado en Rust. El renderizado escribe directo al framebuffer nativo (`ANativeWindow`) y los gestos táctiles se leen directamente de `AInputQueue` (`AMotionEvent`), eliminando pausas de GC y reduciendo la latencia de entrada a niveles de microsegundos.


2. **Requisitos Críticos de Android Moderno**:
   - **16 KB Page Alignment**: Totalmente compatible con **Android 15+**, compilando con banderas de enlazador `-Wl,-z,max-page-size=16384` y `-Wl,-z,common-page-size=16384`. Segmentos `LOAD` alineados a `0x4000`.
   - **PIE / PIC (Position Independent Executable / Code)**: Biblioteca compartida `DYN` de código independiente de posición (`relocation-model=pic`).

3. **Diferenciación de Vehículo según el Escenario**:
   - **Espacio Exterior / Migración Interplanetaria (Fases 1, 6, 7)**: El jugador pilota la **Nave Caza de Combate Estelar** con toberas de iones gemelas, alas delta, alerones y cañones gemelos frontales.
   - **Dentro de los Planetas (Fases 2, 3, 4, 5, 8)**: El jugador comanda al **Soldado Humano Cibernético con Jetpack** (mecánica de Final Mission Famicom Japón), equipado con exoesqueleto de titanio, botas gravitatorias, visor táctico y satélites orbitales rotatorios.

4. **Experiencia Táctil Pura Directa desde `AInputQueue` (`touch.rs`)**:
   - **Pantalla 100% limpia**: Ni palancas virtuales, ni D-Pads, ni botones virtuales.
   - **Fórmula de Arrastre Relativo 1:1**:
     ```rust
     // Al presionar la pantalla (AMOTION_EVENT_ACTION_DOWN):
     delta_x_tactil = touch_x - player.x;
     delta_y_tactil = touch_y - player.y;

     // Al deslizar el dedo (AMOTION_EVENT_ACTION_MOVE):
     player.x = (touch_x - delta_x_tactil).clamp(36.0, screen_width - 36.0);
     player.y = (touch_y - delta_y_tactil).clamp(36.0, screen_height - 36.0);
     ```
     El jugador responde inmediatamente al dedo con latencia cero sin ocultar el sprite.
   - **Auto-Disparo Continuo**: Disparo constante de plasma mientras el dedo mantenga el contacto táctil.
   - **Doble Toque Rápido (Double-Tap < 0.35 s)**: Detona la Bomba de Pulso Electromagnético (EMP) de pantalla completa.
   - **Multi-Touch (Segundo Dedo)**: Al apoyar un segundo dedo simultáneo, se activa el *Satellite Lock* y se direcciona el ángulo de tiro de los satélites orbitales.

5. **Motor Visual Moderno 2026 (Luz Ambiental, Resplandor y Renderizado Directo a `ANativeWindow`)**:
   - **Luz Ambiental Dinámica Puntual (`draw_point_light`)**:
     - Las llamas del propulsor jetpack, los fogonazos de disparo (muzzle flash), los proyectiles de plasma, el reactor de los jefes y las explosiones emiten luz de color con atenuación cuadrática que baña la escena.
   - **Mezcla Aditiva (Bloom Simulation)**:
     - Partículas y láseres con canal alfa aditivo (`blend_additive`) para lograr un efecto incandescente futurista.
   - **Renderizado Directo sobre el Hardware (`ANativeWindow`)**:
     - `NativeRenderThread` vuelca el búfer de píxeles directamente a `ANativeWindow_Buffer` con `ANativeWindow_lock` y `ANativeWindow_unlockAndPost` respetando estrictamente el `stride` del controlador gráfico de SurfaceFlinger.
   - **Texturas y Sprites Detallados**:
     - *Soldados Humanos*: Armaduras tácticas con exoesqueleto, casco con visor reflectivo HUD, botas gravitacionales, rifle pesado y llamas de plasma pulsantes.
     - *6 Arquetipos de Enemigos*: Drones patrulleros angulares de titanio, avispas suicidas con bandas de peligro amarillo/negro, torretas pesadas hexagonales, cangrejos cibernéticos blindados, sanguijuelas bio-orgánicas con ojos pulsantes (homenaje Abadox) y cazas furtivos ala-delta.
     - *8 Jefes Colosales de Fase*: Cascos blindados multicapa, cañones dobles de proa, núcleo cuántico pulsante y barras de salud sci-fi con marco de alerta.
   - **8 Escenarios Ricos con Paralaje Multi-Capa**:
     1. *Órbita Terrestre*: Curvatura planetaria en gradiente y constelaciones profundas.
     2. *Fábrica de Drones*: Silos industriales y crisoles de metal fundido incandescente.
     3. *Ciudad Desolada*: Rascacielos cyberpunk nocturnos con ventanas de neón cian/magenta.
     4. *Colmena Bio-Orgánica*: Paredes viscerales con zarcillos y costillas ondulantes.
     5. *Laboratorio Secreto*: Rejillas criogénicas y tanques de estasis tecnológica.
     6. *Cinturón de Asteroides*: Campos densos de meteoritos y escombros rocosos.
     7. *Nave Nodriza Némesis*: Mamparas imperiales con iluminación carmesí.
     8. *Vórtice Cuántico*: Distorsión gravitatoria de horizonte de sucesos.

6. **Motor de Audio Procedural Chiptune en Rust (16-bit PCM, 44,100 Hz)**:
   - Todo el audio se sintetiza matemáticamente en Rust (`audio.rs`) sin dependencias de Java:
     - **BGM Procedural:** Generador melódico retro basado en ondas cuadradas y percusión de ruido sincronizado por compases según el nivel.
     - **Efectos de Sonido (SFX):**
       - `Laser`: Disparo de plasma con modulación lineal descendente.
       - `SpreadFire`: Ráfaga multidireccional con ciclo de trabajo rectangular.
       - `Explosion` / `BombExplosion`: Generador pseudo-aleatorio de ruido blanco para detonaciones e impactos.
       - `PowerUp`: Arpegio ascendente de frecuencia retro.
       - `BossAlarm`: Tono bitonal urgente de aviso de enfrentamiento con jefe.
       - `PlayerHit`: Transitorio de impacto sordo y distorsión.
   - **Cero latencia de GC**: Generación directa sin capas intermedias `SoundPool` o `MediaPlayer`.

---

## 2. Estructura del Proyecto

```text
AI-Rebellion/
├── app/
│   ├── build.gradle                   # Configuración del APK (ndk, packaging, java.srcDirs = [])
│   └── src/
│       └── main/
│           ├── AndroidManifest.xml    # NativeActivity (android.app.NativeActivity), permisos Bluetooth
│           ├── assets/                # Activos y recursos del juego
│           ├── jniLibs/
│           │   └── arm64-v8a/
│           │       └── libai_rebellion.so # Biblioteca Rust nativa (16 KB + PIE + C ABI NativeActivity)
│           └── rust/
│               ├── Cargo.toml             # [lib] crate-type = ["cdylib"]
│               ├── build.rs               # Enlace de stubs para entornos de prueba
│               ├── .cargo/
│               │   └── config.toml        # Flags -Wl,-z,max-page-size=16384, PIC, -landroid, -llog
│               └── src/
│                   ├── lib.rs             # Exportación ANativeActivity_onCreate y puente nativo
│                   ├── native_activity.rs # Motor 100% nativo C ABI: ciclo de vida, ANativeWindow, AInputQueue
│                   ├── game.rs            # Bucle de juego y máquina de estados
│                   ├── touch.rs           # Motor táctil puro (arrastre relativo 1:1, doble tap, multi-touch)
│                   ├── player.rs          # Soldado cibernético, jetpack, armas, satélites orbitales
│                   ├── renderer.rs        # Luz ambiental dinámica, bloom aditivo, sprites y escenarios
│                   ├── boss.rs            # Jefes de los 8 niveles
│                   ├── enemy.rs           # 6 tipos de enemigos y cápsulas de items
│                   ├── bullet.rs          # Proyectiles, láseres, ondas expansivas
│                   ├── level.rs           # Gestor de fases
│                   ├── audio.rs           # Síntesis procedural PCM chiptune en Rust
│                   └── multiplayer.rs     # Protocolo Bluetooth RFCOMM para 4 jugadores
├── compilar_en_termux.sh              # Script de sincronización y compilación nativa en Termux vía ADB
├── build_rust.sh                      # Script ejecutable de compilación y verificación local en Termux
├── GEMINI.md                          # Directiva operativa estándar para Agentes de IA
├── README.md                          # Visión general del proyecto y capturas
├── docs_gameplay_screenshot.png       # Captura de pantalla real del juego en dispositivo TECNO BF7
└── GUIA_DESARROLLO_RUST_ANDROID.md    # Este documento
```

---

## 3. Guía de Compilación y Flujo de Trabajo

### Opción A: Compilación Nativa en el Dispositivo (Termux)

1. **Instalar paquetes en Termux**:
   ```bash
   pkg update && pkg upgrade -y
   pkg install rust clang binutils git ffmpeg vorbis-tools -y
   ```

2. **Verificar configuración de 16 KB y PIE en `.cargo/config.toml`**:
   ```toml
   [build]
   rustflags = [
       "-C", "relocation-model=pic",
       "-C", "link-arg=-Wl,-z,max-page-size=16384",
       "-C", "link-arg=-Wl,-z,common-page-size=16384"
   ]
   ```

3. **Ejecutar el script de construcción**:
   ```bash
   chmod +x ./build_rust.sh
   ./build_rust.sh
   ```

4. **Verificar la alineación de 16 KB y PIE**:
   ```bash
   readelf -l app/src/main/jniLibs/arm64-v8a/libai_rebellion.so | grep -E "LOAD|Align"
   readelf -h app/src/main/jniLibs/arm64-v8a/libai_rebellion.so | grep "Type:"
   ```
   *Debe confirmar `Align 0x4000` (16,384 bytes) y `Type: DYN` (Position-Independent Executable).*

---

### Opción B: Flujo de Trabajo Automatizado desde Cloud Shell / PC con ADB

1. **Conectar el dispositivo**:
   ```bash
   adb-phone connect
   # O manualmente:
   adb connect localhost:5555
   ```

2. **Compilar en Termux y sincronizar el binario automáticamente**:
   ```bash
   chmod +x ./compilar_en_termux.sh
   ./compilar_en_termux.sh
   ```
   *(El script sincroniza el código fuente hacia Termux excluyendo temporales, compila con los 4 núcleos Cortex-A53 y extrae `libai_rebellion.so` hacia `app/src/main/jniLibs/arm64-v8a/`).*

3. **Compilar el APK con Gradle (salida aislada en `/tmp/ai_rebellion`)**:
   ```bash
   ./gradlew assembleDebug
   ```
   *Toda la compilación intermedia y caché se almacena en `/tmp` (`GRADLE_USER_HOME=/tmp/.gradle`), protegiendo el almacenamiento de `$HOME`.*

4. **Instalar y Ejecutar en el Teléfono**:
   ```bash
   adb -s localhost:5555 install -r /tmp/ai_rebellion/outputs/apk/debug/app-debug.apk
   adb -s localhost:5555 shell monkey -p com.diamon.iarebellion -c android.intent.category.LAUNCHER 1
   # O directamente:
   adb -s localhost:5555 shell am start -n com.diamon.iarebellion/android.app.NativeActivity
   ```

5. **Capturar Pantalla del Juego en Tiempo Real**:
   ```bash
   adb-phone screenshot
   # O manualmente:
   adb -s localhost:5555 exec-out screencap -p > captura.png
   ```


---

## 4. Algoritmo Táctil Puro (Zero Buttons)

El módulo `touch.rs` elimina cualquier control en pantalla en favor de una experiencia gestual pura:

```rust
pub fn on_touch_move(&mut self, id: i32, x: f32, y: f32) {
    let idx = (id.abs() as usize) % self.pointers.len();
    if self.pointers[idx].id == id && self.pointers[idx].active {
        let dx = x - self.pointers[idx].last_x;
        let dy = y - self.pointers[idx].last_y;

        self.pointers[idx].last_x = x;
        self.pointers[idx].last_y = y;

        // Puntero primario: arrastre del soldado humano y disparo continuo
        if self.primary_id == id {
            self.delta_x += dx;
            self.delta_y += dy;
            self.is_firing = true;
        } else if self.secondary_id == id {
            // Puntero secundario (segundo dedo): fijación y rotación de satélites
            self.update_satellite_aim();
        }
    }
}
```

- **Posicionamiento relativo**: `player.x = (player.x + dx).clamp(36.0, screen_w - 36.0)`.
- **Giro automático**: Si `dx > 0.8`, el soldado apunta a la derecha; si `dx < -0.8`, apunta a la izquierda.
- **Doble Toque (< 0.35 s)**: Detona la bomba EMP sin requerir botones.

---

## 5. Protocolo Multijugador Bluetooth RFCOMM

El protocolo binario `multiplayer.rs` opera sobre UUID SPP estándar:

| Byte 0 (Tipo) | Nombre | Carga Útil | Función |
| :---: | :--- | :--- | :--- |
| `0x00` | `Ping` | Ninguno | Latencia y Keepalive |
| `0x01` | `JoinRequest` | `player_id (u8)` | Petición de conexión |
| `0x02` | `JoinAccept` | `assigned_id (u8)` | Asignación de ID (P2, P3, P4) |
| `0x03` | `PlayerInput` | `pid (u8), mx (i8), my (i8), flags (u8)` | Movimiento relativo y disparos a 60 Hz |
| `0x04` | `PlayerSync` | `pid (u8), x (u16), y (u16), hp (u8), weapon (u8), score (u32)` | Sincronización del Host a 20 Hz |
| `0x05` | `StageSync` | `stage_num (u8)` | Cambio de nivel sincronizado |

---

## 6. Créditos y Licencia

- **Desarrollador**: Daniel Diamond ([@Danielk10](https://github.com/Danielk10))
- **Inspiración**: *Final Mission* (Natsume Japón, 1990) & *Abadox* (Natsume, 1989).
- **Tecnología**: 100% Rust (`cdylib`, ARM64, 16 KB Page Aligned, PIE, C ABI `android.app.NativeActivity`, `ANativeWindow`, `AInputQueue`), audio procedural PCM, cero intermediación de Java en caliente (`java.srcDirs = []`).

---

## 7. Validación y Pruebas en Dispositivo Real (TECNO BF7)

* **Dispositivo Físico:** TECNO BF7 / SPARK Go 2023 (Android 12, ARM64 Cortex-A53).
* **Conexión:** Túnel inverso SSH ADB en `localhost:5555`.
* **Compilación Nativa:** Módulo Rust compilado directamente en Termux con `cargo build --release` (tiempo: 2m 49s). Binario resultante `libai_rebellion.so` de 934 KB con alineación estricta de 16 KB (`Align 0x4000`) y tipo `DYN`.
* **Empaquetado Gradle:** APK generado desatendidamente en `/tmp/ai_rebellion/outputs/apk/debug/app-debug.apk` (9.0 MB) aislando el almacenamiento temporal y protegiendo la cuota de disco de `$HOME`.
* **Instalación y Ejecución:**
  * Instalado exitosamente vía `adb install -r`.
  * Lanzado en modo apaisado inmersivo (`1612x720`) directamente vía `android.app.NativeActivity`.
  * Validación de jugabilidad fluida a 60 FPS estables: arquitectura 100% nativa sin pausas de Garbage Collector, motor táctil "Zero Buttons" procesado directamente por `AInputQueue`, renderizado al framebuffer nativo mediante `ANativeWindow_lock` y `ANativeWindow_unlockAndPost` respetando el stride de hardware, auto-disparo continuo de proyectiles de plasma, satélites orbitales de apoyo interceptando amenazas y simulación de resplandor aditivo (bloom) sobre el escenario de la Órbita Terrestre.
  * Captura de pantalla real extraída y documentada: [`docs_gameplay_screenshot.png`](docs_gameplay_screenshot.png).

---

## 8. Directivas Operativas para Agentes de IA

Para agentes de IA (Antigravity CLI, Gemini CLI) y colaboradores que operen en este repositorio:
* Consulta obligatoria: [`GEMINI.md`](GEMINI.md) — Manual de procedimiento operativo estándar paso a paso (`[1. Verificar ADB] ➔ [2. Compilar Rust (Termux)] ➔ [3. Empaquetar APK] ➔ [4. Probar en Móvil] ➔ [5. Commit y Push GitHub]`), normas de seguridad de almacenamiento y verificación de hardware.

