package com.diamon.graficos;

import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.configuracion.Configuracion;
import com.diamon.dato.ConfiguracionesDeJuego;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.utilidad.Recurso;

import java.util.ArrayList;

public abstract class Pantalla2D implements Pantalla {

    protected final Juego juego;

    protected ArrayList<Actor> actores;

    protected ConfiguracionesDeJuego configuracionesDeJuego;

    protected Configuracion configuraciones;

    protected Recurso recurso;

    protected ServicioBluetooth bluetooth;

    protected Camara2D camara;

    public Pantalla2D(final Juego juego) {

        this.juego = juego;

        configuracionesDeJuego = juego.getConfiguracionesDeJuego();

        configuraciones = juego.getConfiguraciones();

        actores = new ArrayList<Actor>();

        recurso = juego.getRecurso();

        bluetooth = juego.getBluetooth();

        camara = juego.getCamara();
    }

    public Juego getJuego() {
        return juego;
    }

    public ArrayList<Actor> getActores() {
        return actores;
    }

    public ServicioBluetooth getBluetooth() {
        return this.bluetooth;
    }

    public Camara2D getCamara() {
        return this.camara;
    }
}
