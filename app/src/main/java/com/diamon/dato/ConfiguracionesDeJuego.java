package com.diamon.dato;

import android.app.Activity;
import android.os.Environment;


import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;

public class ConfiguracionesDeJuego {

    private boolean leerDatosAsset;

    private DatosJuego datos;

    public static final int ASSET = 1;

    public static final int INTERNO = 0;

    public static final int EXTERNO = 3;

    private int tipo;

    public ConfiguracionesDeJuego(Activity actividad, int tipo) {

        datos = new DatosJuego(actividad);

        this.tipo = tipo;
    }

    public boolean isLeerDatosAsset() {
        return leerDatosAsset;
    }

    public void setLeerDatosAsset(boolean leerDatosAsset) {
        this.leerDatosAsset = leerDatosAsset;
    }

    public ConfiguracionesDeJuego cargarConfiguraciones() {

        BufferedReader buferarchivoLeer = null;

        try {

            if (Environment.getExternalStorageState().equals(Environment.MEDIA_MOUNTED)) {
                
                if (tipo == ConfiguracionesDeJuego.ASSET) {

                    tipo = ConfiguracionesDeJuego.ASSET;

                } else {

                    tipo = ConfiguracionesDeJuego.EXTERNO;
                }

            } else {

                if (tipo == ConfiguracionesDeJuego.ASSET) {

                    tipo = ConfiguracionesDeJuego.ASSET;

                } else {

                    tipo = ConfiguracionesDeJuego.INTERNO;
                }
            }

            if (tipo == ConfiguracionesDeJuego.ASSET) {

                buferarchivoLeer =
                        new BufferedReader(
                                new InputStreamReader(datos.leerAsset("datos.txt"), "UTF-8"));
            }

            if (tipo == ConfiguracionesDeJuego.EXTERNO) {

                buferarchivoLeer =
                        new BufferedReader(
                                new InputStreamReader(
                                        datos.leerDatoExterno(DatosJuego.DATOS), "UTF-8"));
            }

            if (tipo == ConfiguracionesDeJuego.INTERNO) {

                buferarchivoLeer =
                        new BufferedReader(
                                new InputStreamReader(
                                        datos.leerDatoInterno(DatosJuego.DATOS), "UTF-8"));
            }

            leerDatosAsset = Boolean.parseBoolean(buferarchivoLeer.readLine());

        } catch (IOException e) {

        } finally {
            try {
                if (buferarchivoLeer != null) {

                    buferarchivoLeer.close();
                }

            } catch (IOException e) {

            }
        }

        return this;
    }

    public void guardarConfiguraciones() {

        BufferedWriter buferarchivoEscribir = null;

        try {

            if (Environment.getExternalStorageState().equals(Environment.MEDIA_MOUNTED)) {
                tipo = ConfiguracionesDeJuego.EXTERNO;

            } else {

                tipo = ConfiguracionesDeJuego.INTERNO;
            }

            if (tipo == ConfiguracionesDeJuego.EXTERNO) {

                buferarchivoEscribir =
                        new BufferedWriter(
                                new OutputStreamWriter(
                                        datos.escribirDatoExterno(DatosJuego.DATOS)));
            }

            if (tipo == ConfiguracionesDeJuego.INTERNO) {

                buferarchivoEscribir =
                        new BufferedWriter(
                                new OutputStreamWriter(
                                        datos.escribirDatoInterno(DatosJuego.DATOS)));
            }

            buferarchivoEscribir.write(Boolean.toString(leerDatosAsset));

            buferarchivoEscribir.newLine();

        } catch (IOException e) {

        } finally {
            if (buferarchivoEscribir != null) {
                try {

                    buferarchivoEscribir.close();

                } catch (IOException e) {

                }
            }
        }
    }
}
