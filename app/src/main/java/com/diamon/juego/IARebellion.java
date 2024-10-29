package com.diamon.juego;


import androidx.appcompat.app.AppCompatActivity;

import com.diamon.nucleo.Juego;
import com.diamon.pantalla.PantallaJuego;

public class IARebellion extends Juego {
    public IARebellion(AppCompatActivity actividad) {
        super(actividad);

        setPantalla(new PantallaJuego(this));
    }
}
