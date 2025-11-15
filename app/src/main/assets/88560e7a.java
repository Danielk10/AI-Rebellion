package com.diamon.utilidad;

import android.content.res.AssetManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.util.Log;

import com.diamon.graficos.Textura2D;
import com.diamon.nucleo.Graficos.FormatoTextura;
import com.diamon.nucleo.Textura;

import java.io.IOException;
import java.io.InputStream;
import java.util.HashMap;
import java.util.Map;

/**
 * Gestor de recursos mejorado para cargar y cachear texturas
 * Previene cargas duplicadas y facilita la gestión de memoria
 */
public class GestorDeRecursos {

    private static final String TAG = "GestorDeRecursos";

    private AssetManager assetManager;
    private Map<String, Textura> texturaCache;
    private FormatoTextura formatoPorDefecto;

    public GestorDeRecursos(AssetManager assetManager) {
        this.assetManager = assetManager;
        this.texturaCache = new HashMap<>();
        this.formatoPorDefecto = FormatoTextura.ARGB8888;
    }

    /**
     * Establece el formato de textura por defecto
     */
    public void setFormatoPorDefecto(FormatoTextura formato) {
        this.formatoPorDefecto = formato;
    }

    /**
     * Carga una textura desde assets
     * Si ya está cargada, retorna la versión en caché
     */
    public Textura cargarTextura(String rutaArchivo) {
        return cargarTextura(rutaArchivo, formatoPorDefecto);
    }

    /**
     * Carga una textura con un formato específico
     */
    public Textura cargarTextura(String rutaArchivo, FormatoTextura formato) {
        // Verificar si ya está en caché
        String cacheKey = rutaArchivo + "_" + formato.name();
        if (texturaCache.containsKey(cacheKey)) {
            Textura textura = texturaCache.get(cacheKey);
            // Verificar si la textura no está reciclada
            if (textura instanceof Textura2D) {
                Textura2D tex2d = (Textura2D) textura;
                if (!tex2d.isReciclada()) {
                    Log.d(TAG, "Textura cargada desde caché: " + rutaArchivo);
                    return textura;
                } else {
                    // Eliminar de caché si está reciclada
                    texturaCache.remove(cacheKey);
                }
            }
        }

        // Cargar desde archivo
        InputStream inputStream = null;
        try {
            inputStream = assetManager.open(rutaArchivo);

            // Configurar opciones de BitmapFactory según el formato
            BitmapFactory.Options opciones = new BitmapFactory.Options();
            opciones.inScaled = false; // No escalar automáticamente

            if (formato == FormatoTextura.RGB565) {
                opciones.inPreferredConfig = Bitmap.Config.RGB_565;
            } else if (formato == FormatoTextura.ARGB4444) {
                opciones.inPreferredConfig = Bitmap.Config.ARGB_4444;
            } else {
                opciones.inPreferredConfig = Bitmap.Config.ARGB_8888;
            }

            Bitmap bitmap = BitmapFactory.decodeStream(inputStream, null, opciones);

            if (bitmap == null) {
                Log.e(TAG, "Error al cargar textura: " + rutaArchivo);
                return null;
            }

            Textura textura = new Textura2D(bitmap);

            // Guardar en caché
            texturaCache.put(cacheKey, textura);

            Log.d(TAG, "Textura cargada: " + rutaArchivo + 
                  " (" + bitmap.getWidth() + "x" + bitmap.getHeight() + ")");

            return textura;

        } catch (IOException e) {
            Log.e(TAG, "Error al abrir archivo: " + rutaArchivo, e);
            return null;
        } finally {
            if (inputStream != null) {
                try {
                    inputStream.close();
                } catch (IOException e) {
                    Log.e(TAG, "Error al cerrar stream", e);
                }
            }
        }
    }

    /**
     * Carga múltiples texturas para una animación
     */
    public Textura[] cargarTexturas(String[] rutasArchivos) {
        return cargarTexturas(rutasArchivos, formatoPorDefecto);
    }

    /**
     * Carga múltiples texturas con un formato específico
     */
    public Textura[] cargarTexturas(String[] rutasArchivos, FormatoTextura formato) {
        Textura[] texturas = new Textura[rutasArchivos.length];
        for (int i = 0; i < rutasArchivos.length; i++) {
            texturas[i] = cargarTextura(rutasArchivos[i], formato);
        }
        return texturas;
    }

    /**
     * Libera una textura específica
     */
    public void liberarTextura(String rutaArchivo) {
        for (String key : texturaCache.keySet()) {
            if (key.startsWith(rutaArchivo + "_")) {
                Textura textura = texturaCache.get(key);
                if (textura != null) {
                    textura.dispose();
                }
                texturaCache.remove(key);
                Log.d(TAG, "Textura liberada: " + rutaArchivo);
            }
        }
    }

    /**
     * Libera todas las texturas cargadas
     */
    public void liberarTodas() {
        for (Textura textura : texturaCache.values()) {
            if (textura != null) {
                textura.dispose();
            }
        }
        texturaCache.clear();
        Log.d(TAG, "Todas las texturas liberadas");
    }

    /**
     * Obtiene el número de texturas en caché
     */
    public int getNumeroTexturasEnCache() {
        return texturaCache.size();
    }

    /**
     * Obtiene el tamaño aproximado de memoria usado por las texturas
     */
    public long getMemoriaUsada() {
        long total = 0;
        for (Textura textura : texturaCache.values()) {
            if (textura != null && textura.getBipmap() != null) {
                total += textura.getBipmap().getByteCount();
            }
        }
        return total;
    }

    /**
     * Obtiene el tamaño de memoria en MB
     */
    public float getMemoriaUsadaMB() {
        return getMemoriaUsada() / (1024f * 1024f);
    }
}
