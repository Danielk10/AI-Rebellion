# 🤖 Directiva Operativa de IA Rebellion para Agentes Antigravity / Gemini

> **Destinatario:** Este documento es el manual de procedimiento estándar para cualquier Agente de IA (Antigravity CLI, Gemini CLI) que trabaje en el repositorio `AI-Rebellion`.
> **Filosofía del Proyecto:** IA Rebellion es un arcade shmup espacial con **arquitectura 100% nativa en Rust** (`android.app.NativeActivity` estilo Whisk3D), eliminando capas intermedias de Java/JNI en caliente para lograr latencia cero, 60 FPS estables y renderizado directo sobre `ANativeWindow`.

---

## 📌 Flujo de Trabajo Obligatorio para el Agente (Paso a Paso)

Todo agente que inicie una sesión en este repositorio DEBE seguir este orden de operaciones:

```
[1. Verificar ADB] ➔ [2. Compilar Rust (Termux)] ➔ [3. Empaquetar APK] ➔ [4. Probar en Móvil] ➔ [5. Commit y Push GitHub]
```

---

## 🔌 1. Verificación Inicial de Conexión ADB (REGLA DE ORO)

Antes de realizar cualquier cambio, compilación o prueba, **SIEMPRE verificar la conexión con el teléfono físico**:

```bash
# 1. Comprobar estado del dispositivo
adb-phone status

# 2. Si no está conectado, conectar al túnel reverso SSH (puerto 5555)
adb-phone connect

# 3. Confirmar que el dispositivo responda
adb -s localhost:5555 get-state
```

* **Dispositivo Físico:** TECNO BF7 / SPARK Go 2023 (Android 12, 4 núcleos ARM64 Cortex-A53).
* **Binario ADB:** `/home/danielpdiamon/.local/bin/adb` (Persistente en `$HOME`, no instalar con `apt`).

---

## 🛠️ 2. Entorno Android SDK y NDK

* El Android SDK y NDK **ya están instalados** y configurados en el sistema:
  - SDK: `/tmp/android-sdk/` y `~/.android-sdk/`
  - Build-tools: `37.0.0`
  - NDK: `30.0.14904198 rc1`
* **NUNCA** intentes reinstalar el SDK ni ejecutar comandos destructivos sobre estas rutas.

---

## 🦀 3. Compilación del Motor Nativo en Rust

El motor nativo debe compilarse para arquitectura **ARM64 (aarch64)** con soporte estricto de **páginas de 16 KB** y código de posición independiente (**PIE / PIC**).

### Método Oficial (Compilación Nativa en Termux vía ADB):
```bash
./compilar_en_termux.sh
```
Este script realiza automáticamente:
1. Sincronización del código fuente `app/src/main/rust` hacia Termux en el dispositivo.
2. Compilación nativa optimizada (`cargo build --release`) aprovechando los 4 núcleos físicos Cortex-A53.
3. Extracción del archivo `libai_rebellion.so` hacia `app/src/main/jniLibs/arm64-v8a/`.
4. Verificación de alineación de 16 KB (`Align 0x4000`) y tipo `DYN`.

---

## 📦 4. Compilación del APK con Gradle

El empaquetado del APK de Android se realiza con Gradle Wrapper aislando los archivos temporales en `/tmp` para proteger el espacio de disco en `$HOME`:

```bash
./gradlew assembleDebug
```
* **Ubicación de Salida:** `/tmp/ai_rebellion/outputs/apk/debug/app-debug.apk`

---

## 📱 5. Instalación, Ejecución y Validación en Dispositivo Real

```bash
# 1. Instalar APK en el dispositivo (si hay error de firma previa, desinstalar primero)
adb -s localhost:5555 install -r /tmp/ai_rebellion/outputs/apk/debug/app-debug.apk

# En caso de error "INSTALL_FAILED_UPDATE_INCOMPATIBLE":
adb -s localhost:5555 uninstall com.diamon.iarebellion
adb -s localhost:5555 install /tmp/ai_rebellion/outputs/apk/debug/app-debug.apk

# 2. Iniciar la actividad nativa
adb -s localhost:5555 shell monkey -p com.diamon.iarebellion -c android.intent.category.LAUNCHER 1

# 3. Limpiar y monitorizar Logcat en tiempo real
adb -s localhost:5555 logcat -c
adb -s localhost:5555 logcat -v time -s AIRebellionNative:V AndroidRuntime:E

# 4. Tomar captura de pantalla para validar rendering visual
adb-phone screenshot ~/captura_gameplay.png
```

---

## 🏛️ 6. Arquitectura del Proyecto (Estilo Whisk3D / NativeActivity)

* **Punto de Entrada:** `ANativeActivity_onCreate` exportado en `app/src/main/rust/src/native_activity.rs` y re-exportado en `lib.rs`.
* **Superficie de Render:** Escribe directamente sobre `ANativeWindow_Buffer` respetando el `stride` de hardware mediante `ANativeWindow_lock` y `ANativeWindow_unlockAndPost`.
* **Entrada Táctil:** Conexión directa a `AInputQueue` (`AMotionEvent`) con soporte para multi-touch, auto-disparo y bomba EMP sin botones en pantalla.
* **Cero Java en Caliente:** `java.srcDirs = []` en `app/build.gradle` para compilación ultra-rápida y cero pausas de Garbage Collector.

---

## 📖 7. Documentación de Referencia

Antes de introducir cambios estructurales importantes, consulta:
- `GUIA_DESARROLLO_RUST_ANDROID.md`: Especificaciones del motor, fórmulas táctiles y protocolo multijugador.
- `README.md`: Resumen público del proyecto y capturas.
- `GUIA_AGENTE_ANTIGRAVITY_ADB.md` (en `$HOME`): Manual operativo de ADB y túnel Termux.

---

## 🚀 8. Sincronización y Envío de Cambios a GitHub

Al finalizar cualquier iteración de trabajo validada en el dispositivo:
```bash
git status
git add .
git commit -m "feat/fix: descripción clara del cambio"
git push origin main
```
