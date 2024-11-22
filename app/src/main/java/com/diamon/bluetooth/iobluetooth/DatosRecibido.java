package com.diamon.bluetooth.iobluetooth;

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;

public class DatosRecibido implements TransmisionDatos {

    private float x;

    private float y;

    private boolean disparar;

    private float velocidad;

    // Modificar DataSender para incluir más datos
    @Override
    public void enviarDatos(OutputStream outputStream) throws IOException {}

    @Override
    public void recibirDatos(InputStream inputStream) throws IOException {

        byte[] buffer = new byte[13]; // Ajustado al nuevo tamaño

        int bytesRead = inputStream.read(buffer);

        if (bytesRead == 13) {

            ByteBuffer byteBuffer = ByteBuffer.wrap(buffer);

            float x = byteBuffer.getFloat();

            float y = byteBuffer.getFloat();

            boolean disparar = byteBuffer.get() == 1;

            float velocidad = byteBuffer.getFloat(); // Leer velocidad

            this.x = x;

            this.y = y;

            this.disparar = disparar;

            this.velocidad = velocidad;

        } else {

            throw new IOException("Los datos resividos estan incompletos");
        }
    }

    @Override
    public float getX() {
        return this.x;
    }

    @Override
    public float getY() {
        return this.y;
    }

    @Override
    public boolean getDisparar() {
        return this.disparar;
    }

    @Override
    public float getVelocidad() {
        return this.velocidad;
    }

    @Override
    public void setX(float x) {}

    @Override
    public void setY(float y) {}

    @Override
    public void setDisparar(boolean disparar) {}

    @Override
    public void setVelocidad(float velocidad) {}
}
