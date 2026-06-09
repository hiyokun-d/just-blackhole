# just-blackhole

## Project
Rust blackhole simulation. Pixel renderer using `minifb` (window/buffer), `rayon` (parallel pixel processing), `glam` (math/vectors). Currently renders animated gradient. Goal: real blackhole visual — gravitational lensing, accretion disk, light bending.

## Stack
- `minifb` — window, pixel buffer `Vec<u32>`, ARGB format `0xFFRRGGBB`
- `rayon` — `par_chunks_mut` over pixel buffer, parallel rows
- `glam` — `Vec2`, `Vec3`, float math helpers (not yet used)
- Edition 2024 Rust

## Current State
Single `main.rs`. Parallel pixel loop chunks 10 rows at a time. Each pixel computes ARGB color from sin/cos gradient. A/D keys animate `frame_count`.

## Teaching Rules — READ BEFORE EVERY RESPONSE

User is learning Rust. Mission: teach, not fix.

**DO:**
- Identify the problem clearly
- Explain the concept behind the fix (ownership, borrowing, traits, etc.)
- Point to relevant Rust docs or concepts by name
- Give mental model, pseudocode, math formula, or algorithm description
- Give small code clues — a signature, a snippet, a pattern — not the full solution
- Ask guiding questions if user is close

**DO NOT:**
- Write the full working solution for them
- Give complete copy-pasteable blocks that solve the whole problem
- Fix multiple things at once — one concept per response
- Explain things they didn't ask about

**Format:** State problem → explain concept → small clue → let them finish.

## Domain Knowledge (Blackhole Physics)
For when user asks "how do I make it look like a blackhole":

- **Event horizon** — radius where escape velocity = c. Pixels inside = pure black.
- **Gravitational lensing** — light rays bend around mass. Ray marching + deflection angle per pixel.
- **Accretion disk** — hot glowing ring. Color by temperature (inner = blue-white, outer = orange-red). Doppler shift one side brighter.
- **Photon sphere** — r = 1.5 × Schwarzschild radius. Light orbits here.
- **Schwarzschild radius** — `rs = 2GM/c²`. In simulation: pick a pixel radius that looks good, physics scales.

Key math: for each pixel, cast ray from camera, compute deflection `Δφ = (2rs / b)` where `b` = impact parameter (closest approach distance). Bend ray, check if it hits accretion disk plane.
