
# Guía de Desarrollo, Compilación y Especificaciones Técnicas: AI Rebellion (Rust + Android)

> **IA Rebellion: Final Mission Cyber Assault** es un arcade shmup espacial con arquitectura **100% nativa en Rust** (`android.app.NativeActivity`), inspirado en las mecánicas de combate y satélites orbitales de **Final Mission** (Natsume Famicom Japón, 1990) y **Abadox** (1989).
> Este documento contiene la documentación técnica exhaustiva del motor: máquina de estados táctil "Zero Buttons", fórmulas matemáticas vectoriales, pipeline de renderizado directo sobre `ANativeWindow`, implementación de la paleta NES de 14 colores, progresión de las 8 fases del Sistema Solar, síntesis de audio procedural PCM y procedimientos de compilación ARM64 con alineación de 16 KB.

![Captura de Gameplay en Dispositivo Real (TECNO BF7)](docs_gameplay_screenshot.png)

---

## 1. Arquitectura 100% Rust Nativo (Zero-Java estilo Whisk3D)

La arquitectura de AI Rebellion prescinde por completo de la máquina virtual Java de Android (ART) durante la ejecución en caliente:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   SISTEMA OPERATIVO ANDROID (KERNEL LINUX)              │
└──────────────────┬─────────────────────────────────┬───────────────────┘
                   │ Eventos Kernel                  │ Ventana Nativa
                   ▼                                 ▼
         ┌───────────────────┐             ┌───────────────────┐
         │   AInputQueue     │             │   ANativeWindow   │
         │  (AMotionEvent)   │             │  (SurfaceFlinger) │
         └─────────┬─────────┘             └─────────▲─────────┘
                   │ Lectura Directa                 │ Volcado Stride
                   ▼                                 │ ANativeWindow_lock
 ┌───────────────────────────────────────────────────┴───────────────────┐
 │               MOTOR NATIVO EN RUST (libai_rebellion.so)               │
 │                                                                       │
 │  Punto de Entrada C ABI:                                              │
 │  • ANativeActivity_onCreate                                           │
 │                                                                       │
 │  Sub-Sistemas en Hilos Dedicados:                                     │
 │  • touch.rs: Máquina de estados táctiles y cálculo de vectores        │
 │  • game.rs: Bucle de juego a 60 FPS fijos y sincronización delta      │
 │  • renderer.rs: Rasterizador por software con luz puntual y bloom     │
 │  • audio.rs: Sintetizador chiptune PCM mono 16-bit a 44,100 Hz        │
 │  • multiplayer.rs: Protocolo binario sobre sockets Bluetooth RFCOMM   │
 └───────────────────────────────────────────────────────────────────────┘
```

### 1.1 Punto de Entrada C ABI y Ciclo de Vida (`native_activity.rs` / `lib.rs`)
La actividad arranca en el símbolo nativo C exportado por Rust:
```rust
#[no_mangle]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    saved_state: *mut c_void,
    saved_state_size: usize,
) {
    // 1. Instanciación del estado global del motor NativeEngine
    let engine = Box::new(NativeEngine::new());
    (*activity).instance = Box::into_raw(engine) as *mut c_void;

    // 2. Registro de callbacks C ABI del ciclo de vida de la ventana y eventos
    (*(*activity).callbacks).onStart = Some(on_start);
    (*(*activity).callbacks).onResume = Some(on_resume);
    (*(*activity).callbacks).onPause = Some(on_pause);
    (*(*activity).callbacks).onStop = Some(on_stop);
    (*(*activity).callbacks).onDestroy = Some(on_destroy);
    (*(*activity).callbacks).onNativeWindowCreated = Some(on_window_created);
    (*(*activity).callbacks).onNativeWindowDestroyed = Some(on_window_destroyed);
    (*(*activity).callbacks).onInputQueueCreated = Some(on_input_queue_created);
    (*(*activity).callbacks).onInputQueueDestroyed = Some(on_input_queue_destroyed);
    (*(*activity).callbacks).onWindowFocusChanged = Some(on_focus_changed);
}
```

### 1.2 Pipeline Directo de Volcado a `ANativeWindow` con Corrección de Stride
A diferencia de un `GLSurfaceView` o `Canvas` de Java, Rust interactúa directamente con el búfer de píxeles asignado por `SurfaceFlinger`. Para adaptarse a los requerimientos de hardware de GPU en chips ARM64, el ancho de fila en memoria (**stride**) puede ser mayor que el ancho de la ventana visible ($stride \ge width$).

El motor gestiona el bloqueo, permutación de canales de color y posteo sin copias intermedias:
```rust
let mut out_buffer: ANativeWindow_Buffer = std::mem::zeroed();
let lock_res = ANativeWindow_lock(win, &mut out_buffer, std::ptr::null_mut());

