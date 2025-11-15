package com.diamon.actor;

import android.view.KeyEvent;

import com.diamon.graficos.Actor2D;
import com.diamon.graficos.Camara2D;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.nucleo.Pantalla;
import com.diamon.nucleo.Textura;

public class Jugador extends Actor2D {

    private float deltaXTactil, deltaYTactil;
    private float coordenadaPantallaX, coordenadaPantallaY;
    private float direccionDezplazamientoX1, direccionDezplazamientoY1;
    private float direccionDezplazamientoX2, direccionDezplazamientoY2;
    private float x1 = 0;
    private float y1 = 0;
    private float velocidadX;
    private float velocidadY;
    private static final float VELOCIDAD_JUGADOR = 5;
    private boolean arriba, abajo, izquierda, derecha, disparar;
    private float tiemoDisparo;
    private float tiemoDisparoL;
    private float tiemoDisparoB;
    private float tiempoParpadeo;
    protected int frames = 0;
    private boolean dezplazamientoInicial;
    private boolean parpadeo;
    private int ladoJugador;
    private float vida;
    private boolean inmune;
    private boolean poder;
    private int tipoPoder;
    private float velocidad;
    private static final int LADO_DERECHO = 1;
    private static final int LADO_IZQUIERDO = 2;
    private float igual1;
    private float igual2;
    private int ciclo;
    private boolean dezplazamiento;
    private int lado;
    private int dedos;
    private boolean deltaToque;

    // MEJORA: Variables para los límites basados en cámara
    private static final int MARGEN_SEGURIDAD = 32;
    private float limiteMundoIzquierdo;
    private float limiteMundoDerecho;
    private float limiteMundoArriba;
    private float limiteMundoAbajo;

    public Jugador(Pantalla pantalla, Textura textura, float x, float y, float ancho, float alto) {
        super(pantalla, textura, x, y, ancho, alto);
        inicializarVariables();
    }

    public Jugador(Pantalla pantalla, Textura textura, float x, float y) {
        super(pantalla, textura, x, y);
        inicializarVariables();
    }

    public Jugador(
            Pantalla pantalla,
            Textura[] texturas,
            float x,
            float y,
            float ancho,
            float alto,
            float tiempoAnimacion) {
        super(pantalla, texturas, x, y, ancho, alto, tiempoAnimacion);
        inicializarVariables();
    }

    /**
     * MEJORA: Inicializar variables comunes
     */
    private void inicializarVariables() {
        velocidadX = 0;
        velocidadY = 0;
        deltaXTactil = deltaYTactil = 0;
        dezplazamientoInicial = true;
        arriba = abajo = izquierda = derecha = disparar = false;
        parpadeo = false;
        velocidad = VELOCIDAD_JUGADOR;
        ladoJugador = Jugador.LADO_DERECHO;
        poder = false;
        vida = 3;
        inmune = false;
        tipoPoder = 0;
        deltaToque = false;
        dedos = -1;
    }

    /**
     * MEJORA: Configurar los límites del mundo basado en la cámara
     * @param mundoAncho ancho total del mundo del juego
     * @param mundoAlto alto total del mundo del juego
     */
    public void configurarLimitesDelMundo(float mundoAncho, float mundoAlto) {
        // Los límites del jugador son el mundo menos el margen de seguridad
        limiteMundoIzquierdo = MARGEN_SEGURIDAD;
        limiteMundoArriba = MARGEN_SEGURIDAD;
        limiteMundoDerecho = mundoAncho - MARGEN_SEGURIDAD - ancho;
        limiteMundoAbajo = mundoAlto - MARGEN_SEGURIDAD - alto;
    }

    @Override
    public void obtenerActores() {
        // TODO: Implement this method
    }

    public float getVelocidad() {
        return velocidad;
    }

    public void setVelocidad(float velocidad) {
        this.velocidad = velocidad;
    }

