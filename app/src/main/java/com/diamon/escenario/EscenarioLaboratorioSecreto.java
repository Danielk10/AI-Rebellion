package com.diamon.escenario;

import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Escena;
import com.diamon.nucleo.Pantalla;
import com.diamon.actor.Jugador;

public class EscenarioLaboratorioSecreto extends Escena {

    public EscenarioLaboratorioSecreto(Pantalla pantalla, Jugador jugador) {
        super(pantalla, jugador);

        // configuracionesDeJuego.setLeerDatosAsset(true);

        actores.add(jugador);
    }

    @Override
    public void iniciar() {}

    @Override
    public boolean escenarioCompletado() {

        return false;
    }

    @Override
    public void actualizar(float delta) {}

    @Override
    public void dibujar(Graficos pincel, float delta) {}

    @Override
    public void guardarDatos() {}

    @Override
    public void liberarRecursos() {}
}
