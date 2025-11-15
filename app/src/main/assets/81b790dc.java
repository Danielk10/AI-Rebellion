package com.diamon.graficos;

import android.graphics.Canvas;
import android.graphics.Paint;
import android.graphics.Rect;

import com.diamon.nucleo.Graficos;
import com.diamon.nucleo.Textura;

public class Graficos2D implements Graficos {

    private Canvas canvas;
    private Paint paint;
    private Textura textura;

    // OPTIMIZACIÓN: Reutilizar instancias de Rect para evitar GC
    private Rect dstRect;
    private Rect srcRect;

    public Graficos2D(Textura textura) {
        this.textura = textura;
        this.canvas = new Canvas(textura.getBipmap());
        this.paint = new Paint();

        // Configurar el paint para mejor rendimiento
        this.paint.setAntiAlias(false); // Desactivar antialiasing para mejor rendimiento
        this.paint.setFilterBitmap(false); // Desactivar filtrado para pixel art

        this.dstRect = new Rect();
        this.srcRect = new Rect();
    }

    @Override
    public void limpiar(int color) {
        this.canvas.drawRGB((color & 0xff0000) >> 16, (color & 0xff00) >> 8, (color & 0xff));
    }

    @Override
    public void dibujarPixel(float x, float y, int color) {
        this.paint.setColor(color);
        this.canvas.drawPoint(x, y, this.paint);
    }

    @Override
    public void dibujarLinea(float x, float y, float x1, float y1, int color) {
        this.paint.setColor(color);
        this.canvas.drawLine(x, y, x1, y1, this.paint);
    }

    @Override
    public void dibujarRectangulo(float x, float y, float ancho, float alto, int color) {
        this.paint.setColor(color);
        this.paint.setStyle(Paint.Style.FILL);
        this.canvas.drawRect(x, y, x + ancho, y + alto, this.paint);
    }

    @Override
    public void dibujarTexto(String texto, float x, float y, int color) {
        this.paint.setColor(color);
        this.canvas.drawText(texto, x, y, paint);
    }

    @Override
    public void dibujarTextura(Textura textura, float x, float y) {
        this.canvas.drawBitmap(textura.getBipmap(), x, y, null);
    }

    @Override
    public void dibujarTextura(
            Textura textura, float x, float y, float x1, float y1, float ancho, float alto) {

        // Configurar rectángulo fuente
        this.srcRect.left = (int) x1;
        this.srcRect.top = (int) y1;
        this.srcRect.right = (int) (x1 + ancho);
        this.srcRect.bottom = (int) (y1 + alto);

        // Configurar rectángulo destino
        this.dstRect.left = (int) x;
        this.dstRect.top = (int) y;
        this.dstRect.right = (int) (x + ancho);
        this.dstRect.bottom = (int) (y + alto);

        this.canvas.drawBitmap(textura.getBipmap(), this.srcRect, this.dstRect, null);
    }

    @Override
    public Paint getLapiz() {
        return this.paint;
    }

    @Override
    public float getAncho() {
        return textura.getAncho();
    }

    @Override
    public float getAlto() {
        return textura.getAlto();
    }

    @Override
    public Textura crearTextura(float ancho, float alto, FormatoTextura formatoTextura) {
        // CORRECCIÓN: Usar ancho y alto correctamente (antes usaba alto dos veces)
        textura = new Textura2D(ancho, alto, formatoTextura);
        return textura;
    }

    public Canvas getCanvas() {
        return this.canvas;
    }

    /**
     * Habilita o deshabilita antialiasing
     */
    public void setAntiAlias(boolean antiAlias) {
        this.paint.setAntiAlias(antiAlias);
    }

    /**
     * Habilita o deshabilita filtrado de bitmap (útil para pixel art)
     */
    public void setFilterBitmap(boolean filter) {
        this.paint.setFilterBitmap(filter);
    }
}
