package com.diamon.ui;

import android.graphics.Color;
import android.graphics.Paint;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;
import kotlinx.coroutines.internal.Symbol;

public class Boton extends Actor {

    private String texto;

    private boolean presionado;

    private Paint lapiz;

    private boolean obtenido;

    public Boton(
            Pantalla pantalla,
            Textura textura,
            float x,
            float y,
            float ancho,
            float alto,
            String texto) {
        super(pantalla, textura, x, y, ancho, alto);

        this.texto = texto;

        this.presionado = false;
    }

    public Boton(Pantalla pantalla, Textura textura, float x, float y, String texto) {
        super(pantalla, textura, x, y);

        this.texto = texto;

        this.presionado = false;
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
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);

        this.texto = texto;

        this.presionado = false;
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {
        super.dibujar(pincel, delta);

        if (!obtenido) {

            lapiz = pincel.getLapiz();

            obtenido = true;
        }

        pincel.dibujarTexto(texto, x + ancho / 4, y + alto / 2, Color.BLACK);
    }

    @Override
    public void obtenerActores() {}

    @Override
    public void colision(Actor actor) {}

    public String getTexto() {
        return this.texto;
    }

    public void setTexto(String texto) {
        this.texto = texto;
    }

    public boolean getPresionado() {
        return this.presionado;
    }

    public void setPresionado(boolean presionado) {
        this.presionado = presionado;
    }

    @Override
    public void setTamano(float ancho, float alto) {
        super.setTamano(ancho, alto);

        if (lapiz != null) {

            lapiz.setTextSize(ancho);
        }
    }

    public void setColor(int color) {

        if (lapiz != null) {

            lapiz.setColor(color);
        }
    }
}
