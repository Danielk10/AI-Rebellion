package com.diamon.actor.enemigo;

import com.diamon.actor.Enemigo;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Lux extends Enemigo {

    private float tiempoCuracion;
    private static final float TIEMPO_MAXIMO_CURACION = 4f; // Cada 4 segundos realiza una curación.
    private static final float RANGO_CURACION =
            200f; // Rango dentro del cual puede sanar a otros enemigos.
    private static final float VELOCIDAD = 30f; // Velocidad de movimiento.

    private float velocidad;

    public Lux(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        tiempoCuracion = 0;

        velocidad = VELOCIDAD;
    }

    public Lux(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        tiempoCuracion = 0;

        velocidad = VELOCIDAD;
    }

    public Lux(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        tiempoCuracion = 0;

        velocidad = VELOCIDAD;
    }

    @Override
    public void actualizar(float delta) {
        super.actualizar(delta);

        // Movimiento
        patrullar();

        // Intentar curar a otros enemigos
        intentarCurar(delta);
    }

    @Override
    public void patrullar() {
        // Movimiento básico: en zigzag horizontal y vertical
        x += velocidad * Math.sin(y / 50) * 0.1f;
        y += velocidad * 0.02f;

        // Mantener dentro de los límites de la pantalla
        if (x < 0 || x + ancho > Juego.ANCHO_PANTALLA) {
            x = Math.max(0, Math.min(x, Juego.ANCHO_PANTALLA - ancho));
        }
        if (y < 0 || y + alto > Juego.ALTO_PANTALLA) {
            y = Math.max(0, Math.min(y, Juego.ALTO_PANTALLA - alto));
        }
    }

    private void intentarCurar(float delta) {
        // Incrementar tiempo acumulado
        tiempoCuracion += delta;

        if (tiempoCuracion >= TIEMPO_MAXIMO_CURACION) {
            // Buscar un enemigo dañado en rango y curarlo
            Enemigo enemigoACurar = encontrarEnemigoCercano();

            if (enemigoACurar != null) {
                curarEnemigo(enemigoACurar);
            }

            // Reiniciar contador
            tiempoCuracion = 0;
        }
    }

    private Enemigo encontrarEnemigoCercano() {
        // Buscar entre los enemigos en pantalla
        for (Actor actor : actores) {
            if (actor instanceof Enemigo && actor != this) {
                Enemigo enemigo = (Enemigo) actor;
                float distancia = calcularDistancia(this.x, this.y, enemigo.getX(), enemigo.getY());

                // Verificar si está dentro del rango de curación
                if (distancia <= RANGO_CURACION
                        && enemigo.getDureza() < enemigo.getDurezaMaxima()) {
                    return enemigo;
                }
            }
        }
        return null;
    }

    private void curarEnemigo(Enemigo enemigo) {
        // Restaurar un porcentaje de la vida del enemigo
        float vidaRestaurada = enemigo.getDurezaMaxima() * 0.25f; // Curar el 25% de la vida máxima
        enemigo.setDureza(
                Math.min(enemigo.getDureza() + vidaRestaurada, enemigo.getDurezaMaxima()));
        System.out.println("Lux ha curado a un enemigo: " + enemigo.getClass().getSimpleName());
    }

    private float calcularDistancia(float x1, float y1, float x2, float y2) {
        return (float) Math.sqrt(Math.pow(x2 - x1, 2) + Math.pow(y2 - y1, 2));
    }

    @Override
    public void colision(Actor actor) {
        super.colision(actor);

        if (actor instanceof Jugador) {
            System.out.println("Lux ha sido destruido por el jugador.");
            remover = true;
        }
    }
}
