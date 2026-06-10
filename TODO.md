# Blackhole Simulation — TODO

Physics-accurate. Real formulas. No aesthetic shortcuts.

---

## Phase 1 — Coordinate System
- [ ] Remove gradient code, replace with solid black screen
- [ ] Convert pixel (x, y) to normalized screen coords (-1.0 to 1.0), centered at screen middle
- [ ] Map screen coords to physical coords using camera distance + FOV
- [ ] Define constants: `M` (mass), `RS` (Schwarzschild radius = 2M in geometric units G=c=1)

---

## Phase 2 — Event Horizon
- [ ] Compute `r` = distance from each pixel ray to BH center
- [ ] If `r < RS` → pixel = `0xFF000000` (pure black)
- [ ] Confirm: black circle visible on screen

---

## Phase 3 — Background Starfield
- [ ] Generate fixed star positions at startup (seeded RNG, not per-frame)
- [ ] Map ray direction → star lookup (equirectangular or cubemap)
- [ ] Far rays sample undeflected background for now

---

## Phase 4 — Null Geodesic Ray Marching (Core Physics)
- [ ] Understand the geodesic ODE: `d²u/dφ² + u = (3/2) * rs * u²` where `u = 1/r`
- [ ] Implement numerical integrator (RK4 recommended) for ray path
- [ ] Compute impact parameter `b = r * sin(θ)` per ray
- [ ] Check critical impact parameter `b_crit = (3√3/2) * rs` — captured rays → black
- [ ] Trace each ray until: hits event horizon, hits disk plane, or escapes

---

## Phase 5 — Accretion Disk Geometry
- [ ] Define disk as flat ring in equatorial plane: `r ∈ [3*RS, 12*RS]` (ISCO to outer edge)
- [ ] Check if ray crosses disk plane (z=0 crossing during march)
- [ ] Record intersection point `r_hit`

---

## Phase 6 — Disk Temperature & Color
- [ ] Implement temperature profile: `T(r) = T_max * (r_inner/r)^(3/4) * (1 - sqrt(r_inner/r))^(1/4)`
- [ ] ISCO at `r = 3*RS` = hottest (~10⁷ K, blue-white)
- [ ] Outer edge = cooler (~10⁵ K, orange-red)
- [ ] Approximate blackbody spectrum → RGB mapping

---

## Phase 7 — Relativistic Doppler & Beaming
- [ ] Compute Keplerian orbital velocity: `v = sqrt(GM/r)` at disk hit point
- [ ] Compute Doppler factor: `D = 1 / (γ * (1 - β*cos(ψ)))`
- [ ] Apply beaming: `I_obs = D⁴ * I_emitted`
- [ ] One side of disk brighter (approaching) + bluer; other side dimmer + redder

---

## Phase 8 — Gravitational Redshift
- [ ] Apply redshift factor to all photons: `g = sqrt(1 - rs/r)`
- [ ] Photons from near BH lose energy climbing gravity well → dimmer + redder
- [ ] Multiply final pixel color by `g`

---

## Phase 9 — Photon Sphere Glow
- [ ] Rays with `b ≈ b_crit` loop many times → accumulate disk light
- [ ] Count orbits during ray march → add glow proportional to loop count
- [ ] Thin bright ring at `r = 1.5 * RS`

---

## Phase 10 — Animation
- [ ] Auto-increment `frame_count` each frame (remove keypress requirement)
- [ ] Animate disk: rotate texture/color pattern around azimuthal angle over time
- [ ] Smooth 60fps confirmed

---

## Phase 11 — Polish
- [ ] Bloom effect: bright pixels bleed into neighbors (convolution or simple blur)
- [ ] Tone mapping: HDR → displayable range (use Reinhard or ACES)
- [ ] Tweak `M`, camera FOV, disk bounds until render matches real BH images

---

## Stretch Goals
- [ ] Tilted camera angle (disk at ~15° tilt like Interstellar)
- [ ] Secondary photon ring (light that orbited once before escaping)
- [ ] Kerr metric — rotating blackhole (frame dragging, ergosphere)
- [ ] Real-time parameter tuning via keyboard (mass, camera distance, disk tilt)