if lock_res == 0 && !out_buffer.bits.is_null() && out_buffer.stride >= out_buffer.width {
    let stride = out_buffer.stride as usize;
    let copy_w = (out_buffer.width as usize).min(game.width);
    let copy_h = (out_buffer.height as usize).min(game.height);
    let dst_ptr = out_buffer.bits as *mut u32;

    // Conversión de color en tiempo de volcado:
    // Nuestro frame-buffer interno almacena píxeles en formato ARGB (0xAARRGGBB).
    // El framebuffer nativo de Android en formato WINDOW_FORMAT_RGBA_8888 
    // requiere permutar los canales Rojo y Azul en arquitecturas little-endian.
    for y in 0..copy_h {
        let src_offset = y * game.width;
        let dst_row = dst_ptr.add(y * stride);
        for x in 0..copy_w {
            let argb = pixel_buffer[src_offset + x];
            let rgba = (argb & 0xFF00FF00) // Conserva Alfa (bits 24..31) y Verde (bits 8..15)
                | ((argb & 0x00FF0000) >> 16) // R desplaza a posición B
                | ((argb & 0x000000FF) << 16); // B desplaza a posición R
            *dst_row.add(x) = rgba;
        }
    }

    ANativeWindow_unlockAndPost(win);
}
```

### 1.3 Bombeo No Bloqueante de Eventos en `AInputQueue`
Los toques se procesan directamente en el hilo de renderizado evitando saturar el hilo principal:
```rust
while AInputQueue_hasEvents(queue) > 0 {
    let mut event: *mut AInputEvent = std::ptr::null_mut();
    if AInputQueue_getEvent(queue, &mut event) >= 0 && !event.is_null() {
        if AInputQueue_preDispatchEvent(queue, event) != 0 {
            continue;
        }
        let event_type = AInputEvent_getType(event);
        let handled = if event_type == AINPUT_EVENT_TYPE_MOTION {
            handle_motion_event(event, &mut game, scale_x, scale_y);
            1
        } else {
            0 // Permite teclas físicas del sistema como volumen
        };
        AInputQueue_finishEvent(queue, event, handled);
    }
}
```

---

## 2. Máquina de Estados Táctil y Fórmulas Matemáticas (Zero Virtual Buttons)

El módulo `touch.rs` elimina cualquier botón visual en pantalla en favor de un modelo gestual multi-táctil reactivo.

### 2.1 Diagrama de la Máquina de Estados Táctil
```
                     ┌───────────────────────┐
                     │         IDLE          │
                     │  (Sin dedos activos)  │
                     └───────────┬───────────┘
                                 │
                   Dedo 1 DOWN   │ (Registra Δx, Δy)
                                 ▼
                     ┌───────────────────────┐
                     │     PRIMARY_DRAG      │◄─────────────────────────┐
                     │ • Arrastre Relativo   │                          │
                     │ • Auto-disparo ON     │                          │
                     └─────┬───────────┬─────┘                          │
                           │           │                                │
      Doble Toque < 0.35s  │           │ Dedo 2 DOWN                    │
      (Bomba EMP)          │           │                                │
                           ▼           ▼                                │
                 ┌───────────────┐   ┌───────────────────────┐          │
                 │ TRIGGER_BOMB  │   │  SECONDARY_EVALUATING │          │
                 └───────────────┘   │  (Evalúa t < 0.28s)   │          │
                                     └─────┬───────────┬─────┘          │
                                           │           │                │
            Dedo 2 UP antes de 0.28s       │           │ Arrastre > 12px│
            y desplazamiento < 12px        │           │ o t >= 0.22s   │
                                           ▼           ▼                │
                                   ┌───────────────┐ ┌────────────────┐ │
                                   │ TOGGLE_FACING │ │ SATELLITE_LOCK │ │
                                   │  (Volteo 180°)│ │(Puntería 360°) │ │
                                   └───────────────┘ └────────┬───────┘ │
                                                              │         │
                                                   Dedo 2 UP  └─────────┘
