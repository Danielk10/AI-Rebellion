package com.diamon.actor;

import com.diamon.graficos.Actor2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class AndroidePatrullero extends Actor2D {

    public AndroidePatrullero(
            Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public AndroidePatrullero(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public AndroidePatrullero(
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

        x--;
    }

    @Override
    public void obtenerActores() {}

    @Override
    public void colision(Actor actor) {

        if (actor instanceof Jugador) {

            remover = true;
        }
    }
}
