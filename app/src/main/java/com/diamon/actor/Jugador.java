package com.diamon.actor;

import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Jugador extends Actor {

    public Jugador(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Jugador(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Jugador(
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
    public void obtenerActores() {}

    @Override
    public void colision(Actor actor) {}
}
