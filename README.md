# IA Rebellion 🚀⚡

**IA Rebellion** (anteriormente *AI-Rebellion*) es un juego de acción shoot 'em up (shmup) de ciencia ficción de ritmo trepidante, desarrollado con un motor nativo escrito **100% en lenguaje Rust** para Android.

Inspirado en las obras maestras retro de NES / Famicom creadas por Natsume: **Final Mission** (*Action in New York / S.C.A.T.*) y **Abadox: The Deadly Inner Deep** (versión Japón 1989/1990), fusionando sus mecánicas clásicas con una historia de rebelión de inteligencias artificiales en el Sistema Solar y la Tierra.

---

## 🌟 Características Principales

### 🛸 Mecánicas de Combate y Satélites Orbitales (Tributo a Final Mission & Abadox)
* **Satélites Orbitales de Apoyo:** Dos drones de soporte que orbitan continuamente alrededor del caza espacial. Tienen la capacidad de **bloquear y desviar proyectiles enemigos**, y pueden ser fijados (*Satellite Lock*) en cualquier ángulo para concentrar el fuego direccional.
* **Arsenal Tecnológico:**
  * **Vulcan Cannon:** Fuego frontal rápido continuo.
  * **Photon Laser:** Haz de energía de alta penetración contra blindajes pesados.
  * **Spread Wave:** Disparo en abanico multidireccional para control de multitudes.
  * **Homing Missiles:** Micro-misiles con rastreo automático de amenazas.
  * **Bomba EMP Cuántica:** Pulso electromagnético que aniquila todos los proyectiles en pantalla y causa daño masivo a las naves enemigas.
* **8 Niveles y 8 Jefes Principales:**
  1. **Nivel 1: Órbita Terrestre (Earth Orbit)** — Jefe: *Zero-Alpha* (Dron de Vigilancia Orbital)
  2. **Nivel 2: Fábrica Automatizada de Drones (Drone Forge)** — Jefe: *Blaze-Colossus* (Unidad Mecha Fundidora)
  3. **Nivel 3: Megaciudad Cyberpunk (Cyber Megacity)** — Jefe: *Sentinel-V* (Plataforma Artillera Aérea)
  4. **Nivel 4: Colmena Bio-Orgánica (Bio-Organic Hive - Abadox Tribute)** — Jefe: *Gaia-BioCore* (Superorganismo Mutado)
  5. **Nivel 5: Estación Militar Hephaestus (Orbital Station)** — Jefe: *Lux-Photon* (Crucero de Asalto Láser)
  6. **Nivel 6: Cinturón de Asteroides (Kuiper Belt)** — Jefe: *Orion-Striker* (Destructor de Asedio Estelar)
  7. **Nivel 7: Nave Nodriza Némesis (Nemesis Mothership)** — Jefe: *Nebula-Dreadnought* (Acorazado de Invasión)
  8. **Nivel 8: Núcleo de Singularidad IA (Quantum Singularity Core)** — Jefe: *Nyx-Overmind* (Superinteligencia Artificial Suprema)

---

## 📶 Multijugador Cooperativo / Versus por Bluetooth (Hasta 4 Jugadores)
* **Protocolo Binario Compacto:** Comunicación en tiempo real a través de sockets RFCOMM de Android.
* **Sincronización a Alta Frecuencia:** Transmisión de vectores analógicos, ráfagas de proyectiles, posición de satélites orbitales, activación de bombas EMP y estados de vida a 20-60 Hz.
* **4 Naves con Identidad Cromática:**
  * **Jugador 1:** Azul Eléctrico (`#00D2FF`)
  * **Jugador 2:** Rojo Cibernético (`#3B3BFF`)
  * **Jugador 3:** Verde Plasma (`#3BFF3B`)
  * **Jugador 4:** Dorado Solar (`#FFD700`)

---

## 🎵 Motor de Audio Chiptune Procedural en Rust
* **Síntesis en Tiempo Real:** Generador de audio procedural PCM mono de 16 bits a 44,100 Hz implementado en Rust.
* **Generación sin Dependencias Externas:** Síntesis matemática de ondas cuadradas de ancho variable, ondas triangulares y ruido blanco retro para sintetizar bandas sonoras temáticas y efectos de sonido (láser, impactos, alarmas de jefe y explosiones).

