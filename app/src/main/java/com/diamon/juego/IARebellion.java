package com.diamon.juego;

import androidx.appcompat.app.AppCompatActivity;

import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.nucleo.Juego;
import com.diamon.pantalla.PantallaPresentacion;

public class IARebellion extends Juego {
    public IARebellion(AppCompatActivity actividad, ServicioBluetooth blueTooth) {
        super(actividad, blueTooth);

        setPantalla(new PantallaPresentacion(this));
    }
}
