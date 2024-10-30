package com.diamon.notificaciones;

import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;

import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.NotificationCompat;

import com.diamon.graficos.Textura2D;
import com.diamon.iarebellion.MainActivity;
import com.diamon.iarebellion.R;

public class NotificacionHelper {

    public static void mostrarNotificacion(Context contexto, String titulo, String mensaje) {

        // Crear la intención que abrirá la app cuando el usuario toque la notificación
        Intent intent = new Intent(contexto, MainActivity.class);
        PendingIntent pendingIntent =
                PendingIntent.getActivity(
                        contexto,
                        0,
                        intent,
                        PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);

        // Crear la notificación
        NotificationCompat.Builder builder =
                new NotificationCompat.Builder(contexto, NotificacionUtils.CHANNEL_ID)
                        .setSmallIcon(R.mipmap.ic_launcher) // Cambia este ícono según sea
                        // necesario
                        .setLargeIcon(
                                new Textura2D(
                                                MainActivity.recurso
                                                        .cargarTextura("creditos.png")
                                                        .getBipmap(),
                                                150,
                                                100)
                                        .getBipmap())
                        .setContentTitle(titulo)
                        .setContentText(mensaje)
                        .setStyle(
                                new NotificationCompat.BigPictureStyle()
                                        .bigPicture(
                                                new Textura2D(
                                                                MainActivity.recurso
                                                                        .cargarTextura(
                                                                                "creditos.png")
                                                                        .getBipmap(),
                                                                300,
                                                                250)
                                                        .getBipmap()))
                        .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                        .setContentIntent(pendingIntent)
                        .setAutoCancel(true); // Elimina la notificación cuando se toca

        // Mostrar la notificación
        NotificationManager notificationManager =
                (NotificationManager) contexto.getSystemService(Context.NOTIFICATION_SERVICE);
        if (notificationManager != null) {

            notificationManager.notify(1, builder.build());
        }
    }

    public static void mostrarNotificacionPersanslizada(
            Context contexto, String titulo, String mensaje) {

        // Crear la intención que abrirá la app cuando el usuario toque la notificación
        Intent intent = new Intent(contexto, MainActivity.class);
        PendingIntent pendingIntent =
                PendingIntent.getActivity(
                        contexto,
                        0,
                        intent,
                        PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);

        NotificationCompat.Builder builder =
                new NotificationCompat.Builder(contexto, NotificacionUtils.CHANNEL_ID)
                        .setSmallIcon(R.mipmap.ic_launcher)
                        .setLargeIcon(
                                new Textura2D(
                                                MainActivity.recurso
                                                        .cargarTextura("creditos.png")
                                                        .getBipmap(),
                                                150,
                                                100)
                                        .getBipmap())
                        .setContentTitle(titulo)
                        .setContentText(mensaje)
                        .setStyle(
                                new NotificationCompat.BigPictureStyle()
                                        .bigPicture(
                                                new Textura2D(
                                                                MainActivity.recurso
                                                                        .cargarTextura(
                                                                                "creditos.png")
                                                                        .getBipmap(),
                                                                300,
                                                                250)
                                                        .getBipmap()))
                        .setPriority(NotificationCompat.PRIORITY_HIGH)
                        .setContentIntent(pendingIntent)
                        .setAutoCancel(true); // Elimina la notificación cuando se toca

        NotificationManager notificationManager =
                (NotificationManager) contexto.getSystemService(Context.NOTIFICATION_SERVICE);
        if (notificationManager != null) {

            notificationManager.notify(1, builder.build());
        }
    }
}
