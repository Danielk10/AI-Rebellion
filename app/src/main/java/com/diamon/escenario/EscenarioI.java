package com.diamon.escenario;

import com.diamon.actor.Jugador;
import com.diamon.nucleo.Escena;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Pantalla;

public class EscenarioI extends Escena {

    public EscenarioI(Pantalla pantalla, Jugador jugador) {
        super(pantalla, jugador);
    }

    @Override
    protected void iniciar() {
        
        
        actores.add(jugador);
    }

    @Override
    public void actualizar(float delta) {

        for (int i = 0; i < actores.size(); i++) {

            actores.get(i).actualizar(delta);
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {

        for (int i = 0; i < actores.size(); i++) {

            actores.get(i).dibujar(pincel, delta);
        }
    }

    @Override
    public void guardarDatos() {

        configuracionesDeJuego.guardarConfiguraciones();
    }

    @Override
    public void liberarRecursos() {

        actores.clear();
    }
}
