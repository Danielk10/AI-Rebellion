package com.diamon.pantalla;

import android.Manifest;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.content.Intent;
import android.content.IntentFilter;
import android.content.pm.PackageManager;

import android.graphics.Color;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import com.diamon.bluetooth.gedtion.AdministradorBluetooth;
import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.graficos.Pantalla2D;
import com.diamon.graficos.Textura2D;
import com.diamon.iarebellion.MainActivity;
import com.diamon.nucleo.Actor;
import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Juego;
import com.diamon.ui.boton.Boton;
import com.diamon.ui.etiqueta.EtiquetaTexto;
import com.diamon.utilidad.Rectangulo;

import java.util.ArrayList;

public class PantallaOpciones extends Pantalla2D {

    private ArrayList<BluetoothDevice> dispositivos;

    private Boton cliente;

    private Boton servidor;

    private Boton cancelar;

    private EtiquetaTexto texto;

    private EtiquetaTexto esperando;

    private Boton aceptar;

    private Textura2D texturaBoton;

    private int registro;

    private boolean busqueda;

    private BluetoothDevice dispositivo;

    private int tipo;

    private boolean toque;

    public static final int REQUEST_ENABLE_DISCOVERABLE = 1;

    public PantallaOpciones(final Juego juego) {
        super(juego);

        registro = 0;

        busqueda = true;

        dispositivos = AdministradorBluetooth.dispositivos;

        texturaBoton = new Textura2D(recurso.getTextura("texturas/creditos.png"), 100, 50);

        servidor = new Boton(this, texturaBoton, 200, 200, "Servidor");

        cliente = new Boton(this, texturaBoton, 200, 300, "Cliente");

        cancelar = new Boton(this, texturaBoton, 200, 400, "Cancelar");

        actores.add(servidor);

        actores.add(cliente);

        actores.add(cancelar);

        texto = new EtiquetaTexto(this, 500, 40, "");

        esperando = new EtiquetaTexto(this, 500, 600, "Esperando...");

        aceptar = new Boton(this, texturaBoton, 500, 50, "Aceptar");
    }

    private void blueTooth() {

        if (dispositivos != null) {

            if (dispositivos.size() > 0) {

                if (busqueda) {

                    for (int i = 0; i < dispositivos.size(); i++) {

                        if (dispositivos.get(i) != null) {

                            texto.setTexto(dispositivos.get(i).getName());

                            dispositivo =
                                    blueTooth
                                            .getAdaptador()
                                            .getRemoteDevice(dispositivos.get(i).getAddress());
                        }
                    }

                    busqueda = false;
                }
            }
        }
    }

    @Override
    public void mostrar() {}

    @Override
    public void resume() {}

    @Override
    public void colisiones() {

        for (int i = 0; i < actores.size(); i++) {

            Actor actor = actores.get(i);

            if (actor.isRemover()) {

                actores.remove(i);
            }
        }
    }

    @Override
    public void actualizar(float delta) {

        for (Actor actor : actores) {

            actor.actualizar(delta);
        }

        blueTooth();

        if (blueTooth.isConectado()) {

            juego.setPantalla(new PantallaJuego(juego));
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

    @Override
    public void teclaPresionada(int codigoDeTecla) {}

    @Override
    public void teclaLevantada(int codigoDeTecla) {}

    @Override
    public void toquePresionado(float x, float y, int puntero) {}

    @Override
    public void toqueLevantado(float x, float y, int puntero) {

        if (servidor.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

            tipo = 1;

            cliente.setPosicion(-100, -100);

            cliente.remover();

            actores.add(texto);

            actores.add(aceptar);

            esperando.setTexto("Esperando...Servidor");

            actores.add(esperando);

            AdministradorBluetooth.activarBluetooth(juego.getActividad(), blueTooth.getAdaptador());

            AdministradorBluetooth.buscarDispositivo(
                    juego.getActividad(), blueTooth.getAdaptador());

            AdministradorBluetooth.activarVisibilidad(
                    juego.getActividad(), blueTooth.getAdaptador(), 300);

            servidor.setPosicion(-100, -100);

            servidor.remover();

        } else if (cliente.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

            tipo = 2;

            servidor.setPosicion(-100, -100);

            servidor.remover();

            actores.add(texto);

            actores.add(aceptar);

            esperando.setTexto("Esperando...Cliente");

            actores.add(esperando);

            AdministradorBluetooth.activarBluetooth(juego.getActividad(), blueTooth.getAdaptador());

            AdministradorBluetooth.buscarDispositivo(
                    juego.getActividad(), blueTooth.getAdaptador());

            AdministradorBluetooth.activarVisibilidad(
                    juego.getActividad(), blueTooth.getAdaptador(), 300);

            cliente.setPosicion(-100, -100);

            cliente.remover();

        } else if (cancelar.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

        }

        if (aceptar.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

            if (tipo == 1) {

                if (dispositivo != null) {

                    AdministradorBluetooth.vincularDispoditivo(dispositivo);

                    if (blueTooth.getAdaptador() != null) {

                        if (blueTooth.getAdaptador().isEnabled()) {

                            blueTooth.setTipo(ServicioBluetooth.SERVIDOR);

                            blueTooth.setConectarServidor();
                        }
                    }
                }
            }

            if (tipo == 2) {

                if (dispositivo != null) {

                    AdministradorBluetooth.vincularDispoditivo(dispositivo);

                    dispositivo =
                            blueTooth.getAdaptador().getRemoteDevice(dispositivo.getAddress());

                    blueTooth.setTipo(ServicioBluetooth.CLIENTE);

                    blueTooth.setConectarCliente(dispositivo);
                }
            }
        }
    }

    @Override
    public void toqueDeslizando(float x, float y, int puntero) {}

    @Override
    public void acelerometro(float x, float y, float z) {}
}
