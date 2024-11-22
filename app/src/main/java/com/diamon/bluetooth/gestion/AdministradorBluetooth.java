package com.diamon.bluetooth.gedtion;

import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;

import androidx.appcompat.app.AppCompatActivity;

import com.diamon.bluetooth.servicio.ServicioBluetooth;

import java.util.ArrayList;

public class AdministradorBluetooth {

    public static final int REQUEST_ENABLE_DISCOVERABLE = 5;

    public static ArrayList<BluetoothDevice> dispositivos = new ArrayList<BluetoothDevice>();

    private static final BroadcastReceiver RESERVADO =
            new BroadcastReceiver() {
                @Override
                public void onReceive(Context contexto, Intent intencion) {

                    final String acccion = intencion.getAction();

                    if (BluetoothDevice.ACTION_FOUND.equals(acccion)) {

                        BluetoothDevice dispositivo =
                                intencion.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE);

                        if (dispositivo != null) {

                            if (dispositivos != null) {

                                dispositivos.add(dispositivo);
                            }
                        }
                    }
                }
            };

    public static void activarBluetooth(AppCompatActivity activity, BluetoothAdapter adaptador) {

        if (adaptador != null) {

            if (!adaptador.isEnabled()) {

                Intent intencion = new Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE);

                activity.startActivityForResult(intencion, ServicioBluetooth.ACTIVAR);
            }
        }
    }

    public static void activarVisibilidad(
            AppCompatActivity activity, BluetoothAdapter adaptador, int durationSeconds) {

        BluetoothAdapter bluetoothAdapter = adaptador;

        if (bluetoothAdapter != null && bluetoothAdapter.isEnabled()) {

            Intent discoverableIntent = new Intent(BluetoothAdapter.ACTION_REQUEST_DISCOVERABLE);

            discoverableIntent.putExtra(
                    BluetoothAdapter.EXTRA_DISCOVERABLE_DURATION, durationSeconds);

            activity.startActivityForResult(discoverableIntent, REQUEST_ENABLE_DISCOVERABLE);
        }
    }

    public static void buscarDispositivo(AppCompatActivity activity, BluetoothAdapter adaptador) {

        if (adaptador.isEnabled()) {

            try {

                IntentFilter intecionFiltrada = new IntentFilter(BluetoothDevice.ACTION_FOUND);

                activity.registerReceiver(RESERVADO, intecionFiltrada);

                adaptador.startDiscovery();

            } catch (Exception e) {

                e.printStackTrace();
            }
        }
    }

    public static void cancelarBusqueda(AppCompatActivity activity, BluetoothAdapter adaptador) {

        if (adaptador != null && adaptador.isDiscovering()) {

            adaptador.cancelDiscovery();
        }
        activity.unregisterReceiver(RESERVADO);
    }

    public static void vincularDispoditivo(BluetoothDevice dispositivo) {
        try {

            dispositivo.createBond();

        } catch (Exception e) {

            e.printStackTrace();
        }
    }
}