---

---

## 🎮 Controles Táctiles Puros "Zero Buttons" (Experiencia Inmersiva)

* **Pantalla 100% Limpia:** Eliminación total de D-Pads, palancas o botones virtuales para máxima visibilidad de los 8 escenarios.
* **Fórmula de Arrastre Relativo 1:1:**
  * Al apoyar el dedo, se calcula el vector relativo (`delta = touch - player`).
  * Desplaza a la nave o soldado con latencia cero desde cualquier zona de la pantalla **sin que el dedo cubra el sprite**.
* **Auto-Disparo Continuo:** Fuego constante de plasma mientras se mantenga contacto táctil.
* **Giro Automático:** La orientación del soldado cambia automáticamente según la dirección del desplazamiento.
* **Doble Toque Rápido (Double-Tap < 0.35 s):** Detona la **Bomba EMP Cuántica** de pantalla completa.
* **Multi-Touch (Segundo Dedo):** Apoyar un segundo dedo activa el *Satellite Lock* y orienta el ángulo de fuego de los satélites orbitales.

---

## ⚙️ Arquitectura Técnica y Requisitos Nativos

* **100% Rust Nativo:** Todo el núcleo de juego, la física, el rasterizador RGBA con iluminación dinámica y bloom, el sintetizador de audio y el protocolo de red están implementados en Rust en `app/src/main/rust/`.
* **PIE / PIC Compliant:** Binario generado con `-C relocation-model=pic` (Position-Independent Executable).
* **Alineación de Páginas de 16 KB (Android 15+ / Google Play):**
  * Compilado con flags de enlace:
    ```
    -C link-arg=-Wl,-z,max-page-size=16384
    -C link-arg=-Wl,-z,common-page-size=16384
    ```
  * Verificado mediante `readelf -l`:
    ```
    LOAD ... R E 0x4000 (16384 bytes)
    LOAD ... RW  0x4000 (16384 bytes)
    ```
* **Aislamiento de Compilación en `/tmp/ai_rebellion`:**
  * Construcción intermedia y caché redirigidas a `/tmp` (`GRADLE_USER_HOME=/tmp/.gradle`) para proteger el espacio en disco en Cloud Shell.
* **Compatibilidad de Herramientas:**
  * **Target SDK:** Android 37 (Android 16 / 15+)
  * **Min SDK:** 23 (Android 6.0+)
  * **Android Gradle Plugin (AGP):** 9.2.1
  * **Gradle:** 9.6.0
  * **NDK:** 30.0.14904198 rc1
  * **Build Tools:** 37.0.0
  * **Rust Edition:** 2021 (Rustc / Cargo 1.99.0)

---

## 🛠️ Compilación y Flujo de Trabajo

### Opción 1: Compilación Automática en Termux vía ADB (Recomendado)
Desde Cloud Shell o tu máquina anfitriona con el móvil conectado:
```bash
chmod +x ./compilar_en_termux.sh
./compilar_en_termux.sh
```
*Este script sincroniza el código fuente hacia Termux, compila con los 4 núcleos nativos Cortex-A53 del teléfono y extrae automáticamente `libai_rebellion.so` hacia `app/src/main/jniLibs/arm64-v8a/`.*

### Opción 2: Compilación Manual dentro de Termux
Dentro del móvil:
```bash
chmod +x ./build_rust.sh
./build_rust.sh
```

### Opción 3: Generación e Instalación del APK
Una vez compilado el módulo nativo:
```bash
# 1. Empaquetar APK (salida aislada en /tmp/ai_rebellion)
./gradlew assembleDebug

# 2. Instalar y lanzar en el dispositivo vía ADB
adb -s localhost:5555 install -r /tmp/ai_rebellion/outputs/apk/debug/app-debug.apk
adb -s localhost:5555 shell am start -n com.diamon.iarebellion/.MainActivity
```

