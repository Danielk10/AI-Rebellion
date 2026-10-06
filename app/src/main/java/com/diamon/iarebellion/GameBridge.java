package com.diamon.iarebellion;

/**
 * JNI Bridge para el motor nativo en Rust de IA R3bellion.
 * Implementación 100% Rust compilada con PIE y alineación de 16 KB (Android 15+).
 */
public final class GameBridge {

    static {
        System.loadLibrary("ai_rebellion");
    }

    private GameBridge() {}

    /**
     * Inicializa el estado del juego con la resolución virtual deseada.
     */
    public static native void nativeInit(int width, int height);

    /**
     * Notifica un cambio de dimensiones de pantalla.
     */
    public static native void nativeResize(int width, int height);

    /**
     * Actualiza el bucle de juego (física, IA, niveles) y dibuja el fotograma
     * en el búfer de píxeles ARGB_8888.
     */
    public static native void nativeUpdateAndRender(int[] pixelBuffer, float dt);

    /**
     * Registra evento táctil: dedo presionado.
     */
    public static native void nativeTouchDown(int pointerId, float x, float y);

    /**
     * Registra evento táctil: movimiento de puntero.
     */
    public static native void nativeTouchMove(int pointerId, float x, float y);

    /**
     * Registra evento táctil: dedo levantado.
     */
    public static native void nativeTouchUp(int pointerId, float x, float y);

    /**
     * Rellena el búfer de audio PCM mono de 16-bit (44100 Hz) sintetizado proceduralmente.
     */
    public static native void nativeGetAudioSamples(short[] audioBuffer, int count);

    /**
     * Inyecta paquetes binarios recibidos desde Bluetooth RFCOMM al motor de red.
     */
    public static native void nativeBluetoothReceive(byte[] data);

    /**
     * Extrae los paquetes generados por el motor de red para enviarlos por Bluetooth.
     */
    public static native byte[] nativeBluetoothGetOutgoing();

    /**
     * Configura el modo multijugador (anfitrión o cliente, ID de jugador 0 a 3).
     */
    public static native void nativeSetMultiplayerMode(boolean isHost, int playerId);
}
