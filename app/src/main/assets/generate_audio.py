#!/usr/bin/env python3
import math
import os
import struct
import subprocess
import wave

SAMPLE_RATE = 44100

def create_wav(filename, samples):
    with wave.open(filename, 'w') as wav_file:
        wav_file.setnchannels(1)  # Mono
        wav_file.setsampwidth(2)  # 16-bit
        wav_file.setframerate(SAMPLE_RATE)
        # Convert float samples (-1.0 to 1.0) to 16-bit PCM
        raw_data = bytearray()
        for s in samples:
            val = max(-1.0, min(1.0, s))
            int_val = int(val * 32767.0)
            raw_data.extend(struct.pack('<h', int_val))
        wav_file.writeframes(raw_data)

def convert_to_ogg(wav_path, ogg_path):
    subprocess.run(['ffmpeg', '-y', '-i', wav_path, '-c:a', 'libvorbis', '-q:a', '5', ogg_path],
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    if os.path.exists(wav_path):
        os.remove(wav_path)

def generate_laser():
    duration = 0.22
    total_samples = int(duration * SAMPLE_RATE)
    samples = []
    phase = 0.0
    for i in range(total_samples):
        t = i / SAMPLE_RATE
        # Exponential pitch drop from 1200 Hz to 90 Hz
        freq = 1200.0 * math.exp(-18.0 * t) + 90.0
        phase += 2.0 * math.pi * freq / SAMPLE_RATE
        # Punchy saw + sine wave with snappy decay envelope
        env = (1.0 - t / duration) ** 2.2
        # Transient click at start
        click = 0.6 * math.sin(2.0 * math.pi * 3200.0 * t) * math.exp(-150.0 * t)
        sample = (0.7 * math.sin(phase) + 0.3 * (2.0 * (phase / (2.0 * math.pi) % 1.0) - 1.0) + click) * env
        samples.append(sample * 0.9)
    return samples

def generate_explosion():
    duration = 0.85
    total_samples = int(duration * SAMPLE_RATE)
    samples = []
    # Seeded pseudo-random noise generator
    rand_state = 12345
    def next_rand():
        nonlocal rand_state
        rand_state = (rand_state * 1103515245 + 12345) & 0x7FFFFFFF
        return (rand_state / 0x7FFFFFFF) * 2.0 - 1.0

    low_pass = 0.0
    for i in range(total_samples):
        t = i / SAMPLE_RATE
        raw_noise = next_rand()
        # Dynamic low-pass filter sweeping down
        cutoff = max(0.01, 0.45 * math.exp(-4.5 * t))
        low_pass += cutoff * (raw_noise - low_pass)
        # Deep sub-bass boom (55 Hz to 28 Hz)
        sub_phase = 2.0 * math.pi * (55.0 - 27.0 * (t / duration)) * t
        sub_boom = math.sin(sub_phase) * math.exp(-3.5 * t) * 0.85
        env = (1.0 - t / duration) ** 1.8
        sample = (low_pass * 0.8 + sub_boom) * env
        samples.append(sample * 0.95)
    return samples

def generate_emp_bomb():
    duration = 1.3
    total_samples = int(duration * SAMPLE_RATE)
    samples = []
    phase = 0.0
    for i in range(total_samples):
        t = i / SAMPLE_RATE
        if t < 0.25:
            # High-tech charge up
            freq = 200.0 + 800.0 * (t / 0.25) ** 2
            env = t / 0.25
            phase += 2.0 * math.pi * freq / SAMPLE_RATE
            sample = math.sin(phase) * env * 0.5
        else:
            # Massive EMP discharge and bass drop
            dt = t - 0.25
            freq = max(35.0, 160.0 * math.exp(-6.0 * dt))
            phase += 2.0 * math.pi * freq / SAMPLE_RATE
            env = (1.0 - dt / 1.05) ** 2.0
            shimmer = math.sin(phase * 4.0) * 0.2 * math.exp(-4.0 * dt)
            sample = (math.sin(phase) * 0.8 + shimmer) * env
        samples.append(sample * 0.95)
    return samples

def generate_shield_hit():
    duration = 0.25
    total_samples = int(duration * SAMPLE_RATE)
    samples = []
    phase1 = 0.0
    phase2 = 0.0
    for i in range(total_samples):
        t = i / SAMPLE_RATE
        phase1 += 2.0 * math.pi * 580.0 / SAMPLE_RATE
        phase2 += 2.0 * math.pi * 1160.0 / SAMPLE_RATE
        env = math.exp(-14.0 * t)
        sample = (0.7 * math.sin(phase1) + 0.3 * math.sin(phase2)) * env
        samples.append(sample * 0.8)
    return samples

def generate_boss_alarm():
    duration = 0.9
    total_samples = int(duration * SAMPLE_RATE)
    samples = []
    phase = 0.0
    for i in range(total_samples):
        t = i / SAMPLE_RATE
        # Two-tone urgent pulse (660 Hz and 880 Hz)
        freq = 880.0 if (int(t * 8.0) % 2 == 0) else 660.0
        phase += 2.0 * math.pi * freq / SAMPLE_RATE
        env = 0.8 * (0.8 + 0.2 * math.sin(2.0 * math.pi * 16.0 * t))
        sample = math.sin(phase) * env
        samples.append(sample * 0.85)
    return samples

def generate_synth_bgm():
    # 4-measure looping darksynth cyberpunk groove (approx 7.5 seconds)
    bpm = 128.0
    beat_sec = 60.0 / bpm
    total_duration = beat_sec * 16.0 # 16 beats = 4 bars
    total_samples = int(total_duration * SAMPLE_RATE)
    samples = [0.0] * total_samples

    # Bassline notes (Frequencies in Hz): C2 (65.4), D#2 (77.8), F2 (87.3), G2 (98.0)
    bass_notes = [65.4, 65.4, 77.8, 65.4, 87.3, 87.3, 98.0, 77.8]
    step_duration = beat_sec / 2.0 # 8th notes

    for step_idx in range(32): # 32 eighth notes
        freq = bass_notes[step_idx % len(bass_notes)]
        start_sample = int(step_idx * step_duration * SAMPLE_RATE)
        step_len = int(step_duration * SAMPLE_RATE)
        phase = 0.0
        for i in range(step_len):
            idx = start_sample + i
            if idx >= total_samples:
                break
            t = i / SAMPLE_RATE
            phase += 2.0 * math.pi * freq / SAMPLE_RATE
            env = math.exp(-7.0 * t)
            # Rich saw bass
            saw = (phase / math.pi % 2.0) - 1.0
            samples[idx] += saw * env * 0.35

    # Kick drum on every beat (4-on-the-floor)
    for beat in range(16):
        start_sample = int(beat * beat_sec * SAMPLE_RATE)
        kick_len = int(0.18 * SAMPLE_RATE)
        for i in range(kick_len):
            idx = start_sample + i
            if idx >= total_samples:
                break
            t = i / SAMPLE_RATE
            freq = 140.0 * math.exp(-28.0 * t) + 40.0
            phase = 2.0 * math.pi * freq * t
            env = (1.0 - t / 0.18) ** 2.0
            samples[idx] += math.sin(phase) * env * 0.55

    # Snare / Clap on beats 2, 4, 6, 8...
    for beat in range(1, 16, 2):
        start_sample = int(beat * beat_sec * SAMPLE_RATE)
        snare_len = int(0.20 * SAMPLE_RATE)
        rand_s = 54321
        for i in range(snare_len):
            idx = start_sample + i
            if idx >= total_samples:
                break
            t = i / SAMPLE_RATE
            rand_s = (rand_s * 1103515245 + 12345) & 0x7FFFFFFF
            noise = (rand_s / 0x7FFFFFFF) * 2.0 - 1.0
            env = math.exp(-16.0 * t)
            samples[idx] += noise * env * 0.35

    # Normalize samples
    max_amp = max(abs(s) for s in samples) or 1.0
    return [s / max_amp * 0.92 for s in samples]

def main():
    out_dir = os.path.join(os.path.dirname(__file__), 'audio')
    os.makedirs(out_dir, exist_ok=True)
    tmp_dir = '/tmp/game_audio_tmp'
    os.makedirs(tmp_dir, exist_ok=True)

    tracks = [
        ('laser_plasma', generate_laser()),
        ('explosion_heavy', generate_explosion()),
        ('emp_bomb', generate_emp_bomb()),
        ('shield_hit', generate_shield_hit()),
        ('boss_alarm', generate_boss_alarm()),
        ('bgm_cyber_rebellion', generate_synth_bgm()),
    ]

    for name, samples in tracks:
        wav_path = os.path.join(tmp_dir, f'{name}.wav')
        ogg_path = os.path.join(out_dir, f'{name}.ogg')
        create_wav(wav_path, samples)
        convert_to_ogg(wav_path, ogg_path)
        print(f'✅ Generado: {ogg_path} ({os.path.getsize(ogg_path)} bytes)')

if __name__ == '__main__':
    main()
