package com.diamon.ui;

import android.graphics.Paint;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import android.graphics.Color;
import com.diamon.nucleo.Pantalla;

public class EtiquetaTexto extends Actor {

    private String texto;

    private Paint lapiz;

    private boolean obtenido;

    public EtiquetaTexto(Pantalla pantalla, float x, float y, String texto) {
        super(pantalla, null, x, y);

        this.texto = texto;
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
