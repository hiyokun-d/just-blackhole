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

- ALWAYS read the code to make sure that we on the right track
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

## Maintenance Rules — ALWAYS DO

- After every task completion, update `TODO.md` — check off the finished item
- If new work is discovered mid-session, add it to `TODO.md` immediately

## Physics Accuracy Goal

This simulation must be **mathematically and physically accurate**. No shortcuts for aesthetics — use real formulas.

- Use **Schwarzschild metric** for a non-rotating blackhole
- For rotating blackhole: **Kerr metric** (stretch goal)
- All formulas must match GR (General Relativity) derivations

---

## Domain Knowledge — Real Physics Formulas

### Schwarzschild Radius

```
rs = 2GM / c²
```

In simulation: set `rs` in "geometric units" where G=1, c=1. Then `rs = 2M`.

### Event Horizon

Pixels where `r < rs` → pure black. No light escapes.

### Photon Sphere

```
r_photon = (3/2) * rs
```

Unstable circular orbit for photons. Faint glow ring here.

### Impact Parameter

```
b = r * sin(θ)
```

where `r` = ray origin distance from BH, `θ` = angle between ray and radial direction.
Critical impact parameter: `b_crit = (3√3 / 2) * rs` — rays with `b < b_crit` are captured.

### Gravitational Lensing — Deflection Angle

Weak field (far from BH):

```
Δφ = 2rs / b     (first-order approximation)
```

Strong field (near BH) — use numerical integration of the null geodesic equation:

```
d²u/dφ² + u = (3/2) * rs * u²
```

where `u = 1/r`. Integrate this ODE per ray for accuracy.

### Ray Marching (Null Geodesic)

For each pixel:

1. Compute ray direction from camera
2. Numerically integrate geodesic equations in Schwarzschild spacetime
3. Step ray forward; at each step check:
   - Did ray cross `r < rs`? → black
   - Did ray cross accretion disk plane? → disk color
   - Did ray escape to infinity? → background/stars

### Accretion Disk Temperature Profile

```
T(r) = T_max * (r_inner / r)^(3/4) * (1 - sqrt(r_inner / r))^(1/4)
```

Map temperature → blackbody RGB using Planck's law approximation.

- `r = 3rs` (ISCO — innermost stable circular orbit) → hottest point
- Convert temperature in Kelvin to RGB via blackbody spectrum

### Doppler / Relativistic Beaming

Disk orbits at Keplerian velocity:

```
v = sqrt(GM / r)
```

Doppler factor:

```
D = 1 / (γ * (1 - β·cos(ψ)))
```

where `β = v/c`, `γ = 1/sqrt(1-β²)`, `ψ` = angle between velocity and line of sight.
Observed intensity: `I_obs = D⁴ * I_emitted`

### Redshift Factor

```
g = sqrt(1 - rs/r)
```

Photons climbing out of gravity well lose energy. Multiply final color by `g`.

---

## Coordinate System Convention

- Origin at blackhole center
- `r` = radial distance, `θ` = polar angle, `φ` = azimuthal angle
- Camera at large `r`, looking toward origin
- Normalize screen coords to [-1, 1] before any physics math
