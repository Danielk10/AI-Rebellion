package com.diamon.graficos;

import android.graphics.Canvas;
import android.graphics.Matrix;
import android.graphics.RectF;

public class Camara2D {

    private float x; // posición X de la cámara en el mundo
    private float y; // posición Y de la cámara en el mundo
    private float ancho; // ancho de la ventana de visualización
    private float alto; // alto de la ventana de visualización
    private float zoom; // nivel de zoom

    private Matrix matrizTransformacion; // matriz de transformación
    private RectF areaDeVisualizacion; // área visible de la cámara en coordenadas del mundo

    // Límites del mundo (opcional, para restringir la cámara)
    private float mundoAncho;
    private float mundoAlto;
    private boolean usarLimitesMundo;

    public Camara2D(float ancho, float alto) {
        this.ancho = ancho;
        this.alto = alto;
        this.zoom = 1.0f;
        this.x = 0;
        this.y = 0;
        this.matrizTransformacion = new Matrix();
        this.areaDeVisualizacion = new RectF(0, 0, ancho, alto);
        this.usarLimitesMundo = false;
        actualizarTransformacion();
    }

    /**
     * Configura los límites del mundo para la cámara
     */
    public void setLimitesMundo(float ancho, float alto) {
        this.mundoAncho = ancho;
        this.mundoAlto = alto;
        this.usarLimitesMundo = true;
        aplicarLimites();
        actualizarTransformacion();
    }

    /**
     * Deshabilita los límites del mundo
     */
    public void deshabilitarLimitesMundo() {
        this.usarLimitesMundo = false;
    }

    /**
     * Aplica límites para que la cámara no salga del mundo
     */
    private void aplicarLimites() {
        if (!usarLimitesMundo) {
            return;
        }

        float mitadAnchoVista = (ancho / 2) / zoom;
        float mitadAltoVista = (alto / 2) / zoom;

        // Limitar X
        if (x - mitadAnchoVista < 0) {
            x = mitadAnchoVista;
        } else if (x + mitadAnchoVista > mundoAncho) {
            x = mundoAncho - mitadAnchoVista;
        }

        // Limitar Y
        if (y - mitadAltoVista < 0) {
            y = mitadAltoVista;
        } else if (y + mitadAltoVista > mundoAlto) {
            y = mundoAlto - mitadAltoVista;
        }
    }

    /**
     * Actualiza la matriz de transformación y el área de visualización
     */
    private void actualizarTransformacion() {
        matrizTransformacion.reset();

        // Primero aplicamos zoom
        matrizTransformacion.postScale(zoom, zoom);

        // Luego trasladamos para centrar la cámara
        // La fórmula correcta para scroll: mueve el mundo en dirección opuesta a la cámara
        matrizTransformacion.postTranslate(-x * zoom + ancho / 2, -y * zoom + alto / 2);

        // Actualizar área de visualización en coordenadas del mundo
        float mitadAnchoVista = (ancho / 2) / zoom;
        float mitadAltoVista = (alto / 2) / zoom;

        areaDeVisualizacion.set(
            x - mitadAnchoVista,
            y - mitadAltoVista,
            x + mitadAnchoVista,
            y + mitadAltoVista
        );
    }

    /**
     * Mueve la cámara a una posición específica
     */
    public void moverA(float x, float y) {
        this.x = x;
        this.y = y;
        aplicarLimites();
        actualizarTransformacion();
    }

    /**
     * Mueve la cámara de forma suave hacia un objetivo (para seguimiento)
     * @param objetivoX posición X del objetivo
     * @param objetivoY posición Y del objetivo
     * @param velocidad velocidad de seguimiento (0-1, donde 1 es instantáneo)
     */
    public void seguirSuave(float objetivoX, float objetivoY, float velocidad) {
        velocidad = Math.max(0, Math.min(1, velocidad)); // Clamp entre 0 y 1
        this.x += (objetivoX - this.x) * velocidad;
        this.y += (objetivoY - this.y) * velocidad;
        aplicarLimites();
        actualizarTransformacion();
    }

    /**
     * Cambia el zoom de la cámara
     */
    public void setZoom(float zoom) {
        // Prevenir zoom inválido
        this.zoom = Math.max(0.1f, Math.min(10.0f, zoom));
        aplicarLimites();
        actualizarTransformacion();
    }

    /**
     * Convierte las coordenadas de pantalla a las del mundo
     */
    public float[] convertirPantallaAMundo(float pantallaX, float pantallaY) {
        float[] punto = new float[] {pantallaX, pantallaY};
        Matrix matrizInvertida = new Matrix();
        if (matrizTransformacion.invert(matrizInvertida)) {
            matrizInvertida.mapPoints(punto);
        }
        return punto;
    }

    /**
     * Convierte las coordenadas del mundo a las de pantalla
     */
    public float[] convertirMundoAPantalla(float mundoX, float mundoY) {
        float[] punto = new float[] {mundoX, mundoY};
        matrizTransformacion.mapPoints(punto);
        return punto;
    }

    /**
     * Obtiene el área visible de la cámara en coordenadas del mundo
     */
    public RectF getAreaDeVisualizacion() {
        return areaDeVisualizacion;
    }

    /**
     * Verifica si un rectángulo está visible en la cámara (para culling)
     */
    public boolean esVisible(float x, float y, float ancho, float alto) {
        return areaDeVisualizacion.intersects(x, y, x + ancho, y + alto);
    }

    /**
     * Aplica la transformación de la cámara al canvas
     */
    public void aplicarTransformacion(Canvas canvas) {
        canvas.setMatrix(matrizTransformacion);
    }

    /**
     * Resetea la transformación del canvas (útil para UI estática)
     */
    public void resetearTransformacion(Canvas canvas) {
        canvas.setMatrix(null);
    }

    public void setPosicion(float x, float y) {
        this.x = x;
        this.y = y;
        aplicarLimites();
        actualizarTransformacion();
    }

    public float getX() {
        return this.x;
    }

    public void setX(float x) {
        this.x = x;
        aplicarLimites();
        actualizarTransformacion();
    }

    public float getY() {
        return this.y;
    }

    public void setY(float y) {
        this.y = y;
        aplicarLimites();
        actualizarTransformacion();
    }

    public float getZoom() {
        return this.zoom;
    }

    public float getAncho() {
        return this.ancho;
    }

    public float getAlto() {
        return this.alto;
    }
}
