package com.diamon.iarebellion;

import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothManager;
import android.bluetooth.BluetoothServerSocket;
import android.bluetooth.BluetoothSocket;
import android.content.Context;
import android.content.pm.ActivityInfo;
import android.os.Build;
import android.os.Bundle;
import android.os.PowerManager;
import android.os.PowerManager.WakeLock;
import android.view.KeyEvent;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

import androidx.appcompat.app.AppCompatActivity;

import com.diamon.bluetooth.permiso.BluetoothPermissions;
import com.diamon.notificaciones.NotificacionScheduler;
import com.diamon.notificaciones.NotificacionUtils;

import java.io.IOException;
import java.util.UUID;

/**
 * Actividad Principal de IA R3bellion.
 * Aloja el GameView nativo acelerado en Rust y gestiona el ciclo de vida y Bluetooth RFCOMM.
 */
public class MainActivity extends AppCompatActivity {

    public static final UUID RFCOMM_UUID = UUID.fromString("fa87c0d0-afac-11de-8a39-0800200c9a66");

    private WakeLock wakeLock;
    private GameView gameView;
    private BluetoothAdapter bluetoothAdapter;
    private Thread acceptThread;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE);

        // 1. Inicializar canal de notificaciones y recordatorios
        try {
            NotificacionUtils.createNotificationChannel(this);
            NotificacionScheduler.programarNotificacion(this);
        } catch (Exception ignored) {}

        // 2. Crear GameView (SurfaceView conectado al motor Rust)
        gameView = new GameView(this);
        setContentView(gameView);
        hideSystemUI();

        // 3. WakeLock para evitar que la pantalla se apague durante el juego
        PowerManager powerManager = (PowerManager) getSystemService(Context.POWER_SERVICE);
        if (powerManager != null) {
            wakeLock = powerManager.newWakeLock(PowerManager.SCREEN_BRIGHT_WAKE_LOCK, "IARebellion:WakeLock");
        }

        // 4. Solicitar permisos Bluetooth si no están concedidos
        if (!BluetoothPermissions.arePermissionsGranted(this)) {
            BluetoothPermissions.requestPermissions(this);
        }

        initBluetooth();
    }

    private void initBluetooth() {
        try {
            BluetoothManager bm = (BluetoothManager) getSystemService(Context.BLUETOOTH_SERVICE);
            if (bm != null) {
                bluetoothAdapter = bm.getAdapter();
                if (bluetoothAdapter != null && bluetoothAdapter.isEnabled()) {
                    startAcceptServer();
                }
            }
        } catch (Exception ignored) {}
    }

    private void startAcceptServer() {
        acceptThread = new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    BluetoothServerSocket serverSocket = bluetoothAdapter.listenUsingRfcommWithServiceRecord(
                            "IARebellionMultiplayer",
                            RFCOMM_UUID
                    );
                    if (serverSocket != null) {
                        BluetoothSocket socket = serverSocket.accept();
                        serverSocket.close();
                        if (socket != null && gameView != null) {
                            runOnUiThread(new Runnable() {
                                @Override
                                public void run() {
                                    gameView.setBluetoothSocket(socket, true, 0);
                                }
                            });
                        }
                    }
                } catch (SecurityException | IOException ignored) {}
            }
        }, "BTAcceptThread");
        acceptThread.start();
    }

    public void connectToHost(final BluetoothDevice hostDevice) {
        new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    bluetoothAdapter.cancelDiscovery();
                    BluetoothSocket socket = hostDevice.createRfcommSocketToServiceRecord(RFCOMM_UUID);
                    socket.connect();
                    if (socket.isConnected() && gameView != null) {
                        final BluetoothSocket connectedSocket = socket;
                        runOnUiThread(new Runnable() {
                            @Override
                            public void run() {
                                gameView.setBluetoothSocket(connectedSocket, false, 1);
                            }
                        });
                    }
                } catch (SecurityException | IOException ignored) {}
            }
        }, "BTConnectThread").start();
    }

    private void hideSystemUI() {
        try {
            View decorView = getWindow() != null ? getWindow().getDecorView() : null;
            if (decorView != null) {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                    WindowInsetsController insetsController = decorView.getWindowInsetsController();
                    if (insetsController != null) {
                        insetsController.hide(WindowInsets.Type.statusBars() | WindowInsets.Type.navigationBars());
                        insetsController.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                    }
                } else {
                    decorView.setSystemUiVisibility(
                            View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                                    | View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                                    | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                                    | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                                    | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                                    | View.SYSTEM_UI_FLAG_FULLSCREEN
                    );
                }
            }
            if (getWindow() != null) {
                getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
            }
        } catch (Exception ignored) {}
    }

    @Override
    protected void onResume() {
        super.onResume();
        hideSystemUI();
        if (gameView != null) {
            gameView.resume();
        }
        if (wakeLock != null && !wakeLock.isHeld()) {
            wakeLock.acquire();
        }
    }

    @Override
    protected void onPause() {
        super.onPause();
        if (gameView != null) {
            gameView.pause();
        }
        if (wakeLock != null && wakeLock.isHeld()) {
            wakeLock.release();
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        if (acceptThread != null) {
            acceptThread.interrupt();
            acceptThread = null;
        }
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            hideSystemUI();
        }
    }

    @Override
    public boolean onKeyUp(int keyCode, KeyEvent event) {
        hideSystemUI();
        return super.onKeyUp(keyCode, event);
    }
}
