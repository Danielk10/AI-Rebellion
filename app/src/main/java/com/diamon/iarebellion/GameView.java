package com.diamon.iarebellion;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.graphics.Rect;
import android.media.AudioFormat;
import android.media.AudioManager;
import android.media.AudioTrack;
import android.os.Process;
import android.util.AttributeSet;
import android.view.MotionEvent;
import android.view.SurfaceHolder;
import android.view.SurfaceView;

import java.io.InputStream;
import java.io.OutputStream;
import java.net.Socket;
import android.bluetooth.BluetoothSocket;

/**
 * Vista de alto rendimiento basada en SurfaceView para IA R3bellion.
 * Comunica eventos táctiles, renderizado de píxeles y audio PCM directamente con el motor nativo en Rust.
 */
public class GameView extends SurfaceView implements SurfaceHolder.Callback, Runnable {

    // Resolución virtual interna del rasterizador (16:9 retro widescreen)
    public static final int VIRTUAL_WIDTH = 960;
    public static final int VIRTUAL_HEIGHT = 540;

    private SurfaceHolder holder;
    private Thread renderThread;
    private Thread audioThread;
    private volatile boolean isRunning = false;

    private int[] pixelBuffer;
    private Bitmap renderBitmap;
    private Rect srcRect;
    private Rect dstRect;
    private Paint renderPaint;

    private BluetoothSocket bluetoothSocket;
    private Thread bluetoothThread;

    public GameView(Context context) {
        super(context);
        init();
    }

