package com.diamon.bluetooth.servicio;

import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothSocket;

import java.io.IOException;

public class ConectarCliente implements Runnable {

    private final BluetoothSocket cliente;

    private BluetoothAdapter adaptador;

    private BluetoothDevice dispositivo;

    private boolean conectado;

    private Thread hilo;

    public ConectarCliente(BluetoothAdapter adaptador, BluetoothDevice dispositivo) {

        this.adaptador = adaptador;

        this.dispositivo = dispositivo;

        this.conectado = false;

        BluetoothSocket clienteTemporal = null;

        try {
            clienteTemporal =
                    this.dispositivo.createRfcommSocketToServiceRecord(ServicioBluetooth.SEGURO);

        } catch (IOException e) {

            e.printStackTrace();
        }

        this.cliente = clienteTemporal;
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

        try {

            cliente.connect();

            this.conectado = true;

        } catch (IOException e) {

            e.printStackTrace();
        }

        try {

            if (cliente != null) {

                cliente.close();
            }

        } catch (IOException e) {

            e.printStackTrace();
        }

        return;
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
