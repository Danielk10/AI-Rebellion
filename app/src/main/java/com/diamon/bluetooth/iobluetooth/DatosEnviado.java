package com.diamon.bluetooth.iobluetooth;

import java.io.OutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;

public class DatosEnviado implements TransmisionDatos {

    private float x;

    private float y;

    private boolean disparar;

    private float velocidad;

    // Modificar DataSender para incluir más datos
    @Override
    public void enviarDatos(OutputStream outputStream) throws IOException {

        byte[] buffer = new byte[13]; // 9 + 4 bytes para velocidad

        ByteBuffer byteBuffer = ByteBuffer.wrap(buffer);

        byteBuffer.putFloat(x); // X

        byteBuffer.putFloat(y); // Y

        byteBuffer.put((byte) (disparar ? 1 : 0)); // Disparar

        byteBuffer.putFloat(velocidad); // Velocidad

        outputStream.write(buffer);

        outputStream.flush();
    }

    @Override
    public void recibirDatos(InputStream inputStream) throws IOException {}

    @Override
    public void setX(float x) {
        this.x = x;
    }

    @Override
    public void setY(float y) {
        this.y = y;
    }

    @Override
    public void setDisparar(boolean disparar) {
        this.disparar = disparar;
    }

    @Override
    public void setVelocidad(float velocidad) {
        this.velocidad = velocidad;
    }

    @Override
    public float getX() {
        return 0;
    }

    @Override
    public float getY() {
        return 0;
    }

    @Override
    public boolean getDisparar() {
        return false;
    }

    @Override
    public float getVelocidad() {
        return 0;
    }
}
