package com.diamon.notificaciones;

import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.content.Context;
import android.os.Build;

public class NotificacionUtils {

    public static final String CHANNEL_ID = "alerta_channel";

    public static void createNotificationChannel(Context contexto) {
        // Para Android 8.0 y superior se necesita crear un canal de notificaciones
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            CharSequence name = "Notificacion";
            String description = "Canal para Notificaciones";
            int importance = NotificationManager.IMPORTANCE_DEFAULT;

            NotificationChannel channel = new NotificationChannel(CHANNEL_ID, name, importance);
            channel.setDescription(description);

            // Registrar el canal con el NotificationManager
            NotificationManager notificationManager =
                    contexto.getSystemService(NotificationManager.class);
            notificationManager.createNotificationChannel(channel);
        }
    }
}
