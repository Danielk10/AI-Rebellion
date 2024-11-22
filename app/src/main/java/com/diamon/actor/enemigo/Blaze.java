package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Blaze extends Enemigo {

    private static final float VELOCIDAD = 60f; // Velocidad de movimiento.
    private static final float RANGO_ATAQUE = 50f; // Rango de ataque cuerpo a cuerpo.
    private static final float TIEMPO_ATAQUE = 1.5f; // Tiempo entre ataques.
    private float tiempoUltimoAtaque;
    
    private float velocidad;

    public Blaze(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoUltimoAtaque = 0;
        velocidad = VELOCIDAD;
    }

    public Blaze(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoUltimoAtaque = 0;
       velocidad = VELOCIDAD;
    }

    public Blaze(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoUltimoAtaque = 0;
        
       velocidad = VELOCIDAD;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

      //  patrullar();
        // Movimiento hacia el jugador si está cerca
        acercarseAlJugador(delta);

        // Intentar atacar si está en rango
        intentarAtacar(delta);
    }

    @Override
    public void patrullar() {
        // Movimiento básico si el jugador no está cerca
        x += velocidad * 0.02f;

        // Mantener Blaze dentro de los límites de la pantalla
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
            x = Math.max(0, Math.min(x, Juego.ANCHO_PANTALLA - ancho));
        }
        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            y = Math.max(0, Math.min(y, Juego.ALTO_PANTALLA - alto));
        }
    }

    private void acercarseAlJugador(float delta) {
        if (jugador == null) {
            return;
        }

        // Movimiento hacia el jugador si está en rango visual pero fuera de rango de ataque
        float distanciaX = jugador.getX() - x;
        float distanciaY = jugador.getY() - y;
        float distanciaTotal = (float) Math.sqrt(distanciaX * distanciaX + distanciaY * distanciaY);

        if (distanciaTotal > RANGO_ATAQUE && distanciaTotal < 300f) { // 300 es rango visual
            x += velocidad * (distanciaX / distanciaTotal) * delta;
            y += velocidad * (distanciaY / distanciaTotal) * delta;
        }
    }

    private void intentarAtacar(float delta) {
        if (jugador == null) {
            return;
        }

        // Calcular distancia al jugador
        float distanciaX = Math.abs(jugador.getX() - x);
        float distanciaY = Math.abs(jugador.getY() - y);

        if (distanciaX <= RANGO_ATAQUE && distanciaY <= RANGO_ATAQUE) {
            // Si está en rango, atacar
            tiempoUltimoAtaque += delta;

            if (tiempoUltimoAtaque >= TIEMPO_ATAQUE) {
                realizarAtaque();
                tiempoUltimoAtaque = 0;
            }
        }
    }

    private void realizarAtaque() {
        if (jugador != null) {
            // Reducir la vida del jugador
            jugador.setVida(jugador.getVida() - 10); // Inflige 10 de daño por ataque
            System.out.println("Blaze ha golpeado al jugador, infligiendo 10 de daño.");
        }
    }

    @Override
    public void colision(Actor actor) {
        super.colision(actor);

        if (actor instanceof Jugador) {
            System.out.println("Blaze ha sido destruido por el jugador.");
            remover = true;
        }
    }
}