    public GameView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    private void init() {
        holder = getHolder();
        holder.addCallback(this);

        pixelBuffer = new int[VIRTUAL_WIDTH * VIRTUAL_HEIGHT];
        renderBitmap = Bitmap.createBitmap(VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Bitmap.Config.ARGB_8888);
        srcRect = new Rect(0, 0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
        dstRect = new Rect(0, 0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT);

        renderPaint = new Paint();
        renderPaint.setFilterBitmap(false); // Estilo píxel art retro nítido

        setFocusable(true);
        setKeepScreenOn(true);

        // Inicializar motor en Rust
        GameBridge.nativeInit(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
    }

    public void setBluetoothSocket(BluetoothSocket socket, boolean isHost, int playerId) {
        this.bluetoothSocket = socket;
        GameBridge.nativeSetMultiplayerMode(isHost, playerId);
        startBluetoothSync();
    }

    @Override
    public void surfaceCreated(SurfaceHolder holder) {
        resume();
    }

    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        dstRect.set(0, 0, width, height);
        GameBridge.nativeResize(VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
    }

    @Override
    public void surfaceDestroyed(SurfaceHolder holder) {
        pause();
    }

    public void resume() {
        if (!isRunning) {
            isRunning = true;
            renderThread = new Thread(this, "GameRenderThread");
            renderThread.start();

            startAudioThread();
            startBluetoothSync();
        }
    }

    public void pause() {
        isRunning = false;
        if (renderThread != null) {
            try {
                renderThread.join(500);
            } catch (InterruptedException ignored) {}
            renderThread = null;
        }
        if (audioThread != null) {
            try {
                audioThread.join(500);
            } catch (InterruptedException ignored) {}
            audioThread = null;
        }
        if (bluetoothThread != null) {
            try {
                bluetoothThread.interrupt();
            } catch (Exception ignored) {}
            bluetoothThread = null;
        }
    }

    @Override
    public void run() {
        long lastTime = System.nanoTime();
        final long targetFrameNs = 16_666_666L; // 60 FPS

        while (isRunning) {
            long now = System.nanoTime();
            float dt = (now - lastTime) / 1_000_000_000.0f;
            lastTime = now;

            // Limitar dt para evitar saltos temporales extremos
            if (dt > 0.1f) dt = 0.016f;

            // 1. Actualizar lógica y rasterizar fotograma en Rust
            GameBridge.nativeUpdateAndRender(pixelBuffer, dt);

            // 2. Volcar píxeles al SurfaceView
            Canvas canvas = null;
            try {
                canvas = holder.lockCanvas();
                if (canvas != null) {
                    renderBitmap.setPixels(pixelBuffer, 0, VIRTUAL_WIDTH, 0, 0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT);
                    canvas.drawBitmap(renderBitmap, srcRect, dstRect, renderPaint);
                }
            } catch (Exception e) {
                // Ignore lock drops
            } finally {
                if (canvas != null) {
                    try {
                        holder.unlockCanvasAndPost(canvas);
                    } catch (Exception ignored) {}
                }
            }

            // 3. Regular a 60 FPS
            long elapsedNs = System.nanoTime() - now;
            long sleepNs = targetFrameNs - elapsedNs;
            if (sleepNs > 1_000_000L) {
                try {
                    Thread.sleep(sleepNs / 1_000_000L, (int) (sleepNs % 1_000_000L));
                } catch (InterruptedException ignored) {}
            }
        }
    }

    private void startAudioThread() {
        audioThread = new Thread(new Runnable() {
            @Override
            public void run() {
                Process.setThreadPriority(Process.THREAD_PRIORITY_AUDIO);
                final int sampleRate = 44100;
                final int minBufSize = AudioTrack.getMinBufferSize(
                        sampleRate,
                        AudioFormat.CHANNEL_OUT_MONO,
                        AudioFormat.ENCODING_PCM_16BIT
                );
                final int bufferSize = Math.max(minBufSize, 2048);
                short[] audioBuffer = new short[bufferSize];

                AudioTrack track = new AudioTrack(
                        AudioManager.STREAM_MUSIC,
                        sampleRate,
                        AudioFormat.CHANNEL_OUT_MONO,
                        AudioFormat.ENCODING_PCM_16BIT,
                        bufferSize * 2,
                        AudioTrack.MODE_STREAM
                );

                try {
                    track.play();
                    while (isRunning) {
                        GameBridge.nativeGetAudioSamples(audioBuffer, audioBuffer.length);
                        track.write(audioBuffer, 0, audioBuffer.length);
                    }
                    track.stop();
                } catch (Exception ignored) {
                } finally {
                    try {
                        track.release();
                    } catch (Exception ignored) {}
                }
            }
        }, "GameAudioThread");
        audioThread.start();
    }

    private void startBluetoothSync() {
        if (bluetoothSocket == null || !bluetoothSocket.isConnected()) return;

        bluetoothThread = new Thread(new Runnable() {
            @Override
            public void run() {
                try {
                    InputStream in = bluetoothSocket.getInputStream();
                    OutputStream out = bluetoothSocket.getOutputStream();
                    byte[] readBuf = new byte[1024];

                    while (isRunning && bluetoothSocket.isConnected()) {
                        // Enviar datos salientes de Rust
                        byte[] outgoing = GameBridge.nativeBluetoothGetOutgoing();
                        if (outgoing != null && outgoing.length > 0) {
                            out.write(outgoing);
                            out.flush();
                        }

                        // Leer datos entrantes y pasarlos a Rust
                        if (in.available() > 0) {
                            int count = in.read(readBuf);
                            if (count > 0) {
                                byte[] packet = new byte[count];
                                System.arraycopy(readBuf, 0, packet, 0, count);
                                GameBridge.nativeBluetoothReceive(packet);
                            }
                        }

                        Thread.sleep(15);
                    }
                } catch (Exception ignored) {}
            }
        }, "GameBluetoothThread");
        bluetoothThread.start();
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        int action = event.getActionMasked();
        int actionIndex = event.getActionIndex();
        float scaleX = (float) VIRTUAL_WIDTH / Math.max(1, getWidth());
        float scaleY = (float) VIRTUAL_HEIGHT / Math.max(1, getHeight());

        switch (action) {
            case MotionEvent.ACTION_DOWN:
            case MotionEvent.ACTION_POINTER_DOWN: {
                int pointerId = event.getPointerId(actionIndex);
                float x = event.getX(actionIndex) * scaleX;
                float y = event.getY(actionIndex) * scaleY;
                GameBridge.nativeTouchDown(pointerId, x, y);
                return true;
            }
            case MotionEvent.ACTION_MOVE: {
                int pointerCount = event.getPointerCount();
                for (int i = 0; i < pointerCount; i++) {
                    int pointerId = event.getPointerId(i);
                    float x = event.getX(i) * scaleX;
                    float y = event.getY(i) * scaleY;
                    GameBridge.nativeTouchMove(pointerId, x, y);
                }
                return true;
            }
            case MotionEvent.ACTION_UP:
            case MotionEvent.ACTION_POINTER_UP: {
                int pointerId = event.getPointerId(actionIndex);
                float x = event.getX(actionIndex) * scaleX;
                float y = event.getY(actionIndex) * scaleY;
                GameBridge.nativeTouchUp(pointerId, x, y);
                return true;
            }
            case MotionEvent.ACTION_CANCEL: {
                for (int i = 0; i < event.getPointerCount(); i++) {
                    int pointerId = event.getPointerId(i);
                    float x = event.getX(i) * scaleX;
                    float y = event.getY(i) * scaleY;
                    GameBridge.nativeTouchUp(pointerId, x, y);
                }
                return true;
            }
        }
        return super.onTouchEvent(event);
    }
}
