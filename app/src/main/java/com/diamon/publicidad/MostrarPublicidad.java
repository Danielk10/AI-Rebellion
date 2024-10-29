package com.diamon.publicidad;

import androidx.appcompat.app.AppCompatActivity;

public class MostrarPublicidad implements Publicidad {

    private AppCompatActivity actividad;

    public MostrarPublicidad(AppCompatActivity actividad) {

        this.actividad = actividad;
    }

    @Override
    public void mostrarInterstitial() {}

    @Override
    public void cargarInterstitial() {}

    @Override
    public void cargarBanner() {}

    @Override
    public void mostrarBanner() {}

    @Override
    public void ocultarBanner() {}
}
