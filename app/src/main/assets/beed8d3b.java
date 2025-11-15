package com.diamon.actores;

import com.diamon.graficos.Animacion2D;
import com.diamon.graficos.Camara2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.utilidad.Rectangulo;

/**
 * Clase base abstracta para implementar actores del juego
 * Proporciona funcionalidad común para todos los actores
 */
public abstract class ActorBase implements Actor {

    protected float x;
    protected float y;
    protected float ancho;
    protected float alto;
    protected Animacion2D animacion;
    protected boolean remover;
    protected Rectangulo rectangulo;

    // MEJORA: Tiempo de vida de la animación
    protected float tiempoAnimacion;

    public ActorBase(float x, float y, float ancho, float alto) {
        this.x = x;
        this.y = y;
        this.ancho = ancho;
        this.alto = alto;
        this.remover = false;
        this.tiempoAnimacion = 0;
        this.rectangulo = new Rectangulo(x, y, ancho, alto);
    }

    @Override
    public void actualizar(float delta) {
        // Actualizar tiempo de animación
        if (animacion != null) {
            tiempoAnimacion += delta;
        }

        // Actualizar rectángulo de colisión
        rectangulo.setPosicion(x, y);
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {
        if (animacion != null) {
            pincel.dibujarTextura(animacion.getKeyFrame(tiempoAnimacion), x, y);
        }
    }

    /**
     * MEJORA: Método para verificar si el actor es visible en la cámara
     */
    public boolean esVisibleEnCamara(Camara2D camara) {
        return camara.esVisible(x, y, ancho, alto);
    }

    /**
     * MEJORA: Método para dibujar solo si es visible (culling automático)
     */
    public void dibujarConCulling(Graficos pincel, Camara2D camara, float delta) {
        if (esVisibleEnCamara(camara)) {
            dibujar(pincel, delta);
        }
    }

    @Override
    public void colision(Actor actor) {
        // Implementación específica en subclases
    }

    @Override
    public void obtenerActores() {
        // Implementación específica en subclases
    }

    @Override
    public void setPosicion(float x, float y) {
        this.x = x;
        this.y = y;
        rectangulo.setPosicion(x, y);
    }

    @Override
    public void setAnimacion(Animacion2D animacion) {
        this.animacion = animacion;
        this.tiempoAnimacion = 0; // Reiniciar tiempo de animación
    }

    @Override
    public Animacion2D getAnimacion() {
        return animacion;
    }

    @Override
    public Rectangulo getRectangulo() {
        return rectangulo;
    }

    @Override
    public float getX() {
        return x;
    }

    @Override
    public void setX(float x) {
        this.x = x;
        rectangulo.setPosicion(x, y);
    }

    @Override
    public float getY() {
        return y;
    }

    @Override
    public void setY(float y) {
        this.y = y;
        rectangulo.setPosicion(x, y);
    }

    @Override
    public void setAncho(float ancho) {
        this.ancho = ancho;
        rectangulo.setAncho(ancho);
    }

    @Override
    public float getAncho() {
        return ancho;
    }

    @Override
    public void setAlto(float alto) {
        this.alto = alto;
        rectangulo.setAlto(alto);
    }

    @Override
    public float getAlto() {
        return alto;
    }

    @Override
    public void setTamano(float ancho, float alto) {
        this.ancho = ancho;
        this.alto = alto;
        rectangulo.setTamano(ancho, alto);
    }

    @Override
    public void remover() {
        this.remover = true;
    }

    @Override
    public boolean isRemover() {
        return remover;
    }

    /**
     * MEJORA: Método para resetear el tiempo de animación
     */
    public void resetearAnimacion() {
        this.tiempoAnimacion = 0;
    }

    /**
     * MEJORA: Obtener el centro del actor
     */
    public float getCentroX() {
        return x + ancho / 2;
    }

    public float getCentroY() {
        return y + alto / 2;
    }
}