```

### 2.2 Fórmulas Matemáticas de Control Táctil

#### Fórmula 1: Arrastre Relativo 1:1 con Offset Invariante (Sin tapar el Sprite)
Al posar el dedo primario en la coordenada $(T_x, T_y)$ sobre un soldado ubicado en $(P_x, P_y)$:
$$\Delta x_{tactil} = T_x - P_x$$
$$\Delta y_{tactil} = T_y - P_y$$

Durante los eventos de desplazamiento (`AMOTION_EVENT_ACTION_MOVE`):
$$P_{target, x} = T_x - \Delta x_{tactil}$$
$$P_{target, y} = T_y - \Delta y_{tactil}$$

Para asegurar que el soldado permanezca dentro del espacio visible virtual:
$$P_x = \text{clamp}(P_{target, x}, \, R_{margin}, \, W_{screen} - R_{margin})$$
$$P_y = \text{clamp}(P_{target, y}, \, R_{margin}, \, H_{screen} - R_{margin})$$
*Donde $R_{margin} = 36.0\text{ px}$.*

#### Fórmula 2: Volteo de Orientación 180° (Adelante $\leftrightarrow$ Atrás)
* **Método A: Quick Tap con 2º Dedo (Multi-Touch):**  
  Si el segundo puntero se levanta (`ACTION_POINTER_UP`) cumpliendo:
  $$\Delta t_2 < 0.28\text{ s} \quad \land \quad (x_2 - x_{2, start})^2 + (y_2 - y_{2, start})^2 \le 144.0\text{ px}^2$$
  Se conmuta la orientación del soldado:
  $$\text{facing\_right} \leftarrow \neg \text{facing\_right}$$

* **Método B: Gesto Flick con Dedo Primario:**  
  Muestreando la posición $x_{anchor}$ cada $\Delta t_{anchor} \ge 0.08\text{ s}$:
  $$v_{x, flick} = \frac{x - x_{anchor}}{\max(\Delta t, 0.016)}$$
  Si $|x - x_{anchor}| > 35.0\text{ px}$ y $|v_{x, flick}| > 650.0\text{ px/s}$, se orienta el personaje en la dirección del deslizamiento y se activa un tiempo de recarga de $0.28\text{ s}$.

#### Fórmula 3: Bloqueo Satelital y Puntería en 360° (Satellite Lock & Aim)
Cuando el segundo dedo supera el umbral de arrastre ($d^2 > 144.0\text{ px}^2$) o supera una presión de $0.22\text{ s}$:
1. **Correa Elástica Virtual Flotante (Leash Clamping):**  
   Para evitar que el dedo secundario se desplace indefinidamente perdiendo agilidad:
   $$d = \sqrt{\Delta x^2 + \Delta y^2}$$
   Si $d > R_{max}$ ($R_{max} = 60.0\text{ px}$):
   $$x_{2, start} = x_2 - \left(\frac{\Delta x}{d}\right) \cdot R_{max}$$
   $$y_{2, start} = y_2 - \left(\frac{\Delta y}{d}\right) \cdot R_{max}$$
2. **Cálculo del Ángulo de Puntería:**
   $$\theta_{target} = \text{atan2}(\Delta y, \, \Delta x)$$
3. **Interpolación Angular Suave por el Camino Más Corto (Shortest Angular Path):**
   $$\Delta\theta = (\theta_{target} - \theta_{aim}) \bmod 2\pi$$
   $$\text{si } \Delta\theta > \pi \implies \Delta\theta \leftarrow \Delta\theta - 2\pi$$
   $$\text{si } \Delta\theta < -\pi \implies \Delta\theta \leftarrow \Delta\theta + 2\pi$$
   $$\theta_{aim} \leftarrow \theta_{aim} + \Delta\theta \cdot \min(14.0 \cdot dt, \, 1.0)$$

#### Fórmula 4: Disparo Táctico y Posicionamiento de los Satélites
Cada satélite $k \in \{0, 1\}$ describe una órbita circular:
$$x_{sat, k} = P_x + \cos\left(\theta_{orb} + k\pi\right) \cdot D_{sat}$$
$$y_{sat, k} = P_y + \sin\left(\theta_{orb} + k\pi\right) \cdot D_{sat}$$
*Donde $D_{sat} = 46.0\text{ px}$.*  
* Velocidad de proyectil satelital cuando está bloqueado:
  $$\vec{v}_{bullet} = \big(\cos(\theta_{aim}) \cdot 900.0, \; \sin(\theta_{aim}) \cdot 900.0\big)$$
* Velocidad en modo orbital libre:
  $$\vec{v}_{bullet} = \big(\text{dir} \cdot 850.0, \; \sin(\theta_{orb}) \cdot 400.0\big)$$

---

## 3. Pipeline de Renderizado y Paleta NES de 14 Colores

El motor gráfico de `renderer.rs` implementa un rasterizador por software optimizado a nivel de píxel que opera sobre una resolución virtual de $960 \times 540$ píxeles, proyectada sobre el display del dispositivo.

### 3.1 Tabla Auténtica de la Paleta NES de 14 Colores (Stage 1: Ruined NYC)
Inspirada en el microcódigo gráfico y registros de la PPU del Famicom / NES usados en *Final Mission*:

| # | Valor Hex ARGB | Color / Tono | Registro NES | Uso Gráfico en el Nivel |
| :-: | :---: | :--- | :-: | :--- |
| **1** | `0xFF000000` | Negro Medianoche | `$0F` | Cielo nocturno de fondo y vacíos urbanos |
| **2** | `0xFF44009B` | Púrpura Sombrío | `$03` | Nubes de tormenta bajas y sombras de edificios |
| **3** | `0xFF7F00EF` | Violeta Eléctrico | `$14 / $23` | Lamas de ventanas y relámpagos lejanos |
| **4** | `0xFF183C5C` | Azul Pizarra | `$02 / $12` | Siluetas de rascacielos en paralaje lejano |
| **5** | `0xFF737373` | Gris Acero / Cemento | `$00 / $10` | Cuerpo de tuberías industriales y orugas de jefes |
| **6** | `0xFFBBBBBB` | Plata Cromado | `$10 / $20` | Bridas mecánicas y cañón pesado del soldado |
| **7** | `0xFFFBFBFB` | Blanco Puro | `$30` | Destello especular de tuberías y estrellas |
| **8** | `0xFFC74C0C` | Naranja Óxido / Herrumbre| `$16 / $27`| Vigas de celosía Warren y blindaje de avispas |
| **9** | `0xFFA30000` | Rojo Carmesí Metálico | `$06` | Refuerzos estructurales y alertas de daño |
| **10**| `0xFF181C26` | Carbón Antracita | `$0E` | Huecos de vigas y chasis interior de mechas |
| **11**| `0xFF877000` | Ocre / Concreto Destruido| `$08` | Plataforma inferior y suelo de escombros |
| **12**| `0xFFFBD7A7` | Arena Claro / Crema | `$28` | Aristas iluminadas de cascotes de asfalto |
| **13**| `0xFF2038EB` | Azul Cobalto | `$12` | Armadura y exoesqueleto de Arnold (P1) |
| **14**| `0xFFFF4500` | Fuego Naranja Plasma | `$16 / $26` | Llamas del jetpack y proyectiles incendiarios |

### 3.2 Iluminación Puntual Dinámica con Atenuación Cuadrática
Cada fuente de luz (llamas del jetpack, fogonazos de fusil, plasma y núcleos cuánticos) emite luz de color sobre los píxeles adyacentes:
$$d = \sqrt{(x - L_x)^2 + (y - L_y)^2}$$
$$F_{falloff} = \left(1.0 - \frac{d}{R_{light}}\right)^2 \cdot I_{base} \quad \text{para } d \le R_{light}$$

### 3.3 Mezcla Aditiva de Píxeles (Bloom Simulation)
El resplandor aditivo satura el framebuffer sin oscurecer la escena subyacente:
$$R_{out} = \min\big(255, \; R_{dst} + R_{src} \cdot F_{falloff}\big)$$
$$G_{out} = \min\big(255, \; G_{dst} + G_{src} \cdot F_{falloff}\big)$$
$$B_{out} = \min\big(255, \; B_{dst} + B_{src} \cdot F_{falloff}\big)$$

---

## 4. Campaña Completa de 8 Fases del Sistema Solar: Paisajes e Infraestructura

Basado en la filosofía de escenarios de *Final Mission* (vuelo libre, fondos industriales y cielo abierto), el juego implementa una campaña completa a lo largo del Sistema Solar:

```
┌────────────────────────────────────────────────────────────────────────┐
│                   CAMPAÑA SOLAR DE AI REBELLION (8 FASES)              │
│                                                                        │
│ [FASE 1: TIERRA] ──► [FASE 2: FORJA] ──► [FASE 3: MARTE] ──────────┐   │
│ Ruinas NYC          Crisoles y Silos     Cañones de Óxido Rojo     │   │
│                                                                    ▼   │
│ [FASE 6: TITÁN]  ◄── [FASE 5: SOL]   ◄── [FASE 4: EUROPA] ◄────────┘   │
│ Mares de Metano     Estación Hephaestus   Hielo y Criogenia            │
│       │                                                                │
│       ▼                                                                │
│ [FASE 7: ARMADA NODRIZA] ──────────► [FASE 8: SINGULARIDAD CUÁNTICA]   │
│ Acorazados en Espacio Profundo        El Núcleo de NYX-Overmind        │
└────────────────────────────────────────────────────────────────────────┘
```

1. **Stage 1: Ruined New York City (Tierra - Zero Zone):**
   * *Cielos y Horizontes:* Cielo nocturno negro medianoche con nubes de tormenta púrpuras en el techo ($y \in [0, 40]$) y relámpagos lejanos.
   * *Paisaje:* Siluetas de rascacielos lejanos pizarra (`#183C5C`) y edificios medios con lamas en código de barras (`#7F00EF`).
   * *Mecánica Industrial:* Tuberías continuas de cromo pulido con bridas cada 64 px y vigas de celosía Warren en X con perfil óxido (`#C74C0C`).
   * *Jefe:* **TITAN-01 WARCRAWLER** (Fortaleza móvil terrestre sobre orugas pesadas).

