package com.diamon.ui;

import android.graphics.Color;
import android.graphics.Paint;

import com.diamon.graficos.Actor2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public abstract class Elemento extends Actor2D {

    protected String texto;

    protected int color;

    protected float tamano;

    protected boolean presionado;

    public Elemento(
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

        color = Color.BLACK;

        tamano = 18;
    }

    public Elemento(Pantalla pantalla, Textura textura, float x, float y, String texto) {
        super(pantalla, textura, x, y);

        this.texto = texto;

        this.presionado = false;

        color = Color.BLACK;

        tamano = 18;
    }

    public Elemento(
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

        color = Color.BLACK;

        tamano = 18;
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {
        super.dibujar(pincel, delta);

        pincel.getLapiz().setTextSize(tamano);

        pincel.dibujarTexto(texto, x + ancho / 4, y + alto / 2, color);
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

    public void setTamanoTexto(float tamano) {

        this.tamano = tamano;
    }

    public void setColor(int color) {

        this.color = color;
    }
}
