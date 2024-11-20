package com.diamon.escenario;

import android.graphics.Color;
import com.diamon.actor.ambiente.Fondo;
import com.diamon.actor.enemigo.AndroidePatrullero;
import com.diamon.actor.Jugador;
import com.diamon.graficos.Textura2D;
import com.diamon.nucleo.Escena;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;

public class EscenarioFabricaDeDrones extends Escena {

    AndroidePatrullero androideA;

    public EscenarioFabricaDeDrones(Pantalla pantalla, Jugador jugador) {
        super(pantalla, jugador);
    }

    @Override
    public void iniciar() {

        Fondo fondo =
                new Fondo(
                        pantalla,
                        recurso.getTextura("texturas/fondo.png"),
                        0,
                        0,
                        Juego.ANCHO_PANTALLA,
                        Juego.ALTO_PANTALLA);

        actores.add(fondo);

        actores.add(jugador);

        androideA =
                new AndroidePatrullero(
                        this.pantalla,
                        recurso.getTextura("texturas/creditos.png"),
                        900,
                        20,
                        64,
                        64);

        actores.add(androideA);
    }

    @Override
    public void actualizar(float delta) {

        if (blueTooth.getDatos() != null) {

            androideA.setX(Long.parseLong(blueTooth.getDatos().leerDatos()));
        }
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

        if (blueTooth.getDatos() != null) {

            pincel.dibujarTexto(
                    "x" + androideA.getX() + " y " + androideA.getY(), 300, 200, Color.GREEN);

            pincel.dibujarTexto(
                    "x" + Long.parseLong(blueTooth.getDatos().leerDatos()), 400, 300, Color.GREEN);
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

    @Override
    public boolean escenarioCompletado() {

        return false;
    }
}