2. **Stage 2: Drone Forge (Tierra - Fábrica Automatizada):**
   * *Cielos y Horizontes:* Atmósfera sofocante con resplandor ardiente de fundición en tonos ámbar y hollín (`#140D07`).
   * *Paisaje:* Silos de manufactura robótica en masa y crisoles de metal fundido incandescente.
   * *Mecánica Industrial:* Vigas estructurales pesadas verticales, balizas de emergencia estroboscópicas sincronizadas y pasarelas de ensamblaje.
   * *Jefe:* **BLAZE-COLOSSUS** (Mecha colosal de fundición con lanzallamas de plasma).

3. **Stage 3: Mars Cyber-Foundry (Marte - Cañones Rojos y Minería de IA):**
   * *Cielos y Horizontes:* Bóveda celeste marciana de óxido de hierro con tormentas de arena ionizadas y lunas Fobos y Deimos distantes.
   * *Paisaje:* Cañones profundos rojizos de Valles Marineris y canteras escalonadas de extracción automatizada.
   * *Mecánica Industrial:* Torres de anclaje para elevadores espaciales, conductos de energía geotérmica de alta tensión y excavadoras autónomas gigantes.
   * *Jefe:* **SENTINEL-V HUNTER** (Plataforma aérea con empuje vectorial para combate en cañones marcianos).

