package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Nebula extends Enemigo {

    private static final float VELOCIDAD = 40f; // Velocidad de exploración.
    private static final float RANGO_DETECCION = 200f; // Rango para detectar al jugador.
    private static final float TIEMPO_ANALISIS = 2f; // Tiempo necesario para completar el análisis.
    private float tiempoAnalizando;
    private boolean analizando;

    private float velocidad;

    public Nebula(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoAnalizando = 0;
        analizando = false;

        velocidad = VELOCIDAD;
    }

    public Nebula(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoAnalizando = 0;
        analizando = false;

        velocidad = VELOCIDAD;
    }

    public Nebula(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoAnalizando = 0;
        analizando = false;

        velocidad = VELOCIDAD;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        if (!analizando) {
            // Movimiento continuo de exploración.
            explorar(delta);

            // Intentar detectar al jugador.
            detectarJugador(delta);
        } else {
            // Analizar al jugador si está en rango.
            analizarJugador(delta);
        }
    }

    @Override
    public void patrullar() {
        // Movimiento simple de reconocimiento.
        x += velocidad * Math.cos(y / 50) * 0.1f;
        y += velocidad * Math.sin(x / 50) * 0.1f;

        // Mantener dentro de los límites de la pantalla.
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
            velocidad *= -1;
        }
        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            velocidad *= -1;
        }
    }

    private void explorar(float delta) {
        // Movimiento exploratorio por el mapa.
       patrullar();
    }

    private void detectarJugador(float delta) {
        if (jugador == null) {
            return;
        }

        // Calcular distancia al jugador.
        float distanciaX = Math.abs(jugador.getX() - x);
        float distanciaY = Math.abs(jugador.getY() - y);

        if (distanciaX <= RANGO_DETECCION && distanciaY <= RANGO_DETECCION) {
            System.out.println("Nebula ha detectado al jugador.");
            analizando = true;
        }
    }

    private void analizarJugador(float delta) {
        tiempoAnalizando += delta;

        if (tiempoAnalizando >= TIEMPO_ANALISIS) {
            enviarDatos();
            tiempoAnalizando = 0;
            analizando = false;
        }
    }

    private void enviarDatos() {
        if (jugador != null) {
            // Simular la creación de rutas de escape y transmitirlas a otros enemigos.
            System.out.println(
                    "Nebula ha analizado al jugador y ha enviado datos estratégicos a otros enemigos.");
            // Potencialmente, este método puede fortalecer a otros enemigos o activar trampas.
        }
    }

    
}
