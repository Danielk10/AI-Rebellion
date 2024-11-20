package com.diamon.actor.ambiente;

import com.diamon.actor.Ambiente;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Fondo extends Ambiente {

    public Fondo(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Fondo(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Fondo(
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