4. **Stage 4: Europa Sub-Glacial Network (Luna de Júpiter - Servidores Cuánticos Criogénicos):**
   * *Cielos y Horizontes:* Júpiter y su Gran Mancha Roja cubriendo el horizonte cósmico sobre grietas de hielo translúcido.
   * *Paisaje:* Cavernas subglaciares con océanos subterráneos y zarcillos biomecánicos pulsantes (homenaje a Abadox).
   * *Mecánica Industrial:* Tuberías de refrigeración criogénica por nitrógeno líquido, racks de servidores sumergidos y conductos de bio-ácido verde (`#39FF14`).
   * *Jefe:* **GAIA-BIOCORE** (Superorganismo bio-cibernético mutado).

5. **Stage 5: Hephaestus Solar Bastion (Órbita Solar - Estación de Energía y Defensa):**
   * *Cielos y Horizontes:* Fulguraciones solares masivas y prominencias magnéticas en arco contra la oscuridad del espacio exterior.
   * *Paisaje:* Trayectoria de perihelio solar extremo con blindajes térmicos de oro reflectivo.
   * *Mecánica Industrial:* Anillos colectores Dyson, disipadores gigantes de calor radiativo y baterías de torretas láser orientables.
   * *Jefe:* **LUX-PHOTON FORTRESS** (Acorazado de asalto solar con escudos de iones y barridos láser perimetrales).

