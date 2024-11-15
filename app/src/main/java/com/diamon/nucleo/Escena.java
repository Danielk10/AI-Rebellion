package com.diamon.nucleo;

import com.diamon.actor.Jugador;
import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.dato.ConfiguracionesDeJuego;
import com.diamon.graficos.Camara2D;
import com.diamon.graficos.Pantalla2D;
import com.diamon.utilidad.Recurso;

import java.util.ArrayList;

public abstract class Escena {

    protected Pantalla pantalla;

    protected Jugador jugador;

    protected Juego juego;

    protected ArrayList<Actor> actores;

    protected Recurso recurso;

    protected ConfiguracionesDeJuego configuracionesDeJuego;

    protected Camara2D camara;

    protected ServicioBluetooth blueTooth;

    public Escena(Pantalla pantalla, Jugador jugador) {

        this.pantalla = pantalla;

        this.juego = ((Pantalla2D) pantalla).getJuego();

        this.jugador = jugador;

        this.actores = ((Pantalla2D) pantalla).getActores();

        this.recurso = ((Pantalla2D) pantalla).getJuego().getRecurso();

        this.configuracionesDeJuego =
                ((Pantalla2D) pantalla).getJuego().getConfiguracionesDeJuego();

        this.camara = ((Pantalla2D) pantalla).getCamara();

        this.camara.setPosicion(Juego.ANCHO_PANTALLA / 2, Juego.ALTO_PANTALLA / 2);

        this.blueTooth = ((Pantalla2D) pantalla).getBluetooth();
    }

    public abstract void iniciar();

    public abstract boolean escenarioCompletado();

    public abstract void actualizar(float delta);

    public abstract void dibujar(Graficos pincel, float delta);

    public abstract void guardarDatos();

    public abstract void liberarRecursos();
}
