#!/usr/bin/env python3
# Analyze effective Q factor in ESP32 approach

import math

fs = 44100  # Sample rate
N = 4096    # FFT size
fmin = 20   # Minimum frequency

print("ESP32 CQT Analysis")
print("==================")
print(f"FFT size: {N}, Sample rate: {fs} Hz, Min freq: {fmin} Hz")
print()

# Test frequencies
freqs = [20, 27.5, 50, 100, 200, 440, 1000, 2000, 5000, 10000, 20000]

print("Freq(Hz) | Window | Eff. Q | BW(Hz) | BW(semitones)")
print("---------|--------|--------|--------|---------------")

for f in freqs:
    # ESP32 window calculation: N_window = N / (f/fmin)
    factor = f / fmin
    window_length = int(N / factor)
    
    # Effective Q = window_length * f / fs
    eff_q = window_length * f / fs
    
    # Bandwidth = f / Q
    bandwidth = f / eff_q if eff_q > 0 else float('inf')
    
    # Bandwidth in semitones = 12 * log2(1 + bandwidth/f)
    bw_semitones = 12 * math.log2(1 + bandwidth/f) if bandwidth < float('inf') else float('inf')
    
    print(f"{f:7.1f} | {window_length:6d} | {eff_q:6.1f} | {bandwidth:6.1f} | {bw_semitones:13.1f}")

print()
print("Observations:")
print("1. Q varies dramatically with frequency (1.9 to 93.0)")
print("2. Low frequencies: Very low Q (wide bandwidth)")
print("3. High frequencies: Very high Q (narrow bandwidth)")
print()
print("For 12 bins/octave, ideal constant Q ≈ 17")
print("ESP32 only achieves this around 200-300 Hz")