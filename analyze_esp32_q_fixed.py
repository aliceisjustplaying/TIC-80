#!/usr/bin/env python3
# Analyze effective Q factor in ESP32 approach - FIXED

import math

fs = 44100  # Sample rate
N = 4096    # FFT size
fmin = 20   # Minimum frequency

print("ESP32 CQT Analysis (Corrected)")
print("==============================")
print(f"FFT size: {N}, Sample rate: {fs} Hz, Min freq: {fmin} Hz")
print()

# Test frequencies spanning 10 octaves
freqs = [20, 27.5, 40, 55, 80, 110, 160, 220, 440, 880, 1760, 3520, 7040, 14080, 20000]

print("Freq(Hz) | Factor | Window | Duration(ms) | Eff. Q | BW(Hz) | BW(cents)")
print("---------|--------|--------|--------------|--------|--------|----------")

for f in freqs:
    # ESP32 window calculation: N_window = N / (f/fmin)
    factor = f / fmin
    window_length = N / factor  # Keep as float for accurate Q
    window_length_int = int(window_length)
    
    # Window duration in milliseconds
    duration_ms = window_length / fs * 1000
    
    # Effective Q = window_length * f / fs
    eff_q = window_length * f / fs
    
    # Bandwidth = f / Q
    bandwidth = f / eff_q
    
    # Bandwidth in cents (1 semitone = 100 cents)
    # cents = 1200 * log2(f2/f1) where f2 = f + bandwidth/2, f1 = f - bandwidth/2
    bw_cents = 1200 * math.log2((f + bandwidth/2) / (f - bandwidth/2))
    
    print(f"{f:7.1f} | {factor:6.1f} | {window_length_int:6d} | {duration_ms:12.1f} | "
          f"{eff_q:6.1f} | {bandwidth:6.1f} | {bw_cents:8.0f}")

print()
print("Key Findings:")
print("-------------")
print("1. ESP32 method gives CONSTANT Q ≈ 1.86 for all frequencies!")
print("2. This is about 9x lower than ideal Q ≈ 17 for 12 bins/octave")
print("3. Bandwidth is constant at ~7.5 semitones (should be ~0.83 semitones)")
print()
print("Trade-offs:")
print("- PRO: All windows fit in FFT size, simple implementation")
print("- PRO: No truncation artifacts")  
print("- CON: Poor frequency resolution (9x worse than ideal)")
print("- CON: Can't distinguish adjacent notes (bandwidth > 1 semitone)")
print()
print("For electronic music with 20Hz start:")
print(f"- Window at 20Hz: {N/1:.0f} samples = {N/1/fs*1000:.1f}ms")
print(f"- Window at 20kHz: {N/1000:.0f} samples = {N/1000/fs*1000:.1f}ms")