    @Override
    public void actualizar(float delta) {
        if (vida > 0) {
            super.actualizar(delta);

            // Actualizar posición basada en velocidad
            x += velocidadX / Juego.DELTA_A_PIXEL * delta;
            y += velocidadY / Juego.DELTA_A_PIXEL * delta;

            // MEJORA: Aplicar límites basados en el mundo de la cámara
            aplicarLimitesDelMundo();

            // Actualizar temporizadores de disparo
            tiemoDisparo += delta;
            tiemoDisparoB += delta;
            tiemoDisparoL += delta;

            // Manejar disparo
            if (disparar) {
                if (!poder) {
                    disparar();
                }

                if (poder) {
                    disparoEspecial();
                }
            }

            // Manejar parpadeo de inmunidad
            if (tiempoParpadeo % 2 == 0) {
                parpadeo = true;
            } else {
                parpadeo = false;
            }

            if (tiempoParpadeo <= 0) {
                tiempoParpadeo = 0;
                inmune = true;
            } else {
                inmune = false;
            }

            tiempoParpadeo -= 1;

            // Manejar desplazamiento
            if (dezplazamiento) {
                if (ciclo >= 3) {
                    ciclo = 0;
                }

                if (ciclo == 0) {
                    igual1 = x;
                    direccionDezplazamientoX1 = coordenadaPantallaX;
                }

                if (ciclo == 1) {
                    igual2 = x;
                    direccionDezplazamientoX2 = coordenadaPantallaX;
                }

                if ((int) igual1 == (int) igual2) {
                    dezplazamiento = false;
                } else {
                    dezplazamiento = true;
                }

                if (ciclo == 2) {
                    if ((int) direccionDezplazamientoX1 > (int) direccionDezplazamientoX2) {
                        lado = Jugador.LADO_IZQUIERDO;
                    }

                    if ((int) direccionDezplazamientoX1 < (int) direccionDezplazamientoX2) {
                        lado = Jugador.LADO_DERECHO;
                    }
                }

                ciclo++;
            }
        }
    }

    /**
     * MEJORA: Aplicar límites del mundo al jugador
     * Asegura que el jugador no salga de los límites definidos por la cámara
     */
    private void aplicarLimitesDelMundo() {
        // Limitar X
        if (x < limiteMundoIzquierdo) {
            x = limiteMundoIzquierdo;
        } else if (x > limiteMundoDerecho) {
            x = limiteMundoDerecho;
        }

        // Limitar Y
        if (y < limiteMundoArriba) {
            y = limiteMundoArriba;
        } else if (y > limiteMundoAbajo) {
            y = limiteMundoAbajo;
        }
    }

    @Override
    public void dibujar(Graficos pincel, float delta) {
        if (vida > 0) {
            if (!parpadeo) {
                if (disparar) {
                    if (ladoJugador == Jugador.LADO_DERECHO) {
                        super.dibujar(pincel, delta);
                    }

                    if (ladoJugador == Jugador.LADO_IZQUIERDO) {
                        super.dibujar(pincel, delta);
                    }
                } else {
                    if (ladoJugador == Jugador.LADO_DERECHO) {
                        super.dibujar(pincel, delta);
                    }

                    if (ladoJugador == Jugador.LADO_IZQUIERDO) {
                        super.dibujar(pincel, delta);
                    }
                }
            }
        }
    }

    @Override
    public void colision(Actor actor) {
        if (!inmune) {
            if (actor instanceof Enemigo) {
                poder = false;
                velocidad = Jugador.VELOCIDAD_JUGADOR;
                tiempoParpadeo = 100;
                vida--;
            }
        }
    }

    public float getVida() {
        return vida;
    }

    protected void actualizarVelocidad() {
        velocidadX = 0;
        velocidadY = 0;

        if (abajo) {
            velocidadY = velocidad;
        }

        if (arriba) {
            velocidadY = -velocidad;
        }

        if (izquierda) {
            velocidadX = -velocidad;
        }

        if (derecha) {
            velocidadX = velocidad;
        }
    }

    public void disparar() {}

    public void disparoEspecial() {}

    public void teclaPresionada(int codigoDeTecla) {
        switch (codigoDeTecla) {
            case KeyEvent.KEYCODE_0:
                izquierda = true;
                dezplazamientoInicial = false;
                ladoJugador = Jugador.LADO_DERECHO;
                break;
            case KeyEvent.KEYCODE_1:
                derecha = true;
                dezplazamientoInicial = false;
                ladoJugador = Jugador.LADO_IZQUIERDO;
                break;
            case KeyEvent.KEYCODE_2:
                arriba = true;
                dezplazamientoInicial = false;
                break;
            case KeyEvent.KEYCODE_3:
                abajo = true;
                dezplazamientoInicial = false;
                break;
            case KeyEvent.KEYCODE_4:
                disparar = true;
                break;
            case KeyEvent.KEYCODE_5:
                break;
            default:
                break;
        }

        actualizarVelocidad();
    }

    public void teclaLevantada(int codigoDeTecla) {
        switch (codigoDeTecla) {
            case KeyEvent.KEYCODE_0:
                izquierda = false;
                dezplazamientoInicial = false;
                break;
            case KeyEvent.KEYCODE_1:
                derecha = false;
                dezplazamientoInicial = false;
                break;
            case KeyEvent.KEYCODE_2:
                arriba = false;
                dezplazamientoInicial = false;
                break;
            case KeyEvent.KEYCODE_3:
                abajo = false;
                dezplazamientoInicial = false;
                break;
            case KeyEvent.KEYCODE_4:
                disparar = false;
                break;
            default:
                break;
        }

        actualizarVelocidad();
    }

