package com.diamon.actor.jugador;

import com.diamon.actor.Jugador;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Jugador1 extends Jugador {

    public Jugador1(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Jugador1(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Jugador1(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
    }

    
}
