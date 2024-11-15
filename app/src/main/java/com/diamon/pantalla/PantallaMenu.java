package com.diamon.pantalla;

import android.graphics.Color;
import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.graficos.Pantalla2D;
import com.diamon.graficos.Textura2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.ui.Boton;
import com.diamon.ui.EtiquetaTexto;
import com.diamon.utilidad.Rectangulo;

public class PantallaMenu extends Pantalla2D {

    private Boton botonIniciar;

    private Boton botonOpciones;

    private Boton botonSalir;
    
    private EtiquetaTexto titulo;

    public PantallaMenu(final Juego juego) {
        super(juego);

        Textura2D texturaBoton =
                new Textura2D(recurso.getTextura("texturas/creditos.png"), 100, 50);

        botonIniciar = new Boton(this, texturaBoton, 200, 200, "Iniciar Juego");

        botonOpciones = new Boton(this, texturaBoton, 200, 300, "Opciones");

        botonSalir = new Boton(this, texturaBoton, 200, 400, "Salir");
        
        titulo = new EtiquetaTexto(this,200,50,"AI Rebellion");
        
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
    public void toquePresionado(float x, float y, int puntero) {

        if (botonIniciar.getRectangulo().intersecion(new Rectangulo(x,y,32,32))) {
            
            iniciarJuego();
            
        } else if (botonOpciones.getRectangulo().intersecion(new Rectangulo(x,y,32,32))) {
            
            mostrarOpciones();
            
        } else if (botonSalir.getRectangulo().intersecion(new Rectangulo(x,y,32,32))) {
            
            salirJuego();
        }
    }

    @Override
    public void toqueLevantado(float x, float y, int puntero) {}

    @Override
    public void toqueDeslizando(float x, float y, int puntero) {}

    @Override
    public void acelerometro(float x, float y, float z) {}

    }
