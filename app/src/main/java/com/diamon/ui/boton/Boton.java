package com.diamon.ui.boton;

import android.graphics.Color;
import android.graphics.Paint;

import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;
import com.diamon.ui.Elemento;

public class Boton extends Elemento {

    public Boton(
            Pantalla pantalla,
            Textura textura,
            float x,
            float y,
            float ancho,
            float alto,
            String texto) {
        super(pantalla, textura, x, y, ancho, alto, texto);
    }

    public Boton(Pantalla pantalla, Textura textura, float x, float y, String texto) {
        super(pantalla, textura, x, y, texto);
    }

    public Boton(
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
