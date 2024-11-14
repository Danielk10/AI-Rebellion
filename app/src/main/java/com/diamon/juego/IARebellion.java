package com.diamon.juego;


import androidx.appcompat.app.AppCompatActivity;

import com.diamon.nucleo.Juego;
import com.diamon.pantalla.PantallaPresentacion;

public class IARebellion extends Juego {
    public IARebellion(AppCompatActivity actividad) {
        super(actividad);

        setPantalla(new PantallaPresentacion(this));
    }
}
