package com.diamon.escenario;

import android.graphics.Color;
import com.diamon.actor.ambiente.Fondo;
import com.diamon.actor.enemigo.AndroidePatrullero;
import com.diamon.actor.Jugador;
import com.diamon.actor.enemigo.Blaze;
import com.diamon.actor.enemigo.Echo;
import com.diamon.actor.enemigo.Gaia;
import com.diamon.actor.enemigo.Lux;
import com.diamon.actor.enemigo.Nebula;
import com.diamon.actor.enemigo.Nyx;
import com.diamon.actor.enemigo.Orion;
import com.diamon.actor.enemigo.Zero;
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
                        recurso.getTextura("texturas/escena1.jpg"),
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

        Blaze dron =
                new Blaze(
                        this.pantalla,
                        recurso.getTextura("texturas/creditos.png"),
                        1000,
                        400,
                        64,
                        64);

        actores.add(dron);

        // En tu PantallaJuego.mostrar()
        camara.setLimitesMundo(5000, 10000);
        
        jugador.configurarLimitesDelMundo(5000, 10000);
    }

    @Override
    public void actualizar(float delta) {

     

        camara.seguirSuave(jugador.getX(), jugador.getY(), 0.5f);

        if (blueTooth.getDatos() != null) {

            androideA.setX(blueTooth.getDatos().resibirDatos().getX());
            androideA.setY(blueTooth.getDatos().resibirDatos().getY());
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
