package com.diamon.notificaciones;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;


public class NotificacionReceiver extends BroadcastReceiver {

    @Override
    public void onReceive(Context contexto, Intent intent) {

        // Mostrar la notificacion
        NotificacionHelper.mostrarNotificacion(contexto, "Sigue Jugando", "Sigue con la aventura");
    }
}
