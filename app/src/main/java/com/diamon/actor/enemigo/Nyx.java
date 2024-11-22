package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Nyx extends Enemigo {

    private float tiempoHackeo;
    
    private static final float TIEMPO_MAXIMO_HACKEO = 3f; // 3 segundos para intentar hackeo.
    
    private static final float VELOCIDAD = 50f; // Velocidad de movimiento de Nyx.
    
    private float velocidad;

    public Nyx(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoHackeo = 0;
        
        velocidad = VELOCIDAD;
    }

    public Nyx(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoHackeo = 0;
        velocidad = VELOCIDAD;
    }

    public Nyx(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoHackeo = 0;
       velocidad = VELOCIDAD;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        // Movimiento sigiloso en zigzag para evitar ataques directos
        patrullar();

        // Intento de hackeo si Nyx está cerca del jugador
        intentarHackeo(delta);
    }

    @Override
    public void patrullar() {
        // Movimiento en zigzag a lo largo de la pantalla
        x -= velocidad * Math.sin(y / 50) * 0.1f;
        y += velocidad * 0.02f;

        // Mantener Nyx dentro de los límites de la pantalla
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
          velocidad  *= -1; // Cambiar dirección en el eje X.
        }
        
        if(y+alto>Juego.ALTO_PANTALLA || y<0) {
            
           velocidad  *= -1;
        	
        }
        
    }

    private void intentarHackeo(float delta) {
        if (jugador == null) {
            return;
        }

        // Si Nyx está cerca del jugador, intentar hackeo
        float distanciaX = Math.abs(jugador.getX() - x);
        float distanciaY = Math.abs(jugador.getY() - y);

        if (distanciaX <= 100 && distanciaY <= 100) { // Rango de hackeo
            tiempoHackeo += delta;

            if (tiempoHackeo >= TIEMPO_MAXIMO_HACKEO) {
                hackearJugador();
                tiempoHackeo = 0;
            }
        } else {
            tiempoHackeo = 0; // Reiniciar si está fuera de rango.
        }
    }

    private void hackearJugador() {
        if (jugador != null) {
            // Reducir velocidad o habilidades del jugador temporalmente
            jugador.setVelocidad(jugador.getVelocidad() * 0.5f);
            System.out.println("Nyx ha hackeado al jugador. Velocidad reducida temporalmente.");
        }
    }

    @Override
    public void colision(Actor actor) {
        super.colision(actor);

        if (actor instanceof Jugador) {
            System.out.println("Nyx ha sido destruida por colisión con el jugador.");
            remover = true;
        }
    }
}
