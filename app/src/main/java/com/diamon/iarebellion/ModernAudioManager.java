package com.diamon.iarebellion;

import android.content.Context;
import android.content.res.AssetFileDescriptor;
import android.media.AudioAttributes;
import android.media.MediaPlayer;
import android.media.SoundPool;
import android.util.Log;

import java.io.IOException;

/**
 * Gestor de Audio Moderno 2026 para IA R3bellion.
 * Utiliza SoundPool para efectos de sonido (SFX) OGG de baja latencia
 * y MediaPlayer para música de fondo (BGM) en bucle con compresión Vorbis.
 */
public class ModernAudioManager {
    private static final String TAG = "ModernAudioManager";

    private SoundPool soundPool;
    private MediaPlayer bgmPlayer;
    private Context context;

    private int sfxLaser = -1;
    private int sfxExplosion = -1;
    private int sfxBomb = -1;
    private int sfxShieldHit = -1;
    private int sfxBossAlarm = -1;

    public ModernAudioManager(Context context) {
        this.context = context.getApplicationContext();
        initSoundPool();
        initBgm();
    }

    private void initSoundPool() {
        AudioAttributes attrs = new AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_GAME)
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                .build();

        soundPool = new SoundPool.Builder()
                .setMaxStreams(12)
                .setAudioAttributes(attrs)
                .build();

        try {
            sfxLaser = loadAssetSound("audio/laser_plasma.ogg");
            sfxExplosion = loadAssetSound("audio/explosion_heavy.ogg");
            sfxBomb = loadAssetSound("audio/emp_bomb.ogg");
            sfxShieldHit = loadAssetSound("audio/shield_hit.ogg");
            sfxBossAlarm = loadAssetSound("audio/boss_alarm.ogg");
            Log.d(TAG, "Audio SFX OGG 2026 cargado exitosamente en SoundPool.");
        } catch (Exception e) {
            Log.w(TAG, "Advertencia cargando SFX: " + e.getMessage());
        }
    }

    private int loadAssetSound(String path) {
        try {
            AssetFileDescriptor afd = context.getAssets().openFd(path);
            return soundPool.load(afd, 1);
        } catch (IOException e) {
            Log.e(TAG, "Error abriendo asset: " + path, e);
            return -1;
        }
    }

    private void initBgm() {
        try {
            AssetFileDescriptor afd = context.getAssets().openFd("audio/bgm_cyber_rebellion.ogg");
            bgmPlayer = new MediaPlayer();
            bgmPlayer.setDataSource(afd.getFileDescriptor(), afd.getStartOffset(), afd.getLength());
            afd.close();
            bgmPlayer.setLooping(true);
            bgmPlayer.setVolume(0.75f, 0.75f);
            bgmPlayer.prepare();
            Log.d(TAG, "BGM OGG Darksynth 2026 preparado exitosamente.");
        } catch (Exception e) {
            Log.w(TAG, "Error preparando BGM OGG: " + e.getMessage());
        }
    }

    public void startBgm() {
        if (bgmPlayer != null && !bgmPlayer.isPlaying()) {
            try {
                bgmPlayer.start();
            } catch (Exception ignored) {}
        }
    }

    public void pauseBgm() {
        if (bgmPlayer != null && bgmPlayer.isPlaying()) {
            try {
                bgmPlayer.pause();
            } catch (Exception ignored) {}
        }
    }

    public void playLaser() {
        if (soundPool != null && sfxLaser != -1) {
            soundPool.play(sfxLaser, 0.8f, 0.8f, 1, 0, 1.0f);
        }
    }

    public void playExplosion() {
        if (soundPool != null && sfxExplosion != -1) {
            soundPool.play(sfxExplosion, 1.0f, 1.0f, 2, 0, 1.0f);
        }
    }

    public void playBomb() {
        if (soundPool != null && sfxBomb != -1) {
            soundPool.play(sfxBomb, 1.0f, 1.0f, 3, 0, 1.0f);
        }
    }

    public void playShieldHit() {
        if (soundPool != null && sfxShieldHit != -1) {
            soundPool.play(sfxShieldHit, 0.7f, 0.7f, 1, 0, 1.0f);
        }
    }

    public void playBossAlarm() {
        if (soundPool != null && sfxBossAlarm != -1) {
            soundPool.play(sfxBossAlarm, 0.9f, 0.9f, 2, 0, 1.0f);
        }
    }

    public void release() {
        if (bgmPlayer != null) {
            try {
                bgmPlayer.stop();
                bgmPlayer.release();
            } catch (Exception ignored) {}
            bgmPlayer = null;
        }
        if (soundPool != null) {
            soundPool.release();
            soundPool = null;
        }
    }
}
