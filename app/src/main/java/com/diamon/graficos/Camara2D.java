package com.diamon.graficos;

import android.graphics.Canvas;
import android.graphics.Matrix;
import android.graphics.RectF;

public class Camara2D {

    private float x; // posición X de la cámara
    private float y; // posición Y de la cámara
    private float ancho; // ancho de la cámara
    private float alto; // alto de la cámara
    private float zoom; // nivel de zoom

    private Matrix
            matrizTransformacion; // matriz de transformación para aplicar zoom y desplazamiento
    private RectF areaDeVisualizacion; // área visible de la cámara

    public Camara2D(float ancho, float alto) {
        this.ancho = ancho;
        this.alto = alto;
        this.zoom = 1.0f;
        this.x = 0;
        this.y = 0;
        this.matrizTransformacion = new Matrix();
        this.areaDeVisualizacion = new RectF(0, 0, ancho, alto);
        actualizarTransformacion();
    }

    // Actualiza la matriz de transformación y el área de visualización
    private void actualizarTransformacion() {
        matrizTransformacion.reset();
        matrizTransformacion.postScale(zoom, zoom);
        matrizTransformacion.postTranslate(-x * zoom + ancho / 2, -y * zoom + alto / 2);
        areaDeVisualizacion.set(
                x - ancho / (2 * zoom),
                y - alto / (2 * zoom),
                x + ancho / (2 * zoom),
                y + alto / (2 * zoom));
    }

    // Mueve la cámara a una posición específica
    public void moverA(float x, float y) {
        this.x = x;
        this.y = y;
        actualizarTransformacion();
    }

    // Cambia el zoom de la cámara
    public void setZoom(float zoom) {
        this.zoom = zoom;
        actualizarTransformacion();
    }

    // Convierte las coordenadas de pantalla a las del mundo
    public float[] convertirPantallaAMundo(float pantallaX, float pantallaY) {
        float[] punto = new float[] {pantallaX, pantallaY};
        Matrix matrizInvertida = new Matrix();
        matrizTransformacion.invert(matrizInvertida);
        matrizInvertida.mapPoints(punto);
        return punto;
    }

    // Convierte las coordenadas del mundo a las de pantalla
    public float[] convertirMundoAPantalla(float mundoX, float mundoY) {
        float[] punto = new float[] {mundoX, mundoY};
        matrizTransformacion.mapPoints(punto);
        return punto;
    }

    // Obtiene el área visible de la cámara
    public RectF getAreaDeVisualizacion() {
        return areaDeVisualizacion;
    }

    // Dibuja utilizando la cámara
    public void aplicarTransformacion(Canvas canvas) {
        canvas.setMatrix(matrizTransformacion);
    }

    public void setPosicion(float x, float y) {
        this.x = x;
        this.y = y;
        actualizarTransformacion();
    }

    public float getX() {
        return this.x;
    }

    public void setX(float x) {
        this.x = x;

        moverA(x, y);
    }

    public float getY() {
        return this.y;
    }

    public void setY(float y) {
        this.y = y;

        moverA(x, y);
    }
}
