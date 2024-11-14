package com.diamon.pantalla;

import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.graficos.Pantalla2D;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;

public class PantallaOpciones extends Pantalla2D {

    public PantallaOpciones(final Juego juego) {
        super(juego);
    }

    @Override
    public void mostrar() {}

    @Override
    public void resume() {}

    @Override
    public void colisiones() {}

    @Override
    public void actualizar(float delta) {}

    @Override
    public void dibujar(Graficos pincel, float delta) {}

    @Override
    public void reajustarPantalla(int ancho, int alto) {}

    @Override
    public void pausa() {}

    @Override
    public void ocultar() {}

    @Override
    public void liberarRecursos() {}

    @Override
    public void servicioBluetooth(ServicioBluetooth blueTooth) {}

    @Override
    public void teclaPresionada(int codigoDeTecla) {}

    @Override
    public void teclaLevantada(int codigoDeTecla) {}

    @Override
    public void toquePresionado(float x, float y, int puntero) {}

    @Override
    public void toqueLevantado(float x, float y, int puntero) {}

    @Override
    public void toqueDeslizando(float x, float y, int puntero) {}

    @Override
    public void acelerometro(float x, float y, float z) {}
}
