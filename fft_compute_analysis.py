#!/usr/bin/env python3
# Analyze computational cost of different FFT sizes

import math

print("FFT Computational Cost Analysis")
print("===============================")
print()

# FFT complexity is O(N log N)
sizes = [4096, 8192, 16384, 32768, 65536]
base_size = 4096

print("FFT Size | Ops (N log N) | Relative Cost | Time @ 1GHz")
print("---------|---------------|---------------|-------------")

for N in sizes:
    ops = N * math.log2(N)
    relative = ops / (base_size * math.log2(base_size))
    # Assume 10 clock cycles per complex multiply-add
    # Modern CPUs can do ~1 operation per clock with SIMD
    time_ms = (ops * 10) / (1e9) * 1000  # milliseconds at 1GHz
    
    print(f"{N:7d}  | {ops:13.0f} | {relative:13.1f}x | {time_ms:10.2f}ms")

print()
print("Real-world estimates (with KISS FFT, no SIMD):")
print("- 4096-point: ~1-2ms on modern CPU")
print("- 16384-point: ~5-10ms")
print("- 32768-point: ~12-24ms")
print()
print("For 60 FPS: 16.7ms per frame total")
print("For 30 FPS: 33.3ms per frame total")
print()

# Memory usage
print("Memory Requirements:")
print("Size  | Samples | Real FFT Output | Kernels (120 bins)")
print("------|---------|-----------------|-------------------")
for N in sizes:
    samples = N * 4  # float32
    fft_out = (N//2 + 1) * 8  # complex float32
    # Assume 30% sparsity for kernels
    kernels = 120 * (N//2 + 1) * 8 * 0.3
    total = (samples + fft_out + kernels) / 1024
    print(f"{N:5d} | {samples/1024:7.0f}K | {fft_out/1024:15.0f}K | {kernels/1024:17.0f}K (Total: {total:.0f}K)")