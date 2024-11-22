package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Zero extends Enemigo {

    private static final float RANGO_INFILTRACION = 300f; // Rango para iniciar sabotaje.
    private static final float TIEMPO_SABOTAJE = 5f; // Tiempo necesario para sabotear un sistema.
    private static final float TIEMPO_ENFRIAMIENTO = 10f; // Tiempo entre sabotajes.
    private float tiempoSabotaje;
    private float tiempoEnfriamiento;
    private boolean enModoFurtivo;

    public Zero(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoSabotaje = 0;
        tiempoEnfriamiento = 0;
        enModoFurtivo = true; // Zero comienza en modo furtivo.
    }

    public Zero(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoSabotaje = 0;
        tiempoEnfriamiento = 0;
        enModoFurtivo = true;
    }

    public Zero(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoSabotaje = 0;
        tiempoEnfriamiento = 0;
        enModoFurtivo = true;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        if (enModoFurtivo) {
            infiltrarse(delta);
        } else {
            patrullar();
        }

        if (tiempoEnfriamiento > 0) {
            tiempoEnfriamiento -= delta;
        }
    }

    private void infiltrarse(float delta) {
        if (jugador == null) {
            return;
        }

        // Calcular distancia al jugador.
        float distanciaX = Math.abs(jugador.getX() - x);
        float distanciaY = Math.abs(jugador.getY() - y);

        if (distanciaX <= RANGO_INFILTRACION
                && distanciaY <= RANGO_INFILTRACION
                && tiempoEnfriamiento <= 0) {
            tiempoSabotaje += delta;

            if (tiempoSabotaje >= TIEMPO_SABOTAJE) {
                ejecutarSabotaje();
                tiempoSabotaje = 0;
                tiempoEnfriamiento = TIEMPO_ENFRIAMIENTO;
            }
        } else {
            tiempoSabotaje = 0; // Reiniciar si el jugador sale del rango.
        }
    }

    private void ejecutarSabotaje() {
        System.out.println("Zero ha saboteado los sistemas del jugador.");
        // Ejemplo de efectos de sabotaje:
        jugador.reducirVelocidad(
                0.5f, 5f); // Reduce la velocidad del jugador al 50% por 5 segundos.
        jugador.desactivarArmas(3f); // Desactiva las armas del jugador por 3 segundos.
    }

    @Override
    public void patrullar() {
        // Patrón de movimiento para simular vigilancia.
        x += Math.sin(y / 100) * 15f;
        y += Math.cos(x / 100) * 15f;

        // Mantener dentro de los límites de la pantalla.
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
            x = Math.max(0, Math.min(x, Juego.ANCHO_PANTALLA - ancho));
        }
        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            y = Math.max(0, Math.min(y, Juego.ALTO_PANTALLA - alto));
        }
    }

   
    public void cambiarModo(boolean furtivo) {
        enModoFurtivo = furtivo;
        if (furtivo) {
            System.out.println("Zero ha entrado en modo furtivo.");
        } else {
            System.out.println("Zero ha salido del modo furtivo.");
        }
    }
}
