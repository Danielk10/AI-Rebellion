package com.diamon.bluetooth.servicio;

import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothServerSocket;
import android.bluetooth.BluetoothSocket;

import java.io.IOException;

public class ConectarServidor implements Runnable {

    private final BluetoothServerSocket servidor;

    private BluetoothSocket cliente;

    private BluetoothAdapter adaptador;

    private boolean conectado;

    private Thread hilo;

    public ConectarServidor(BluetoothAdapter adaptador) {

        this.adaptador = adaptador;

        this.conectado = false;

        BluetoothServerSocket servidorTempoaral = null;

        try {
            servidorTempoaral =
                    this.adaptador.listenUsingRfcommWithServiceRecord(
                            "Seguro", ServicioBluetooth.SEGURO);

        } catch (IOException e) {

            e.printStackTrace();
        }

        this.servidor = servidorTempoaral;
    }

    public void iniciar() {

        this.hilo = new Thread(this);

        if (hilo != null) {

            hilo.start();
        }
    }

    @Override
    public void run() {

        adaptador.cancelDiscovery();

        BluetoothSocket cliente = null;

        while (true) {

            try {

                cliente = servidor.accept();

                this.conectado = true;

            } catch (IOException e) {

                e.printStackTrace();
            }

            try {

                this.cliente = cliente;

                if (servidor != null) {

                    servidor.close();
                }

            } catch (IOException e) {

                e.printStackTrace();
            }

            break;
        }
    }

    public void cancelarCliente() {

        try {

            this.conectado = false;

            if (cliente != null) {

                cliente.close();
            }

        } catch (IOException e) {

            e.printStackTrace();
        }
    }

    public BluetoothSocket getCliente() {

        return cliente;
    }

    public boolean isConectado() {

        return this.conectado;
    }
}
