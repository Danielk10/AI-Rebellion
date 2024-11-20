package com.diamon.graficos;

import com.diamon.dato.ConfiguracionesDeJuego;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;
import com.diamon.utilidad.Rectangulo;
import com.diamon.utilidad.Recurso;

import java.util.ArrayList;

public abstract class Actor2D implements Actor {

    protected float x;

    protected float y;

    protected float ancho;

    protected float alto;

    protected boolean remover;

    private float tiempo;

    protected Textura[] texturas;

    protected Animacion2D animacion;

    protected Pantalla pantalla;

    protected Camara2D camara;

    protected Recurso recurso;

    private float tiempoAnimacion;

    protected ConfiguracionesDeJuego configuracionesDeJuego;

    protected ArrayList<Actor> actores;

    private boolean animar;

    public Actor2D(Pantalla pantalla, Textura textura, float x, float y) {

        this.pantalla = pantalla;

        this.actores = ((Pantalla2D) pantalla).getActores();

        this.recurso = ((Pantalla2D) pantalla).getJuego().getRecurso();

        this.camara = ((Pantalla2D) pantalla).getCamara();

        this.configuracionesDeJuego =
                ((Pantalla2D) pantalla).getJuego().getConfiguracionesDeJuego();

        this.x = x;

        this.y = y;

        tiempo = 0;

        texturas = new Textura[1];

        texturas[0] = textura;

        tiempoAnimacion = 1;

        if (textura != null) {

            animacion = new Animacion2D(tiempoAnimacion / Juego.FPS, texturas);

            animacion.setModo(Animacion2D.NORMAL);

            ancho = textura.getAncho();

            alto = textura.getAlto();
        }

        remover = false;

        animar = false;

        obtenerActores();
    }

    public Actor2D(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {

        this.pantalla = pantalla;

        this.actores = ((Pantalla2D) pantalla).getActores();

        this.recurso = ((Pantalla2D) pantalla).getJuego().getRecurso();

        this.camara = ((Pantalla2D) pantalla).getCamara();

        this.configuracionesDeJuego =
                ((Pantalla2D) pantalla).getJuego().getConfiguracionesDeJuego();

        this.x = x;

        this.y = y;

        tiempo = 0;

        texturas = new Textura[1];

        texturas[0] = textura;

        tiempoAnimacion = 1;

        if (textura != null) {
            animacion = new Animacion2D(tiempoAnimacion / Juego.FPS, texturas);

            animacion.setModo(Animacion2D.NORMAL);
        }

        this.ancho = ancho;

        this.alto = alto;

        this.setTamano(ancho, alto);

        remover = false;

        animar = false;

        obtenerActores();
    }

    public Actor2D(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {

        this.pantalla = pantalla;

        this.actores = ((Pantalla2D) pantalla).getActores();

        this.recurso = ((Pantalla2D) pantalla).getJuego().getRecurso();

        this.camara = ((Pantalla2D) pantalla).getCamara();

        this.configuracionesDeJuego =
                ((Pantalla2D) pantalla).getJuego().getConfiguracionesDeJuego();

        this.x = x;

        this.y = y;

        tiempo = 0;

        this.texturas = texturas;

        this.tiempoAnimacion = tiempoAnimacion;

        if (texturas != null && texturas.length > 0) {
            animacion = new Animacion2D(tiempoAnimacion / Juego.FPS, texturas);

            animacion.setModo(Animacion2D.REPETIR);
        }

        this.ancho = ancho;

        this.alto = alto;

        this.setTamano(ancho, alto);

        remover = false;

        animar = true;

        obtenerActores();
    }

    @Override
    public void setAnimacion(Animacion2D animacion) {
        this.animacion = animacion;
    }

    @Override
    public Animacion2D getAnimacion() {
        return animacion;
    }

    @Override
    public void setPosicion(float x, float y) {

        this.x = x;

        this.y = y;
    }

    @Override
    public boolean isRemover() {

        return remover;
    }

    @Override
    public void actualizar(float delta) {

        if (animar) {

            if (delta == 0) {

                return;
            }

            if (delta > 0.1f) {

                delta = 0.1f;
            }

            tiempo += delta;
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {

        if (animacion != null) {

            pincel.dibujarTextura(animacion.getKeyFrame(tiempo), x, y);
        }
    }

    @Override
    public float getX() {
        return x;
    }

    @Override
    public void setX(float x) {
        this.x = x;
    }

    @Override
    public float getY() {
        return y;
    }

    @Override
    public void setY(float y) {
        this.y = y;
    }

    @Override
    public void setAncho(float ancho) {

        this.ancho = ancho;

        if (texturas != null && texturas.length > 0) {

            for (int i = 0; i < texturas.length; i++) {

                texturas[i] = new Textura2D(texturas[i].getBipmap(), ancho, alto);
            }
        }
    }

    @Override
    public void setAlto(float alto) {

        this.alto = alto;

        if (texturas != null && texturas.length > 0) {

            for (int i = 0; i < texturas.length; i++) {

                texturas[i] = new Textura2D(texturas[i].getBipmap(), ancho, alto);
            }
        }
    }

    @Override
    public float getAncho() {
        return ancho;
    }

    @Override
    public float getAlto() {
        return alto;
    }

    @Override
    public Rectangulo getRectangulo() {
        return new Rectangulo(x, y, ancho, alto);
    }

    @Override
    public void setTamano(float ancho, float alto) {

        this.ancho = ancho;

        this.alto = alto;

        if (texturas != null && texturas.length > 0) {

            for (int i = 0; i < texturas.length; i++) {

                texturas[i] = new Textura2D(texturas[i].getBipmap(), ancho, alto);
            }
        }
    }

    @Override
    public void remover() {

        remover = true;
    }
}
