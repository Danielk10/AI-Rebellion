package com.diamon.escenario;

import android.graphics.Color;
import com.diamon.actor.AndroidePatrullero;
import com.diamon.actor.Jugador;
import com.diamon.nucleo.Escena;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Pantalla;

public class EscenarioFabricaDeDrones extends Escena {

    
    float x =0;
    
    public EscenarioFabricaDeDrones(Pantalla pantalla, Jugador jugador) {
        super(pantalla, jugador);

        // configuracionesDeJuego.setLeerDatosAsset(true);

        actores.add(jugador);
        
        x= camara.getX();
    }

    @Override
    public void iniciar() {
        

        AndroidePatrullero androideA =
                new AndroidePatrullero(
                        this.pantalla,
                        recurso.getTextura("texturas/creditos.png"),
                        1200,
                        20,
                        64,
                        64);

        actores.add(androideA);
        
        
        
    }

    @Override
    public void actualizar(float delta) {
        
          camara.setX(x+=50*delta);
        
        for (int i = 0; i < actores.size(); i++) {

            actores.get(i).actualizar(delta);
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {

        for (int i = 0; i < actores.size(); i++) {

            actores.get(i).dibujar(pincel, delta);
        }

        pincel.dibujarTexto(
                "Numero de Actores: "
                        + actores.size()
                        + " "
                        + configuracionesDeJuego.isLeerDatosAsset(),
                250,
                250,
                Color.GREEN);
    }

    @Override
    public void guardarDatos() {

        configuracionesDeJuego.guardarConfiguraciones();
    }

    @Override
    public void liberarRecursos() {

        actores.clear();
    }

    @Override
    public boolean escenarioCompletado() {

        return false;
    }
}