    public void toquePresionado(float xPantalla, float yPantalla, int puntero) {
        if (dedos == 0) {
            dezplazamiento = false;
            disparar = true;
            x1 = this.x;
            y1 = this.y;
            deltaXTactil = (int) xPantalla - (int) x1;
            deltaYTactil = (int) yPantalla - (int) y1;

            // Aplicar límites al presionar
            if (x1 <= limiteMundoIzquierdo) x1 = limiteMundoIzquierdo;
            if (y1 <= limiteMundoArriba) y1 = limiteMundoArriba;
            if (x1 >= limiteMundoDerecho) x1 = limiteMundoDerecho;
            if (y1 >= limiteMundoAbajo) y1 = limiteMundoAbajo;

        } else if (puntero == 0) {
            dezplazamiento = false;
            disparar = true;
            x1 = this.x;
            y1 = this.y;
            deltaXTactil = (int) xPantalla - (int) x1;
            deltaYTactil = (int) yPantalla - (int) y1;

            // Aplicar límites al presionar
            if (x1 <= limiteMundoIzquierdo) x1 = limiteMundoIzquierdo;
            if (y1 <= limiteMundoArriba) y1 = limiteMundoArriba;
            if (x1 >= limiteMundoDerecho) x1 = limiteMundoDerecho;
            if (y1 >= limiteMundoAbajo) y1 = limiteMundoAbajo;
        }

        dedos++;
    }

    public void toqueLevantado(float x, float y, int puntero) {
        if (dedos == 0) {
            dezplazamiento = false;
            disparar = false;
        }

        deltaToque = true;
        dedos--;

        if (dedos == -1) {
            disparar = false;
        }
    }

    public void toqueDeslizando(float xPantalla, float yPantalla, int puntero) {
        if (dedos == 0) {
            if (deltaToque) {
                x1 = this.x;
                y1 = this.y;
                deltaXTactil = xPantalla - x1;
                deltaYTactil = yPantalla - y1;
                deltaToque = false;
            }

            if (xPantalla > this.x) {
                if (lado == Jugador.LADO_DERECHO) {
                    ladoJugador = Jugador.LADO_DERECHO;
                }
            }

            if (xPantalla < this.x) {
                if (lado == Jugador.LADO_IZQUIERDO) {
                    ladoJugador = Jugador.LADO_IZQUIERDO;
                }
            }

            dezplazamiento = true;
            this.coordenadaPantallaX = xPantalla;
            this.coordenadaPantallaY = yPantalla;
            x1 = (int) xPantalla - deltaXTactil;
            y1 = (int) yPantalla - deltaYTactil;

            // MEJORA: Aplicar límites del mundo al deslizar
            if (x1 <= limiteMundoIzquierdo) x1 = limiteMundoIzquierdo;
            if (y1 <= limiteMundoArriba) y1 = limiteMundoArriba;
            if (x1 >= limiteMundoDerecho) x1 = limiteMundoDerecho;
            if (y1 >= limiteMundoAbajo) y1 = limiteMundoAbajo;

            this.x = x1;
            this.y = y1;
            dezplazamientoInicial = false;

        } else if (puntero == 0) {
            if (xPantalla > this.x) {
                if (lado == Jugador.LADO_DERECHO) {
                    ladoJugador = Jugador.LADO_DERECHO;
                }
            }

            if (xPantalla < this.x) {
                if (lado == Jugador.LADO_IZQUIERDO) {
                    ladoJugador = Jugador.LADO_IZQUIERDO;
                }
            }

            dezplazamiento = true;
            this.coordenadaPantallaX = xPantalla;
            this.coordenadaPantallaY = yPantalla;
            x1 = (int) xPantalla - deltaXTactil;
            y1 = (int) yPantalla - deltaYTactil;

            // MEJORA: Aplicar límites del mundo al deslizar
            if (x1 <= limiteMundoIzquierdo) x1 = limiteMundoIzquierdo;
            if (y1 <= limiteMundoArriba) y1 = limiteMundoArriba;
            if (x1 >= limiteMundoDerecho) x1 = limiteMundoDerecho;
            if (y1 >= limiteMundoAbajo) y1 = limiteMundoAbajo;

            this.x = x1;
            this.y = y1;
            dezplazamientoInicial = false;
        }
    }

    public void acelerometro(float x, float y, float z) {}

    public void setVida(float vida) {
        this.vida = vida;
    }

    public void reducirVelocidad(float velocidadX, float velecidadY) {}

    public void desactivarArmas(float tiempo) {}
}
