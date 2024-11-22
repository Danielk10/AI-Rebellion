package com.diamon.bluetooth.iobluetooth;

import android.bluetooth.BluetoothSocket;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

public class DatoBluetooth implements Runnable {

    private final BluetoothSocket cliente;

    private final InputStream entrada;

    private final OutputStream salida;

    private Thread hilo;

    private String texto;

    private TransmisionDatos datosRecibido;

    private TransmisionDatos datosEviado;

    public DatoBluetooth(BluetoothSocket cliente) {

        this.cliente = cliente;

        this.texto = "0";

        datosRecibido = new DatosRecibido();

        datosEviado = new DatosEnviado();

        InputStream entradaTemporal = null;

        OutputStream salidaTemporal = null;

        try {

            entradaTemporal = this.cliente.getInputStream();

            salidaTemporal = this.cliente.getOutputStream();

        } catch (IOException e) {

        }

        this.entrada = entradaTemporal;

        this.salida = salidaTemporal;
    }

    public void iniciar() {

        hilo = new Thread(this);

        if (hilo != null) {

            hilo.start();
        }
    }

    public String leerDatos() {

        return texto.toString();
    }

    public void escribirDatos(String texto) {

        try {

            salida.write(texto.getBytes());

        } catch (IOException e) {

            e.printStackTrace();
        }
    }

    @Override
    public void run() {

        final byte[] bufer = new byte[1024];

        while (true) {

            try {

                int dato = entrada.read(bufer);

                this.texto = new String(bufer, 0, dato);

                datosRecibido.recibirDatos(this.entrada);

            } catch (IOException e) {

                e.printStackTrace();

                break;
            }
        }
    }

    public void cancelar() {

        try {

            cliente.close();

        } catch (IOException e) {

            e.printStackTrace();
        }
    }

    public TransmisionDatos resibirDatos() {
        return this.datosRecibido;
    }

    public TransmisionDatos enviarDatos() {

        try {

            datosEviado.enviarDatos(this.salida);

        } catch (Exception e) {

            e.printStackTrace();
        }

        return this.datosEviado;
    }
}
