package com.diamon.iarebellion;

import android.content.Context;
import android.os.Bundle;
import android.os.PowerManager;
import android.os.PowerManager.WakeLock;
import android.view.KeyEvent;
import android.widget.FrameLayout;
import android.widget.RelativeLayout;

import androidx.appcompat.app.AppCompatActivity;

import com.diamon.juego.IARebellion;
import com.diamon.notificaciones.NotificacionScheduler;
import com.diamon.notificaciones.NotificacionUtils;
import com.diamon.utilidad.PantallaCompleta;
import com.diamon.utilidad.Recurso;

public class MainActivity extends AppCompatActivity {

    private WakeLock wakeLock;

    private IARebellion juego;

    private PantallaCompleta pantallaCompleta;

    public static Recurso recurso;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        recurso = new Recurso(this);

        NotificacionUtils.createNotificationChannel(this);

        NotificacionScheduler.programarNotificacion(this);

        pantallaCompleta = new PantallaCompleta(this);

        pantallaCompleta.pantallaCompleta();

        pantallaCompleta.ocultarBotonesVirtuales();

        juego = new IARebellion(this);

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
}
