# ADR 0001: Windowing/Presentation Stack

Status: Accepted
Date: 2025-08-26

Context
- We need a portable, Rust-native way to open a window and present a 240×136 framebuffer with predictable scaling.

Decision
- Use `winit` for window/events and `pixels` for presenting an RGBA buffer (wgpu-backed) with integer nearest-neighbor scaling.

Consequences
- Desktop-first with a path to Web via wgpu. No SDL runtime dependency.
- Keep an optional SDL2 backend as a future alternative if needed.

