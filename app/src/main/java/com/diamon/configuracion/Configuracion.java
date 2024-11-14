package com.diamon.configuracion;

import android.content.SharedPreferences;
import android.content.Context;

public class Configuracion {

    private SharedPreferences preferencias;

    private Context contexto;

    public Configuracion(Context contexto) {

        this.contexto = contexto;
    }

    public void setVolumen(float volumen) {

        preferencias = contexto.getSharedPreferences("Configuraciones", Context.MODE_PRIVATE);

        SharedPreferences.Editor editor = preferencias.edit();

        editor.putFloat("Volumen", volumen);

        editor.apply();
    }

    public float getVolumen() {

        preferencias = contexto.getSharedPreferences("Configuraciones", Context.MODE_PRIVATE);

        float volumen = preferencias.getFloat("Volumen", 0.5f);

        return volumen;
    }
}
