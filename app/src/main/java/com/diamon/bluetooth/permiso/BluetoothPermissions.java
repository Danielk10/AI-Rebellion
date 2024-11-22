package com.diamon.bluetooth.permiso;

import android.Manifest;
import android.content.pm.PackageManager;
import android.os.Build;

import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

public class BluetoothPermissions {

    public static final int REQUEST_BLUETOOTH_PERMISSIONS = 1;

    public static boolean arePermissionsGranted(AppCompatActivity activity) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            return ContextCompat.checkSelfPermission(
                                    activity.getApplicationContext(),
                                    Manifest.permission.BLUETOOTH_SCAN)
                            == PackageManager.PERMISSION_GRANTED
                    && ContextCompat.checkSelfPermission(
                                    activity.getApplicationContext(),
                                    Manifest.permission.BLUETOOTH_CONNECT)
                            == PackageManager.PERMISSION_GRANTED;
        } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            return ContextCompat.checkSelfPermission(
                            activity.getApplicationContext(),
                            Manifest.permission.ACCESS_FINE_LOCATION)
                    == PackageManager.PERMISSION_GRANTED;
        }
        return true;
    }

    public static void requestPermissions(AppCompatActivity activity) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            ActivityCompat.requestPermissions(
                    activity,
                    new String[] {
                        Manifest.permission.BLUETOOTH_SCAN, Manifest.permission.BLUETOOTH_CONNECT
                    },
                    REQUEST_BLUETOOTH_PERMISSIONS);
        } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            ActivityCompat.requestPermissions(
                    activity,
                    new String[] {Manifest.permission.ACCESS_FINE_LOCATION},
                    REQUEST_BLUETOOTH_PERMISSIONS);
        }
    }
}
