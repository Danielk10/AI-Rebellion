package com.diamon.bluetooth.iobluetooth;

import java.io.InputStream;
import java.io.OutputStream;
import java.io.IOException;

public interface TransmisionDatos {

    public abstract void enviarDatos(OutputStream outputStream) throws IOException;

    public abstract void recibirDatos(InputStream inputStream) throws IOException;

    public abstract float getX();

    public abstract float getY();

    public abstract boolean getDisparar();

    public abstract float getVelocidad();

    public abstract void setX(float x);

    public abstract void setY(float y);

    public abstract void setDisparar(boolean disparar);

    public abstract void setVelocidad(float velocidad);
}
