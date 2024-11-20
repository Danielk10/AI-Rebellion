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

    public ServicioBluetooth(BluetoothAdapter adaptador) {

        this.adaptador = adaptador;

        conectado = false;

        dato = null;
    }

    public void setTipo(int tipo) {
        this.tipo = tipo;
    }

    public BluetoothDevice getDispositivo() {

        return dispositivo;
    }

    public void setConectarCliente(BluetoothDevice dispositivo) {

        this.dispositivo = dispositivo;

        if (CLIENTE == tipo) {

            conectarCliente = new ConectarCliente(adaptador, dispositivo);

            conectarCliente.iniciar();
        }
    }

    public void setConectarServidor() {

        if (SERVIDOR == tipo) {

            conectarServidor = new ConectarServidor(adaptador);

            conectarServidor.iniciar();
        }
    }

    public boolean isConectado() {

        if (CLIENTE == tipo) {

            conectado = conectarCliente.isConectado();
        }

        if (SERVIDOR == tipo) {

            conectado = conectarServidor.isConectado();
        }

        return conectado;
    }

    public DatoBluetooth getDatos() {

        if (CLIENTE == tipo) {

            dato = conectarCliente.getDato();
        }

        if (SERVIDOR == tipo) {

            dato = conectarServidor.getDato();
        }

        return dato;
    }

    public BluetoothAdapter getAdaptador() {
        return this.adaptador;
    }
}
