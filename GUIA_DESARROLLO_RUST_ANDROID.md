# Guía de Desarrollo, Compilación y Configuración: IA Rebellion (Rust + Android 2026)

> **IA Rebellion** es un shoot 'em up (shmup) de acción arcade espacial de **nueva generación (2026)** programado 100% en **Rust nativo**, inspirado en las mecánicas clásicas de **Final Mission** (versión japonesa de Natsume para Famicom/NES) y **Abadox**. Integra una **experiencia táctil pura sin botones virtuales en pantalla**, gráficos modernos con **luz ambiental dinámica**, simulación de **resplandor aditivo (bloom)**, sistema de partículas avanzado, texturas detalladas, soporte para **shaders/OpenGL ES 3** y audio moderno **OGG Vorbis** con música darksynth.

![Captura de Gameplay en Dispositivo Real (TECNO BF7)](docs_gameplay_screenshot.png)

---

## 1. Características Principales y Filosofía 2026

1. **100% Rust Nativo (Sin C / C++)**:
   - Toda la física del juego, máquina de estados, detección de colisiones, cálculo de iluminación dinámica puntual, sistema de partículas y protocolo de red Bluetooth están programados en **Rust** puro (`cdylib` mediante `jni = "0.22"`).
   - Java actúa únicamente como contenedor ligero de la plataforma Android (`SurfaceView`, `ModernAudioManager` con `SoundPool`/`MediaPlayer` y sockets Bluetooth RFCOMM).

2. **Requisitos Críticos de Android Moderno**:
   - **16 KB Page Alignment**: Totalmente compatible con **Android 15+**, compilando con banderas de enlazador `-Wl,-z,max-page-size=16384` y `-Wl,-z,common-page-size=16384`. Segmentos `LOAD` alineados a `0x4000`.
   - **PIE / PIC (Position Independent Executable / Code)**: Biblioteca compartida `DYN` de código independiente de posición (`relocation-model=pic`).

3. **Diferenciación de Vehículo según el Escenario**:
   - **Espacio Exterior / Migración Interplanetaria (Fases 1, 6, 7)**: El jugador pilota la **Nave Caza de Combate Estelar** con toberas de iones gemelas, alas delta, alerones y cañones gemelos frontales.
   - **Dentro de los Planetas (Fases 2, 3, 4, 5, 8)**: El jugador comanda al **Soldado Humano Cibernético con Jetpack** (mecánica de Final Mission Famicom Japón), equipado con exoesqueleto de titanio, botas gravitatorias, visor táctico y satélites orbitales rotatorios.

4. **Experiencia Táctil Pura (Algoritmo Directo de `Jugador.java`)**:
   - **Pantalla 100% limpia**: Ni palancas virtuales, ni D-Pads, ni botones virtuales.
   - **Fórmula de Arrastre 1:1 de `Jugador.java`**:
     ```rust
     // Al presionar la pantalla (on_touch_down):
     delta_x_tactil = touch_x - player.x;
     delta_y_tactil = touch_y - player.y;

     // Al deslizar el dedo (on_touch_move):
     player.x = (touch_x - delta_x_tactil).clamp(36.0, screen_width - 36.0);
     player.y = (touch_y - delta_y_tactil).clamp(36.0, screen_height - 36.0);
     ```
     El jugador responde inmediatamente al dedo con latencia cero sin ocultar el sprite.
   - **Auto-Disparo Continuo**: Disparo constante de plasma mientras el dedo mantenga el contacto táctil.
   - **Doble Toque Rápido (Double-Tap < 0.35 s)**: Detona la Bomba de Pulso Electromagnético (EMP) de pantalla completa.
   - **Multi-Touch (Segundo Dedo)**: Al apoyar un segundo dedo simultáneo, se activa el *Satellite Lock* y se direcciona el ángulo de tiro de los satélites orbitales.

4. **Motor Visual Moderno 2026 (Luz Ambiental, Resplandor y OpenGL ES 3)**:
   - **Luz Ambiental Dinámica Puntual (`draw_point_light`)**:
     - Las llamas del propulsor jetpack, los fogonazos de disparo (muzzle flash), los proyectiles de plasma, el reactor de los jefes y las explosiones emiten luz de color con atenuación cuadrática que baña la escena.
   - **Mezcla Aditiva (Bloom Simulation)**:
     - Partículas y láseres con canal alfa aditivo (`blend_additive`) para lograr un efecto incandescente futurista.
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

