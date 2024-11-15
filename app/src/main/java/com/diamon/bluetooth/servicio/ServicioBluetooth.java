package com.diamon.bluetooth.servicio;

import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;

import com.diamon.bluetooth.iobluetooth.DatoBluetooth;

import java.util.UUID;

public class ServicioBluetooth {

    public static final UUID SEGURO = UUID.fromString("fa87c0d0-afac-11de-8a39-0800200c9a66");

    public static final int SERVIDOR = 1;

    public static final int CLIENTE = 2;

    public static final int ACTIVAR = 3;

    private int tipo;

    private BluetoothAdapter adaptador;

    private ConectarCliente conectarCliente;

    private ConectarServidor conectarServidor;

    private BluetoothDevice dispositivo;

    private DatoBluetooth dato;

    private boolean conectado;

    public ServicioBluetooth(BluetoothAdapter adaptador, int tipo) {

        this.tipo = tipo;

        this.adaptador = adaptador;

        conectado = false;
    }

    public BluetoothDevice getDispositivo() {

        return dispositivo;
    }

    public void setDispositivo(BluetoothDevice dispositivo) {

        this.dispositivo = dispositivo;

        if (CLIENTE == tipo) {

            conectarCliente = new ConectarCliente(adaptador, dispositivo);

            conectarCliente.iniciar();

            conectado = conectarCliente.isConectado();

            if (conectarCliente.getCliente() != null) {

                dato = new DatoBluetooth(conectarCliente.getCliente());

                dato.iniciar();
            }
        }

        if (SERVIDOR == tipo) {

            conectarServidor = new ConectarServidor(adaptador);

            conectarServidor.iniciar();

            conectado = conectarServidor.isConectado();

            if (conectarServidor.getCliente() != null) {

                dato = new DatoBluetooth(conectarServidor.getCliente());

                dato.iniciar();
            }
        }
    }

    public boolean isConectado() {

        return conectado;
    }

    public DatoBluetooth getDatos() {

        return dato;
    }

    public BluetoothAdapter getAdaptador() {
        return this.adaptador;
    }

    
}
