package com.diamon.pantalla;

import android.Manifest;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.content.Intent;
import android.content.IntentFilter;
import android.content.pm.PackageManager;

import android.graphics.Color;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

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

    private MainActivity actividad;

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

    public PantallaOpciones(final Juego juego) {
        super(juego);

        registro = 0;

        busqueda = true;

        dispositivos = ((MainActivity) juego.getActividad()).getDispositivos();

        actividad = ((MainActivity) juego.getActividad());

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

        if (blueTooth.getAdaptador().isEnabled()) {

            if (blueTooth.getAdaptador() != null) {

                dispositivo = blueTooth.getAdaptador().getBondedDevices().iterator().next();

                if (dispositivo != null) {

                    texto.setTexto(dispositivo.getName());
                }
            }
        }
    }

    public void activarBluetooth() {

        if ((ContextCompat.checkSelfPermission(
                        juego.getActividad().getApplicationContext(),
                        Manifest.permission.BLUETOOTH_CONNECT)
                != PackageManager.PERMISSION_GRANTED)) {

            ActivityCompat.requestPermissions(
                    juego.getActividad(),
                    new String[] {Manifest.permission.BLUETOOTH_CONNECT},
                    MainActivity.REQUEST_CODE_BLUETOOTH_CONNECT);
        }

        if (blueTooth.getAdaptador() != null) {

            if (!blueTooth.getAdaptador().isEnabled()) {

                Intent intencion = new Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE);

                juego.getActividad().startActivityForResult(intencion, ServicioBluetooth.ACTIVAR);
            }
        }
    }

    private void buscarDispositivo() {

        if (registro < 1) {

            if (blueTooth.getAdaptador().isEnabled()) {

                try {
                    if (blueTooth.getAdaptador().startDiscovery()) {

                        IntentFilter intecionFiltrada =
                                new IntentFilter(BluetoothDevice.ACTION_FOUND);

                        juego.getActividad()
                                .registerReceiver(actividad.getReservado(), intecionFiltrada);
                    }
                } catch (Exception e) {

                    e.printStackTrace();
                }

                registro++;
            }
        }
    }

    private void blueTooth() {

        if (dispositivos.size() > 0) {

            if (busqueda) {

                if (dispositivos != null) {

                    for (int i = 0; i < dispositivos.size(); i++) {

                        // if (dispositivos.get(i) != null) {

                        // texto.setTexto(dispositivos.get(i).getName());

                        // actores.add(texto);

                        //  actores.add(aceptar);

                        /* dispositivo =
                        blueTooth
                                .getAdaptador()
                                .getRemoteDevice(dispositivos.get(i).getAddress());*/
                        // }
                    }
                }

                busqueda = false;
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

            activarBluetooth();

            buscarDispositivo();

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

            activarBluetooth();

            buscarDispositivo();

            cliente.setPosicion(-100, -100);

            cliente.remover();

        } else if (cancelar.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

        }

        if (aceptar.getRectangulo().intersecion(new Rectangulo(x, y, 32, 32))) {

            if (tipo == 1) {

                if (blueTooth.getAdaptador() != null) {

                    if (blueTooth.getAdaptador().isEnabled()) {

                        blueTooth.setTipo(ServicioBluetooth.SERVIDOR);

                        blueTooth.setConectarServidor();
                    }
                }
            }

            if (tipo == 2) {

                if (dispositivo != null) {

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