5. **Sistema de Audio Moderno OGG Vorbis 2026**:
   - Integración de `ModernAudioManager.java` con activos `.ogg` de alta fidelidad:
     - `laser_plasma.ogg`: Disparo pesado de plasma con transitorio nítido.
     - `explosion_heavy.ogg`: Explosión cinemática con sub-graves de 45 Hz y estruendo expansivo.
     - `emp_bomb.ogg`: Descarga eléctrica masiva con caída de tono.
     - `shield_hit.ogg`: Deflexión metálica resonante.
     - `boss_alarm.ogg`: Alarma futurista bitonal urgente.
     - `bgm_cyber_rebellion.ogg`: Pista musical de fondo en bucle estilo darksynth cyberpunk a 128 BPM.
   - Modo híbrido: Soporte simultáneo para síntesis procedural en tiempo real en Rust (`AudioTrack` PCM).

---

## 2. Estructura del Proyecto

```text
AI-Rebellion/
├── app/
│   ├── build.gradle                   # Configuración del paquete Android (ndk, packagingOptions)
│   └── src/
│       └── main/
│           ├── AndroidManifest.xml    # Permisos BLUETOOTH_CONNECT, BLUETOOTH_SCAN, etc.
│           ├── assets/
│           │   ├── audio/             # Archivos OGG Vorbis modernos 2026
│           │   │   ├── laser_plasma.ogg
│           │   │   ├── explosion_heavy.ogg
│           │   │   ├── emp_bomb.ogg
│           │   │   ├── shield_hit.ogg
│           │   │   ├── boss_alarm.ogg
│           │   │   └── bgm_cyber_rebellion.ogg
│           │   └── generate_audio.py  # Generador de audio OGG con ffmpeg / vorbis
│           ├── java/com/diamon/iarebellion/
│           │   ├── MainActivity.java       # Activity inmersiva en modo apaisado (Landscape)
│           │   ├── GameView.java           # SurfaceView optimizada con filtrado de texturas a 60 FPS
│           │   ├── ModernAudioManager.java # Reproductor SoundPool (SFX) y MediaPlayer (BGM) OGG
│           │   └── GameBridge.java         # Declaraciones nativas JNI con Rust
│           ├── jniLibs/
│           │   └── arm64-v8a/
│           │       └── libai_rebellion.so # Biblioteca Rust compilada (16 KB + PIE)
│           └── rust/
│               ├── Cargo.toml             # [lib] crate-type = ["cdylib"], jni = "0.22"
│               ├── .cargo/
│               │   └── config.toml        # Flags -Wl,-z,max-page-size=16384 y PIC
│               └── src/
│                   ├── lib.rs             # Puente JNI a Java
│                   ├── game.rs            # Bucle de juego y máquina de estados
│                   ├── touch.rs           # Motor táctil puro (arrastre relativo, doble tap, multi-touch)
│                   ├── player.rs          # Soldado cibernético, jetpack, armas, satélites
│                   ├── renderer.rs        # Luz ambiental dinámica, bloom aditivo, sprites y escenarios
│                   ├── boss.rs            # Jefes de los 8 niveles
│                   ├── enemy.rs           # 6 tipos de enemigos y cápsulas de items
│                   ├── bullet.rs          # Proyectiles, láseres, ondas expansivas
│                   ├── level.rs           # Gestor de fases
│                   ├── audio.rs           # Síntesis procedural PCM
│                   └── multiplayer.rs     # Protocolo Bluetooth RFCOMM para 4 jugadores
├── build_rust.sh                      # Script ejecutable de compilación y verificación
├── docs_gameplay_screenshot.png       # Captura de pantalla real del juego
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
   adb -s localhost:5555 shell am start -n com.diamon.iarebellion/.MainActivity
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
- **Tecnología**: 100% Rust (`cdylib`, ARM64, 16 KB Page Aligned, PIE), OGG Vorbis, Java Android SurfaceView/SoundPool.
