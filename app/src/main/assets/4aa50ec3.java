package com.diamon.utilidad;

/**
 * MEJORA: Clase helper para manejar la integración entre la cámara y el jugador
 * Coordina los límites del mundo, la cámara y el jugador de forma sincronizada
 */
public class GestorDeMundoJuego {

    private float mundoAncho;
    private float mundoAlto;

    // Límites de la pantalla
    private float pantalllaAncho;
    private float pantallaAlto;

    // Margen de seguridad para el jugador
    private static final int MARGEN_SEGURIDAD_JUGADOR = 32;

    public GestorDeMundoJuego(float mundoAncho, float mundoAlto, float pantallaAncho, float pantallaAlto) {
        this.mundoAncho = mundoAncho;
        this.mundoAlto = mundoAlto;
        this.pantalllaAncho = pantallaAncho;
        this.pantallaAlto = pantallaAlto;
    }

    /**
     * Obtiene el límite izquierdo del mundo donde puede estar el jugador
     */
    public float getLimiteIzquierdo() {
        return MARGEN_SEGURIDAD_JUGADOR;
    }

    /**
     * Obtiene el límite superior del mundo donde puede estar el jugador
     */
    public float getLimiteArriba() {
        return MARGEN_SEGURIDAD_JUGADOR;
    }

    /**
     * Obtiene el límite derecho del mundo donde puede estar el jugador
     * Considera el ancho del jugador
     */
    public float getLimiteDerecho(float anchoJugador) {
        return mundoAncho - MARGEN_SEGURIDAD_JUGADOR - anchoJugador;
    }

    /**
     * Obtiene el límite inferior del mundo donde puede estar el jugador
     * Considera el alto del jugador
     */
    public float getLimiteAbajo(float altoJugador) {
        return mundoAlto - MARGEN_SEGURIDAD_JUGADOR - altoJugador;
    }

    /**
     * Verifica si una posición está dentro de los límites válidos
     */
    public boolean esPosicionValida(float x, float y, float ancho, float alto) {
        return x >= getLimiteIzquierdo() &&
                x <= getLimiteDerecho(ancho) &&
                y >= getLimiteArriba() &&
                y <= getLimiteAbajo(alto);
    }

    /**
     * Restringe una posición a los límites válidos
     */
    public float[] restringirPosicion(float x, float y, float ancho, float alto) {
        float xRestringida = x;
        float yRestringida = y;

        // Limitar X
        if (x < getLimiteIzquierdo()) {
            xRestringida = getLimiteIzquierdo();
        } else if (x > getLimiteDerecho(ancho)) {
            xRestringida = getLimiteDerecho(ancho);
        }

        // Limitar Y
        if (y < getLimiteArriba()) {
            yRestringida = getLimiteArriba();
        } else if (y > getLimiteAbajo(alto)) {
            yRestringida = getLimiteAbajo(alto);
        }

        return new float[]{xRestringida, yRestringida};
    }

    /**
     * Obtiene la posición central óptima para la cámara centrada en el jugador
     * Asegura que la cámara no salga del mundo pero sigue al jugador
     */
    public float[] calcularPosicionCamara(
            float jugadorCentroX,
            float jugadorCentroY,
            float pantallaAncho,
            float pantallaAlto) {

        float camaraCentroX = jugadorCentroX;
        float camaraCentroY = jugadorCentroY;

        // Limitar la cámara para que no salga del mundo
        float mitadAnchoVista = pantallaAncho / 2;
        float mitadAltoVista = pantallaAlto / 2;

        // Limitar X
        if (camaraCentroX - mitadAnchoVista < 0) {
            camaraCentroX = mitadAnchoVista;
        } else if (camaraCentroX + mitadAnchoVista > mundoAncho) {
            camaraCentroX = mundoAncho - mitadAnchoVista;
        }

        // Limitar Y
        if (camaraCentroY - mitadAltoVista < 0) {
            camaraCentroY = mitadAltoVista;
        } else if (camaraCentroY + mitadAltoVista > mundoAlto) {
            camaraCentroY = mundoAlto - mitadAltoVista;
        }

        return new float[]{camaraCentroX, camaraCentroY};
    }

    /**
     * Verifica si la cámara se puede mover hacia una posición sin salir del mundo
     */
    public boolean puedeMoverseCamara(float x, float y, float pantallaAncho, float pantallaAlto) {
        float mitadAnchoVista = pantallaAncho / 2;
        float mitadAltoVista = pantallaAlto / 2;

        boolean dentroX = (x - mitadAnchoVista >= 0) && (x + mitadAnchoVista <= mundoAncho);
        boolean dentroY = (y - mitadAltoVista >= 0) && (y + mitadAltoVista <= mundoAlto);

        return dentroX && dentroY;
    }

    /**
     * Obtiene el ancho del mundo
     */
    public float getMundoAncho() {
        return mundoAncho;
    }

    /**
     * Obtiene el alto del mundo
     */
    public float getMundoAlto() {
        return mundoAlto;
    }

    /**
     * Actualiza el tamaño del mundo
     */
    public void setTamanoMundo(float mundoAncho, float mundoAlto) {
        this.mundoAncho = mundoAncho;
        this.mundoAlto = mundoAlto;
    }
}
