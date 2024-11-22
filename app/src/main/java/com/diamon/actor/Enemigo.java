package com.diamon.actor;

import com.diamon.graficos.Actor2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public abstract class Enemigo extends Actor2D {

    protected Jugador jugador;

    protected float dureza;

    public Enemigo(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
    }

    public Enemigo(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
    }

    public Enemigo(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
    }


    public void patrullar() {
        
    }

    @Override
    public void obtenerActores() {

        for (int i = 0; i < actores.size(); i++) {

            if (actores.get(i) instanceof Jugador) {

                jugador = (Jugador) actores.get(i);
            }
        }
    }

    @Override
    public void colision(Actor actor) {

        if (actor instanceof Jugador) {

            remover = true;
        }
    }

    public float getDureza() {
        return this.dureza;
    }
    
   public float getDurezaMaxima() {
        return this.dureza;
    }
    

    public void setDureza(float dureza) {
        this.dureza = dureza;
    }
}
