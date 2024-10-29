package com.diamon.pantalla;

import com.diamon.graficos.Pantalla2D;
import com.diamon.graficos.Textura2D;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Textura;

public class PantallaJuego extends Pantalla2D {

    private Textura textura;

    public PantallaJuego(final Juego juego) {
        super(juego);

        textura =
                new Textura2D(
                        recurso.getTextura("creditos.png").getBipmap(),
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
    public void actualizar(float delta) {}

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
}
