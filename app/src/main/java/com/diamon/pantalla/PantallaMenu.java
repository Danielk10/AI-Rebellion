package com.diamon.pantalla;

import android.graphics.Color;
import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.graficos.Pantalla2D;
import com.diamon.graficos.Textura2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.ui.boton.Boton;
import com.diamon.ui.etiqueta.EtiquetaTexto;
import com.diamon.ui.lienzo.Imagen;
import com.diamon.utilidad.Rectangulo;

public class PantallaMenu extends Pantalla2D {

    private Boton botonIniciar;

    private Boton botonOpciones;

    private Boton botonSalir;

    private EtiquetaTexto titulo;

    private Imagen fondo;

    public PantallaMenu(final Juego juego) {
        super(juego);

        Textura2D texturaBoton =
                new Textura2D(recurso.getTextura("texturas/creditos.png"), 100, 50);

        fondo =
                new Imagen(
                        this,
                        recurso.getTextura("texturas/fondo.jpg"),
                        0,
                        0,
                        Juego.ANCHO_PANTALLA,
                        Juego.ALTO_PANTALLA,
                        "");

        titulo =
                new EtiquetaTexto(
                        this,
                        Juego.ANCHO_PANTALLA * 0.5f - 100,
                        Juego.ALTO_PANTALLA * 0.1f,
                        "AI Rebellion");
        titulo.setColor(Color.GREEN);
        titulo.setTamanoTexto(30);

        botonIniciar =
                new Boton(
                        this,
                        texturaBoton,
                        Juego.ANCHO_PANTALLA * 0.5f - 100,
                        Juego.ALTO_PANTALLA * 0.3f,
                        200,
                        50,
                        "Iniciar Juego");
        botonIniciar.setColor(Color.RED);

        botonOpciones =
                new Boton(
                        this,
                        texturaBoton,
                        Juego.ANCHO_PANTALLA * 0.5f - 100,
                        Juego.ALTO_PANTALLA * 0.4f,
                        200,
                        50,
                        "Opciones");
        botonOpciones.setColor(Color.RED);

        botonSalir =
                new Boton(
                        this,
                        texturaBoton,
                        Juego.ANCHO_PANTALLA * 0.5f - 100,
                        Juego.ALTO_PANTALLA * 0.5f,
                        200,
                        50,
                        "Salir");
        botonSalir.setColor(Color.RED);

        actores.add(fondo);

        actores.add(titulo);

        actores.add(botonIniciar);

        actores.add(botonOpciones);

        actores.add(botonSalir);
    }

    @Override
    public void mostrar() {}

    @Override
    public void resume() {}

    @Override
    public void colisiones() {}

    @Override
    public void actualizar(float delta) {

        for (Actor actor : actores) {

            actor.actualizar(delta);
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {

        for (Actor actor : actores) {

            actor.dibujar(pincel, delta);
        }
    }

    @Override
    public void reajustarPantalla(int ancho, int alto) {}

    @Override
    public void pausa() {}

    @Override
    public void ocultar() {}

    @Override
    public void liberarRecursos() {

        actores.clear();
    }

    private void iniciarJuego() {
        juego.setPantalla(new PantallaJuego(juego)); // Cambia a la pantalla principal del juego
    }

    private void mostrarOpciones() {
        juego.setPantalla(new PantallaOpciones(juego));
    }

    private void salirJuego() {}

    @Override
    public void teclaPresionada(int codigoDeTecla) {}

    @Override
    public void teclaLevantada(int codigoDeTecla) {}

    @Override
    public void toquePresionado(float x, float y, int puntero) {}

    @Override
    public void toqueLevantado(float x, float y, int puntero) {

        if (puntero == 0) {

            if (botonIniciar.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

                iniciarJuego();

            } else if (botonOpciones.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

                mostrarOpciones();

            } else if (botonSalir.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

                salirJuego();
            }
        }
    }

    @Override
    public void toqueDeslizando(float x, float y, int puntero) {}

    @Override
    public void acelerometro(float x, float y, float z) {}
}
