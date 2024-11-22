package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;
import com.diamon.nucleo.Actor;

public class Orion extends Enemigo {

    private float tiempoAnalisis;
    private static final float TIEMPO_ANALISIS_MAXIMO = 2f; // 2 segundos para prever el siguiente movimiento.
    private boolean enDefensiva;
    private static final float VELOCIDAD = 50f; // Velocidad de movimiento de Nyx.
    
    
    private float velocidad;

    public Orion(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoAnalisis = 0;
        
        enDefensiva = false;
        
        velocidad = VELOCIDAD;
    }

    public Orion(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoAnalisis = 0;
        enDefensiva = false;
        
       velocidad = VELOCIDAD;
    }

    public Orion(Pantalla pantalla, Textura[] texturas, float x, float y, float ancho, float alto, float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoAnalisis = 0;
        
        
        enDefensiva = false;
        
        
       velocidad = VELOCIDAD;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        // Analizar el entorno para prever ataques
        analizarEntorno(delta);

        // Decidir si atacar o moverse defensivamente
        if (enDefensiva) {
            moverseDefensivamente();
        } else {
            atacarJugador();
        }
    }

    private void analizarEntorno(float delta) {
        if (jugador == null) {
            return;
        }

        tiempoAnalisis += delta;

        if (tiempoAnalisis >= TIEMPO_ANALISIS_MAXIMO) {
            // Evaluar la posición del jugador y su proximidad
            float distanciaX = Math.abs(jugador.getX() - x);
            float distanciaY = Math.abs(jugador.getY() - y);

            if (distanciaX <= 200 && distanciaY <= 200) {
                enDefensiva = false; // Prepararse para atacar si el jugador está cerca.
            } else {
                enDefensiva = true; // Moverse defensivamente si el jugador está lejos.
            }

            tiempoAnalisis = 0; // Reiniciar el análisis
        }
    }

    private void moverseDefensivamente() {
        // Realizar un movimiento evasivo en zigzag
        x += velocidad * Math.cos(y / 50) * 0.1f;
        y -= velocidad * 0.02f;

        // Mantener a Orion dentro de los límites de la pantalla
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
            velocidad *= -1; // Cambiar dirección horizontal.
        }

        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            y = Math.max(0, Math.min(y, Juego.ALTO_PANTALLA - alto));
        }

        System.out.println("Orion se mueve defensivamente.");
    }

    private void atacarJugador() {
        if (jugador == null) {
            return;
        }

        // Disparar proyectiles al jugador
        System.out.println("Orion está atacando al jugador.");
        // Lógica para disparar un proyectil hacia la posición del jugador (implementación según tu sistema de disparos).
    }

   
}