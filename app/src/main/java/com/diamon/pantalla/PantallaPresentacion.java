package com.diamon.pantalla;

import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.graficos.Textura2D;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.graficos.Pantalla2D;

public class PantallaPresentacion extends Pantalla2D {

    private Textura2D textura;

    private float tiempo;

    public PantallaPresentacion(final Juego juego) {
        super(juego);

        textura =
                new Textura2D(
                        recurso.getTextura("texturas/diamondBlack.png"),
                        Juego.ANCHO_PANTALLA,
                        Juego.ALTO_PANTALLA);
    }

    @Override
    public void mostrar() {}

    @Override
    public void resume() {}

    @Override
    public void colisiones() {}

    @Override
    public void actualizar(float delta) {

        tiempo += delta;

        if (tiempo / 10 > 1) {

            juego.setPantalla(new PantallaMenu(juego));

            tiempo = 0;
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {

        pincel.dibujarTextura(textura, 0, 0);
    }

    @Override
    public void reajustarPantalla(int ancho, int alto) {}

    @Override
    public void pausa() {}

    @Override
    public void ocultar() {}

    @Override
    public void liberarRecursos() {}

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

    @Override
    public void servicioBluetooth(ServicioBluetooth blueTooth) {}
}
