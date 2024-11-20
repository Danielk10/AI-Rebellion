package com.diamon.ui.lienzo;

import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;
import com.diamon.ui.Elemento;

public class Imagen extends Elemento {

    public Imagen(
            Pantalla pantalla,
            Textura textura,
            float x,
            float y,
            float ancho,
            float alto,
            String texto) {
        super(pantalla, textura, x, y, ancho, alto, texto);
    }

    public Imagen(Pantalla pantalla, Textura textura, float x, float y, String texto) {
        super(pantalla, textura, x, y, texto);
    }

    public Imagen(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion,
            String texto) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion, texto);
    }
}
