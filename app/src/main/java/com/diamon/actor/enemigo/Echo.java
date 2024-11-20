package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Echo extends Enemigo {

    public Echo(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Echo(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Echo(
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
