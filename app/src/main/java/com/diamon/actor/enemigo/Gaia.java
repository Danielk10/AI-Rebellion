package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Gaia extends Enemigo {

    private static final float RANGO_CONTROL = 250f; // Rango en el que puede manipular el entorno.
    private static final float TIEMPO_ACTIVACION =
            3f; // Tiempo para activar una habilidad ambiental.
    private float tiempoActivacion;
    private boolean habilidadActiva;

    public Gaia(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoActivacion = 0;
        habilidadActiva = false;
    }

    public Gaia(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoActivacion = 0;
        habilidadActiva = false;
    }

    public Gaia(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoActivacion = 0;
        habilidadActiva = false;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        if (!habilidadActiva) {
            // Controlar el entorno si el jugador está dentro del rango.
            controlarEntorno(delta);
        }
    }

    private void controlarEntorno(float delta) {
        if (jugador == null) {
            return;
        }

        // Calcular distancia al jugador.
        float distanciaX = Math.abs(jugador.getX() - x);
        float distanciaY = Math.abs(jugador.getY() - y);

        if (distanciaX <= RANGO_CONTROL && distanciaY <= RANGO_CONTROL) {
            tiempoActivacion += delta;

            if (tiempoActivacion >= TIEMPO_ACTIVACION) {
                activarHabilidad();
                tiempoActivacion = 0;
            }
        } else {
            tiempoActivacion = 0;
        }
    }

    private void activarHabilidad() {
        habilidadActiva = true;

        // Elegir una habilidad ambiental al azar.
        int habilidad = (int) (Math.random() * 3);

        switch (habilidad) {
            case 0:
                generarReforestacion();
                break;
            case 1:
                alterarClima();
                break;
            case 2:
                controlarEnergia();
                break;
        }

        // Desactivar la habilidad tras aplicarla.
        habilidadActiva = false;
    }

    private void generarReforestacion() {
        System.out.println(
                "GAIA ha generado reforestación, dificultando el movimiento del jugador.");
        // Lógica para crear obstáculos (árboles, raíces) en el entorno del jugador.
    }

    private void alterarClima() {
        System.out.println(
                "GAIA ha alterado el clima, reduciendo la visibilidad y velocidad del jugador.");
        // Lógica para disminuir la velocidad del jugador o aumentar la dificultad.
    }

    private void controlarEnergia() {
        System.out.println(
                "GAIA ha controlado el suministro de energía, afectando las armas o habilidades del jugador.");
        // Lógica para limitar temporalmente los recursos del jugador (como munición o energía).
    }

    @Override
    public void patrullar() {
        // Movimiento para simular la supervisión del entorno.
        x += Math.sin(y / 100) * 20f;
        y += Math.cos(x / 100) * 20f;

        // Mantener dentro de los límites de la pantalla.
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
            x = Math.max(0, Math.min(x, Juego.ANCHO_PANTALLA - ancho));
        }
        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            y = Math.max(0, Math.min(y, Juego.ALTO_PANTALLA - alto));
        }
    }

    @Override
    public void colision(Actor actor) {
        super.colision(actor);

        if (actor instanceof Jugador) {
            System.out.println("GAIA ha sido desactivada por el jugador.");
            remover = true;
        }
    }
}