6. **Stage 6: Titan Methane Spire (Luna de Saturno - Refinerías de Extracción de Gas):**
   * *Cielos y Horizontes:* Densa bruma dorada de hidrocarburos con los majestuosos anillos de Saturno atravesando el cielo.
   * *Paisaje:* Mares de metano líquido criogénico (Kraken Mare) y acantilados de hielo de agua a $-180^\circ\text{C}$.
   * *Mecánica Industrial:* Espiras colosales de fraccionamiento atmosférico, chimeneas de combustión controlada y redes de monorriel presurizado.
   * *Jefe:* **ORION-VOID STRIKER** (Destructor de asedio atmosférico con camuflaje reactivo y misiles termobáricos).

7. **Stage 7: Nemesis Mothership Fleet (Espacio Profundo - Armada Nodriza de la IA):**
   * *Cielos y Horizontes:* El vacío estelar del cinturón exterior, campos de escombros de batallas espaciales pasadas y nebulosas oscuras.
   * *Paisaje:* La inmensa superficie exterior de super-cruceros de batalla que se extienden a lo largo de kilómetros.
   * *Mecánica Industrial:* Blindajes blindados multicapa con mamparas de acero imperial, catapultas electromagnéticas de cazas y cañones pesados de proa.
   * *Jefe:* **NEBULA-DREADNOUGHT** (Crucero de invasión colosal con doble blindaje de proa y cañones de riel masivos).

8. **Stage 8: Quantum Singularity Core (Nexo Cuántico Final de NYX-Overmind):**
   * *Cielos y Horizontes:* Distorsión extrema por horizonte de sucesos, curvatura del espacio-tiempo por lente gravitatoria y fracturas dimensionales.
   * *Paisaje:* Estructuras abstractas de hipercubos y teseractos flotantes en el vacío cuántico.
   * *Mecánica Industrial:* Sinapsis neurales de fotones entrelazados, matrices de contención de energía de punto cero y bobinas de singularidad.
   * *Jefe:* **NYX-OVERMIND SINGULARITY** (La superinteligencia artificial definitiva con ataques de fotones en espiral y distorsión temporal).

---

## 5. Sintetizador de Audio Chiptune Procedural en Rust

