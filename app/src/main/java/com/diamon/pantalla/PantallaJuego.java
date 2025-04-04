package com.diamon.pantalla;

import com.diamon.actor.jugador.Jugador1;
import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.actor.Jugador;
import com.diamon.escenario.EscenarioBosqueCibernetico;
import com.diamon.escenario.EscenarioCiudadDesolada;
import com.diamon.escenario.EscenarioFabricaDeDrones;
import com.diamon.graficos.Pantalla2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Escena;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.utilidad.Rectangulo;
import android.graphics.Color;

public class PantallaJuego extends Pantalla2D {

    private Jugador1 jugador;

    private Escena escena;

    private int numeroEscenaro;

    public PantallaJuego(final Juego juego) {
        super(juego);
    }

    private void cargarEscenario(int numeroEscenario) {

        if (escena != null) {

            escena.guardarDatos();

            escena.liberarRecursos();
        }
        switch (numeroEscenario) {
            case 1:
                escena = new EscenarioFabricaDeDrones(this, jugador);
                break;
            case 2:
                escena = new EscenarioCiudadDesolada(this, jugador);
                break;
            case 3:
                escena = new EscenarioBosqueCibernetico(this, jugador);
                break;
            default:
                break;
        }

        if (escena != null) {

            escena.iniciar();
        }
    }

    @Override
    public void mostrar() {

        numeroEscenaro = 1;

        jugador = new Jugador1(this, recurso.getTextura("texturas/creditos.png"), 500, 500, 32, 32);

        escena = null;

        cargarEscenario(numeroEscenaro);
    }

    @Override
    public void resume() {}

    @Override
    public void colisiones() {

        for (int i = 0; i < actores.size(); i++) {

            Actor actor1 = actores.get(i);

            Rectangulo rectangulo1 = actor1.getRectangulo();

            for (int j = i + 1; j < actores.size(); j++) {

                Actor actor2 = actores.get(j);

                Rectangulo rectangulo2 = actor2.getRectangulo();

                if (rectangulo1.intersecion(rectangulo2)) {

                    actor1.colision(actor2);

                    actor2.colision(actor1);
                }
            }

            Actor actor = actores.get(i);

            if (actor.isRemover()) {

                actores.remove(i);
            }
        }
    }

    @Override
    public void actualizar(float delta) {

        if (escena != null) {

            escena.actualizar(delta);

            if (escena.escenarioCompletado()) {

                numeroEscenaro++;

                cargarEscenario(numeroEscenaro);
            }
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {

        if (escena != null) {

            escena.dibujar(pincel, delta);
        }
    }

    @Override
    public void reajustarPantalla(int ancho, int alto) {

        if (escena != null) {

            escena.guardarDatos();
        }
    }

    @Override
    public void pausa() {

        if (escena != null) {

            escena.guardarDatos();
        }
    }

    @Override
    public void ocultar() {

        if (escena != null) {

            escena.guardarDatos();
        }
    }

    @Override
    public void liberarRecursos() {

        if (escena != null) {

            escena.guardarDatos();

            escena.liberarRecursos();
        }
    }

    @Override
    public void teclaPresionada(int codigoDeTecla) {

        jugador.teclaPresionada(codigoDeTecla);
    }

    @Override
    public void teclaLevantada(int codigoDeTecla) {
        jugador.teclaLevantada(codigoDeTecla);
    }

    @Override
    public void toquePresionado(float x, float y, int puntero) {
        jugador.toquePresionado(x, y, puntero);
    }

    @Override
    public void toqueLevantado(float x, float y, int puntero) {
        jugador.toqueLevantado(x, y, puntero);
        
      
    }

    @Override
    public void toqueDeslizando(float x, float y, int puntero) {

        jugador.toqueDeslizando(x, y, puntero);
        
         if (blueTooth.getDatos() != null) {

            blueTooth.getDatos().enviarDatos().setX(x);
            
            blueTooth.getDatos().enviarDatos().setY(y);
        }
        
    }

    @Override
    public void acelerometro(float x, float y, float z) {
        jugador.acelerometro(x, y, z);
    }
}
