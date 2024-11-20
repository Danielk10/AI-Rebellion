package com.diamon.actor;

import com.diamon.graficos.Actor2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public abstract class Arma extends Actor2D {

    public Arma(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Arma(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Arma(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);
    }

    @Override
    public void obtenerActores() {}

    @Override
    public void colision(Actor actor) {

        if (actor instanceof Jugador) {}
    }
}