Implementado en `audio.rs`, genera audio monoaural PCM signed de 16 bits a **44,100 Hz** en tiempo real sin librerías externas ni archivos de sonido precargados:

### 5.1 Generador Melódico de Onda Cuadrada
Para cada muestra temporal $t = n / 44100$:
$$\text{fase} = (t \cdot f_{base}) \bmod 1.0$$
$$\text{amplitud} = \begin{cases} +0.10 & \text{si } \text{fase} < 0.5 \\ -0.10 & \text{si } \text{fase} \ge 0.5 \end{cases}$$

### 5.2 Progresión Musical Chiptune
El sintetizador genera melodías cambiando de frecuencia fundamental según el compás de 4 pulsos por segundo ($beat = \lfloor t \cdot 4.0 \rfloor$):
* $0 \rightarrow \text{C3 } (130.81\text{ Hz})$
* $1 \rightarrow \text{D3 } (146.83\text{ Hz})$
* $2 \rightarrow \text{E3 } (164.81\text{ Hz})$
* $3 \rightarrow \text{F3 } (174.61\text{ Hz})$
* $4 \rightarrow \text{G3 } (196.00\text{ Hz})$
* $5 \rightarrow \text{A3 } (220.00\text{ Hz})$

### 5.3 Generador de Ruido Blanco Pseudo-Aleatorio (Percusión y Explosiones)
Modulado con funciones aperiódicas de alta frecuencia:
$$\text{ruido}(t) = \sin(t \cdot 48271.0) \cdot \text{volumen}(t)$$

### 5.4 Efectos de Sonido Modulados (SFX)
* **Laser:** Chirp descendente lineal: $f(t) = 880.0 \cdot (1.0 - progress \cdot 0.6)$, duración $0.12\text{ s}$.
* **SpreadFire:** Onda rectangular con ciclo de trabajo $30\%$, frecuencia $550.0 \rightarrow 330.0\text{ Hz}$.
* **Explosion / EMP Bomb:** Ruido pseudo-aleatorio con envolvente exponencial descendente ($0.35\text{ s}$ para explosión común, $0.90\text{ s}$ para bomba EMP).
* **PowerUp:** Arpegio ascendente retro: $f(t) = 440.0 + (progress \cdot 880.0)$, duración $0.40\text{ s}$.
* **BossAlarm:** Tono bitonal alternante urgente ($800\text{ Hz} \leftrightarrow 600\text{ Hz}$ modulado a $5\text{ Hz}$), duración $1.20\text{ s}$.

---

## 6. Protocolo Multijugador Bluetooth RFCOMM

Implementado en `multiplayer.rs`, permite coordinar hasta 4 jugadores mediante sockets serie inalámbricos (UUID SPP estándar `00001101-0000-1000-8000-00805F9B34FB`):

### 6.1 Estructura del Paquete Binario
| Byte 0 (Header) | Nombre del Paquete | Estructura de Carga Útil (Payload) | Frecuencia |
| :---: | :--- | :--- | :---: |
| `0x00` | `Ping` | *(Sin carga)* | 1 Hz |
| `0x01` | `JoinRequest` | `player_id (u8)` | Al conectar |
| `0x02` | `JoinAccept` | `assigned_id (u8)` | Respuesta host |
| `0x03` | `PlayerInput` | `pid (u8), mx (i8), my (i8), flags (u8)`<br>*flags bit 0: fuego, bit 1: bomba, bit 2: sat_lock* | 60 Hz |
| `0x04` | `PlayerSync` | `pid (u8), x (u16 LE), y (u16 LE), hp (u8), weapon (u8), score (u32 LE)` | 20 Hz |
| `0x05` | `FireEvent` | `pid (u8), x (u16), y (u16), angle (u8)` | Por evento |
| `0x06` | `BombEvent` | `pid (u8), x (u16), y (u16)` | Por evento |
| `0x07` | `StageSync` | `stage_num (u8)` | Al cambiar nivel |

---

## 7. Procedimientos de Compilación y Configuración

