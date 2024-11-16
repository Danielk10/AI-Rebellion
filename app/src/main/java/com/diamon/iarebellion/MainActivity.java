package com.diamon.iarebellion;

import android.Manifest;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothManager;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.pm.ActivityInfo;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.os.PowerManager;
import android.os.PowerManager.WakeLock;
import android.view.KeyEvent;
import android.widget.FrameLayout;
import android.widget.RelativeLayout;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import com.diamon.bluetooth.servicio.ServicioBluetooth;
import com.diamon.juego.IARebellion;
import com.diamon.notificaciones.NotificacionScheduler;
import com.diamon.notificaciones.NotificacionUtils;
import com.diamon.utilidad.PantallaCompleta;
import com.diamon.utilidad.Recurso;

import java.util.ArrayList;
import kotlin.jvm.Volatile;

public class MainActivity extends AppCompatActivity {

    private WakeLock wakeLock;

    private IARebellion juego;

    private PantallaCompleta pantallaCompleta;

    public static Recurso recurso;

    private BluetoothAdapter adaptador;

    public ServicioBluetooth servicio;
    
    private int numeroDispositivos;

    public ArrayList<BluetoothDevice> dispositivos;

    private String[] permisos = {
        Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.ACCESS_FINE_LOCATION
    };

    public static final int REQUEST_CODE_PERMISOS = 1;

    public static final int REQUEST_CODE_BLUETOOTH_CONNECT = 2;

    /* private final ActivityResultLauncher<Intent> enableBluetoothLauncher =
    registerForActivityResult(
            new ActivityResultContracts.StartActivityForResult(),
            result -> {
                if (result.getResultCode() == Activity.RESULT_OK) {

                    buscarDispositivo();
                } else {
                    // El usuario rechazó la activación de Bluetooth
                }
            });*/

    private final BroadcastReceiver reservado =
            new BroadcastReceiver() {
                @Override
                public void onReceive(Context contexto, Intent intencion) {

                    
                    final String acccion = intencion.getAction();

                    if (BluetoothDevice.ACTION_FOUND.equals(acccion)) {

                        BluetoothDevice dispositivo =
                                intencion.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE);

                        if (dispositivo != null) {

                            Toast.makeText(
                                            getApplicationContext(),
                                            "Dispositivo Encontrado y Agregado: "
                                                    + dispositivo.getName(),
                                            Toast.LENGTH_SHORT)
                                    .show();
                    
                    
                       if(numeroDispositivos <1) {
                        
                       	dispositivos.add(dispositivo);
                    
                       }

                            
                    numeroDispositivos++;
                    
                        }
                    }
                }
            };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

       setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE);
        
        BluetoothManager bluetoothManager =
                (BluetoothManager)
                        getApplicationContext().getSystemService(Context.BLUETOOTH_SERVICE);
        
        dispositivos = new ArrayList<BluetoothDevice>();

        numeroDispositivos = 0;
        
        adaptador = bluetoothManager.getAdapter();

        servicio = new ServicioBluetooth(adaptador, ServicioBluetooth.SERVIDOR);

       //  servicio = new ServicioBluetooth(adaptador, ServicioBluetooth.CLIENTE);

        recurso = new Recurso(this);

        NotificacionUtils.createNotificationChannel(this);

        NotificacionScheduler.programarNotificacion(this);

        pantallaCompleta = new PantallaCompleta(this);

        pantallaCompleta.pantallaCompleta();

        pantallaCompleta.ocultarBotonesVirtuales();

        juego = new IARebellion(this, servicio);

        RelativeLayout mainLayout = new RelativeLayout(this);

        FrameLayout frame = new FrameLayout(this);

        RelativeLayout.LayoutParams mrecParameters =
                new RelativeLayout.LayoutParams(
                        RelativeLayout.LayoutParams.WRAP_CONTENT,
                        RelativeLayout.LayoutParams.WRAP_CONTENT);
        mrecParameters.addRule(RelativeLayout.CENTER_HORIZONTAL);
        mrecParameters.addRule(RelativeLayout.ALIGN_PARENT_TOP);

        frame.addView(
                juego,
                new FrameLayout.LayoutParams(
                        FrameLayout.LayoutParams.MATCH_PARENT,
                        FrameLayout.LayoutParams.MATCH_PARENT));

        frame.addView(mainLayout);

        setContentView(frame);

        PowerManager powerManejador = (PowerManager) getSystemService(Context.POWER_SERVICE);

        wakeLock = powerManejador.newWakeLock(PowerManager.FULL_WAKE_LOCK, "GLGame");

        permisos();
    }

    @Override
    protected void onPause() {

        super.onPause();

        juego.pausa();

        wakeLock.release();

        if (isFinishing()) {

            juego.liberarRecursos();
        }
    }

    @Override
    protected void onResume() {

        super.onResume();

        juego.resumen();

        wakeLock.acquire();
    }

    private void permisos() {

        if (ContextCompat.checkSelfPermission(
                                getApplicationContext(), Manifest.permission.BLUETOOTH_SCAN)
                        != PackageManager.PERMISSION_GRANTED
                || ContextCompat.checkSelfPermission(
                                getApplicationContext(), Manifest.permission.ACCESS_FINE_LOCATION)
                        != PackageManager.PERMISSION_GRANTED) {

            ActivityCompat.requestPermissions(this, permisos, MainActivity.REQUEST_CODE_PERMISOS);
        }
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);

        if (hasFocus) {

            pantallaCompleta.ocultarBotonesVirtuales();
        }
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {

        pantallaCompleta.ocultarBotonesVirtuales();

        return super.onKeyUp(keyCode, event);
    }

    @Override
    protected void onDestroy() {

        super.onDestroy();

        // unregisterReceiver(reservado);
    }

    public ArrayList<BluetoothDevice> getDispositivos() {
        return this.dispositivos;
    }

   
    /*@Override
    protected void onActivityResult(int arg0, int arg1, Intent arg2) {
        super.onActivityResult(arg0, arg1, arg2);

         buscarDispositivo();
    }*/

    /*  public void activarBluetooth() {

        if (adaptador != null && !adaptador.isEnabled()) {
            Intent intencion = new Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE);
            enableBluetoothLauncher.launch(intencion);
        }
    }*/

    /* private void buscarDispositivo() {

        if (adaptador.isEnabled()) {

            try {
                if (adaptador.startDiscovery()) {

                    IntentFilter intecionFiltrada = new IntentFilter(BluetoothDevice.ACTION_FOUND);

                    registerReceiver(reservado, intecionFiltrada);
                }
            } catch (Exception e) {

                e.printStackTrace();
            }
        }
    }*/

    public BroadcastReceiver getReservado() {
        return this.reservado;
    }
}
