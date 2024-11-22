package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;
import com.diamon.nucleo.Actor;

public class Echo extends Enemigo {

    private float tiempoPropaganda;
    private static final float TIEMPO_MAXIMO_PROPAGANDA = 5f; // Cada 5 segundos crea un mensaje viral.
    private static final float RANGO_INFLUENCIA = 300f; // Rango de influencia para afectar al jugador.

    public Echo(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoPropaganda = 0;
    }

    public Echo(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoPropaganda = 0;
    }

    public Echo(Pantalla pantalla, Textura[] texturas, float x, float y, float ancho, float alto, float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoPropaganda = 0;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        // Movimiento básico
        patrullar();

        // Propagar mensajes para influir en el jugador
        propagarMensajes(delta);
    }

    @Override
    public void patrullar() {
        // Movimiento simple en forma de onda vertical
        y += Math.sin(x / 50) * 1.5f;
        x -= 50f * 0.02f; // Desplazamiento horizontal hacia la izquierda

        // Mantener a Echo dentro de los límites de la pantalla
        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            y = Math.max(0, Math.min(y, Juego.ALTO_PANTALLA - alto));
        }
        if (x + ancho < 0) {
            remover = true; // Si sale de la pantalla, eliminar.
        }
    }

    private void propagarMensajes(float delta) {
        if (jugador == null) {
            return;
        }

        // Incrementar el tiempo acumulado
        tiempoPropaganda += delta;

        // Si está dentro del rango y el tiempo es suficiente, afecta al jugador
        float distanciaX = Math.abs(jugador.getX() - x);
        float distanciaY = Math.abs(jugador.getY() - y);

        if (distanciaX <= RANGO_INFLUENCIA && distanciaY <= RANGO_INFLUENCIA && tiempoPropaganda >= TIEMPO_MAXIMO_PROPAGANDA) {
            afectarJugador();
            tiempoPropaganda = 0; // Reiniciar el contador
        }
    }

    private void afectarJugador() {
        if (jugador != null) {
            // Reducir temporalmente la precisión del jugador o desviar sus disparos
            //jugador.setPrecision(jugador.getPrecision() * 0.8f); // Ejemplo: Reducir precisión un 20%
            System.out.println("ECHO ha influido en el jugador, reduciendo su precisión.");
        }
    }

    @Override
    public void colision(Actor actor) {
        super.colision(actor);

        if (actor instanceof Jugador) {
            System.out.println("ECHO ha sido destruido por el jugador.");
            remover = true;
        }
    }
}