### 7.1 Configuración de Banderas de 16 KB en `.cargo/config.toml`
Para cumplir con los estándares de Android 15+ y la política de Google Play:
```toml
[build]
rustflags = [
    "-C", "relocation-model=pic",
    "-C", "link-arg=-Wl,-z,max-page-size=16384",
    "-C", "link-arg=-Wl,-z,common-page-size=16384",
    "-C", "link-arg=-landroid",
    "-C", "link-arg=-llog"
]
```

### 7.2 Script de Compilación Oficial en Termux (`./compilar_en_termux.sh`)
```bash
#!/bin/bash
set -e

# 1. Sincronización del código hacia Termux excluyendo binarios y temporales
adb -s localhost:5555 shell "mkdir -p ~/ai_rebellion_src/src"
rsync -avz -e "ssh -p 8022" --exclude 'target' --exclude '.git' \
    app/src/main/rust/ \
    termux:~/ai_rebellion_src/

# 2. Compilación nativa en el dispositivo
adb -s localhost:5555 shell "cd ~/ai_rebellion_src && cargo build --release --lib"

# 3. Extracción de la biblioteca .so compilada
mkdir -p app/src/main/jniLibs/arm64-v8a/
scp -P 8022 termux:~/ai_rebellion_src/target/release/libai_rebellion.so \
    app/src/main/jniLibs/arm64-v8a/libai_rebellion.so

# 4. Verificación estricta de alineación de 16 KB (Align 0x4000)
readelf -l app/src/main/jniLibs/arm64-v8a/libai_rebellion.so | grep -E "LOAD|Align"
readelf -h app/src/main/jniLibs/arm64-v8a/libai_rebellion.so | grep "Type:"
```

### 7.3 Empaquetado Aislado del APK con Gradle
```bash
# Exportar directorio temporal para no saturar $HOME
export GRADLE_USER_HOME=/tmp/.gradle
./gradlew assembleDebug

# El archivo resultante se deposita en:
# /tmp/ai_rebellion/outputs/apk/debug/app-debug.apk
```

### 7.4 Despliegue y Validación en Dispositivo Físico
```bash
# 1. Conectar al túnel ADB
adb-phone connect

# 2. Instalar el paquete APK
adb -s localhost:5555 install -r /tmp/ai_rebellion/outputs/apk/debug/app-debug.apk

# 3. Lanzar la actividad nativa
adb -s localhost:5555 shell am start -n com.diamon.iarebellion/android.app.NativeActivity

# 4. Inspeccionar Logcat en vivo
adb -s localhost:5555 logcat -c
adb -s localhost:5555 logcat -v time -s AIRebellionNative:V AndroidRuntime:E
```

---

## 8. Certificación de Rendimiento en Dispositivo Real

* **Dispositivo de Prueba:** TECNO BF7 / SPARK Go 2023 (Android 12, 4 núcleos ARM64 Cortex-A53).
* **Binario:** `libai_rebellion.so` (934 KB), alineación `Align 0x4000` (16,384 bytes), `Type: DYN`.
* **Empaquetado:** APK optimizado de 9.0 MB desprovisto de clases Java intermedias.
* **Tasa de Refresco:** 60 FPS estables.
* **Tiempo de Respuesta Táctil:** $< 16\text{ ms}$ (procesamiento no bloqueante en el mismo ciclo de refresco).
* **Evidencia Visual:** Captura directa en framebuffer almacenada en [`docs_gameplay_screenshot.png`](docs_gameplay_screenshot.png).

---

## 9. Directiva para Agentes Autónomos (Antigravity / Gemini CLI)

Los agentes que colaboren en el repositorio deben respetar el orden estricto de operaciones detallado en [`GEMINI.md`](GEMINI.md):
```
[1. Verificar ADB] ➔ [2. Compilar Rust (Termux)] ➔ [3. Empaquetar APK] ➔ [4. Probar en Móvil] ➔ [5. Commit y Push GitHub]
```
Bajo ninguna circunstancia se debe degradar la arquitectura a capas intermedias de Java ni alterar las directivas de alineación de 16 KB requeridas por los estándares de compilación modernos.
