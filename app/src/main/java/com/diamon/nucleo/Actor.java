package com.diamon.nucleo;

import com.diamon.graficos.Animacion2D;
import com.diamon.utilidad.Rectangulo;


public interface Actor {

    public abstract void actualizar(float delta);

    public abstract void dibujar(Graficos pincel, float delta);

    public abstract void obtenerActores();

    public abstract void colision(Actor actor);

    public abstract void setPosicion(float x, float y);

    public abstract void setAnimacion(Animacion2D animacion);

    public abstract Animacion2D getAnimacion();

    public abstract Rectangulo getRectangulo();

    public abstract float getX();

    public abstract void setX(float x);

    public abstract float getY();

    public abstract void setY(float y);

    public abstract void setAncho(float ancho);

    public abstract float getAncho();

    public abstract void setAlto(float alto);

    public abstract float getAlto();

    public abstract void setTamano(float ancho, float alto);

    public abstract void remover();

    public abstract boolean isRemover();
}
