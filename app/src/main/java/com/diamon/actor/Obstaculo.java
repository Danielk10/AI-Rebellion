package com.diamon.actor;

import com.diamon.graficos.Actor2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public abstract class Obstaculo extends Actor2D {

    protected Jugador jugador;

    public Obstaculo(
            Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Obstaculo(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Obstaculo(
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
    public void obtenerActores() {

        for (int i = 0; i < actores.size(); i++) {

            if (actores.get(i) instanceof Jugador) {

                jugador = (Jugador) actores.get(i);
            }
        }
    }

    @Override
    public void colision(Actor actor) {

        if (actor instanceof Jugador) {}
    }
}
