package com.diamon.bluetooth.servidor;

import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import com.diamon.bluetooth.servicio.ServicioBluetooth;

import com.diamon.iarebellion.MainActivity;
import java.util.ArrayList;

public class ServidorBluetooth extends AppCompatActivity {

    public ServicioBluetooth servicio;

    private ArrayList<BluetoothDevice> dispositivos;

    private TextView textoActualizar;

    private Button actualizar;

    private BluetoothAdapter adaptador;

    private LinearLayout diseno;

    private boolean encontrado;

    private int i;

    private final BroadcastReceiver reservado =
            new BroadcastReceiver() {
                @Override
                public void onReceive(Context contexto, Intent intencion) {

                    dispositivos = new ArrayList<BluetoothDevice>();

                    String acccion = intencion.getAction();

                    if (BluetoothDevice.ACTION_FOUND.equals(acccion)) {

                        BluetoothDevice dispositivo =
                                intencion.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE);

                        if (dispositivo != null) {

                            encontrado = true;

                            i++;

                            Toast.makeText(
                                            contexto,
                                            "Dispositivo Encontrado y Agregado: "
                                                    + dispositivo.getName(),
                                            Toast.LENGTH_SHORT)
                                    .show();

                            dispositivos.add(dispositivo);

                            if (encontrado) {

                                if (dispositivos.get(i) != null) {

                                    TextView texto = new TextView(ServidorBluetooth.this);

                                    texto.setText(dispositivos.get(i).getName());

                                    Button boton = new Button(ServidorBluetooth.this);

                                    boton.setText("Aceptar");

                                    final int j = i;

                                    boton.setOnClickListener(
                                            new View.OnClickListener() {

                                                @Override
                                                public void onClick(View v) {

                                                    BluetoothDevice dis =
                                                            adaptador.getRemoteDevice(
                                                                    dispositivos
                                                                            .get(j)
                                                                            .getAddress());

                                                    servicio.setDispositivo(dis);

                                                    Intent nuevaActividad =
                                                            new Intent(
                                                                    ServidorBluetooth.this,
                                                                    MainActivity.class);

                                                    startActivity(nuevaActividad);
                                                }
                                            });

                                    diseno.addView(texto);

                                    diseno.addView(boton);
                                }

                                encontrado = false;
                            }
                        }
                    }
                }
            };

    @Override
    protected void onStart() {

        super.onStart();

        i = -1;

        adaptador = BluetoothAdapter.getDefaultAdapter();

        servicio = new ServicioBluetooth(adaptador, ServicioBluetooth.SERVIDOR);

        textoActualizar = new TextView(this);

        textoActualizar.setText("Actualizar: ");

        actualizar = new Button(this);

        actualizar.setText("Actualizar");

        diseno = new LinearLayout(this);

        diseno.addView(textoActualizar);

        diseno.addView(actualizar);

        activarBluetooth();

        setContentView(diseno);

        // eventos();
    }

    private void eventos() {

        actualizar.setOnClickListener(
                new View.OnClickListener() {
                    @Override
                    public void onClick(View v) {

                        if (encontrado) {

                            for (int i = 0; i < dispositivos.size(); i++) {

                                if (dispositivos.get(i) != null) {

                                    TextView texto = new TextView(ServidorBluetooth.this);

                                    texto.setText(dispositivos.get(i).getName());

                                    Button boton = new Button(ServidorBluetooth.this);

                                    boton.setText("Aceptar");

                                    final int j = i;

                                    boton.setOnClickListener(
                                            new View.OnClickListener() {

                                                @Override
                                                public void onClick(View v) {

                                                    BluetoothDevice dis =
                                                            adaptador.getRemoteDevice(
                                                                    dispositivos
                                                                            .get(j)
                                                                            .getAddress());

                                                    servicio.setDispositivo(dis);

                                                    Intent nuevaActividad =
                                                            new Intent(
                                                                    ServidorBluetooth.this,
                                                                    MainActivity.class);

                                                    startActivity(nuevaActividad);
                                                }
                                            });

                                    diseno.addView(texto);

                                    diseno.addView(boton);
                                }
                            }

                            encontrado = false;
                        }
                    }
                });
    }

    @Override
    protected void onResume() {

        super.onResume();

        buscarDispositivo();
    }

    public void activarBluetooth() {

        if (!adaptador.isEnabled()) {

            Intent intencion = new Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE);

            startActivityForResult(intencion, ServicioBluetooth.ACTIVAR);
        }
    }

    @Override
    protected void onActivityResult(int arg0, int arg1, Intent arg2) {
        super.onActivityResult(arg0, arg1, arg2);

        buscarDispositivo();
    }

    private void buscarDispositivo() {

        if (adaptador.isEnabled()) {

            if (adaptador.startDiscovery()) {

                Toast.makeText(this, "Buscando", Toast.LENGTH_SHORT).show();

                IntentFilter intecionFiltrada = new IntentFilter(BluetoothDevice.ACTION_FOUND);

                registerReceiver(reservado, intecionFiltrada);
            }
        }
    }

    @Override
    protected void onDestroy() {

        super.onDestroy();

        unregisterReceiver(reservado);
    }
}
