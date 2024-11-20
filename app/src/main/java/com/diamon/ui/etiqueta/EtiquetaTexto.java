package com.diamon.ui.etiqueta;

import android.graphics.Color;
import android.graphics.Paint;

import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Pantalla;
import com.diamon.ui.Elemento;

public class EtiquetaTexto extends Elemento {

    public EtiquetaTexto(Pantalla pantalla, float x, float y, String texto) {
        super(pantalla, null, x, y, texto);
    }
}
