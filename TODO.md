# Blackhole Simulation — TODO

Physics-accurate. Real formulas. No aesthetic shortcuts. Modular structure.

---

## Phase 1 — Coordinate System
- [x] Remove gradient code, replace with solid black screen
- [x] Convert pixel (x, y) to normalized screen coords (-1.0 to 1.0), centered at screen middle
- [x] Map screen coords to physical coords using camera distance + FOV
- [x] Define constants: `M` (mass), `RS` (Schwarzschild radius = 2M in geometric units G=c=1)

---

## Phase 2 — Event Horizon
- [x] Compute `r` = distance from each pixel ray to BH center
- [x] If `r < RS` → pixel = `0xFF000000` (pure black)
- [x] Confirm: black circle visible on screen

---

## Phase 3 — Background Starfield
- [x] Add `rand` crate to `Cargo.toml`
- [x] Generate fixed star positions at startup (seeded RNG, not per-frame)
- [x] Map ray direction → star lookup
- [ ] Far rays sample undeflected background for now
- [ ] Stars move over time — animate position using `frame_count` as time offset (parallax or slow drift)

---

## Phase 4 — Null Geodesic Ray Marching (Core Physics)
- [x] Understand the geodesic ODE: `d²u/dφ² + u = (3/2) * rs * u²` where `u = 1/r`
- [x] Implement numerical integrator (RK4 recommended) for ray path — `deriv` + `geodesic_step` done
- [x] Implement `trace_ray(u0, w0) -> Option<f32>` — march ray until captured or escaped
- [x] Compute impact parameter `b = r * sin(θ)` per ray
- [x] Check critical impact parameter `b_crit = (3√3/2) * rs` — captured rays → black
- [x] Trace each ray until: hits event horizon or escapes
- [x] Wire `trace_ray` into pixel loop — replaced `is_horizon` check
- [x] Gravitational lensing — deflect star lookup using `phi - PI` rotation

---

## Phase 5 — Accretion Disk Geometry
- [x] Define disk constants: `DISK_INNER = 3*RS`, `DISK_OUTER = 12*RS`
- [x] `RayResult::HitDisk(r)` enum variant added
- [x] 3D ray-plane intersection — disk visible from angled camera
- [x] Camera moved above disk plane (`y = 3.0`) — disk ring visible on screen

---

## Phase 6 — Disk Temperature & Color
- [x] Implement temperature profile: `T(r) = T_max * (r_inner/r)^(3/4) * (1 - sqrt(r_inner/r))^(1/4)`
- [x] `disk_color(r)` function written — maps temperature to orange→white RGB
- [x] Wired into `HitDisk` match arm — ready, waiting for 3D intersection to activate

---

## Phase 7 — Relativistic Doppler & Beaming
- [x] Compute Keplerian orbital velocity: `v = sqrt(GM/r)` at disk hit point
- [x] Compute Doppler factor: `D = 1 / (γ * (1 - β*cos(ψ)))`
- [x] Apply beaming: `I_obs = D⁴ * I_emitted`
- [x] One side of disk brighter (approaching) + bluer; other side dimmer + redder

---

## Phase 8 — Gravitational Redshift
- [x] Apply redshift factor to all photons: `g = sqrt(1 - rs/r)`
- [x] Photons from near BH lose energy climbing gravity well → dimmer + redder
- [x] Multiply final pixel color by `g`

---

## Phase 9 — Photon Sphere Glow
- [x] Rays with `b ≈ b_crit` loop many times → accumulate disk light
- [x] Count orbits during ray march → add glow proportional to loop count
- [x] Thin bright ring at `r = 1.5 * RS`

---

## Phase 10 — Animation (Interstellar-style live motion)
- [x] Auto-increment time each frame using `Instant` (smooth, not frame-count based)
- [x] Animate disk: rotate color/brightness pattern around azimuthal angle over time
- [x] Pre-compute ray paths once at startup — render loop only does color math (smooth 60fps)
- [x] Smooth 60fps confirmed
- [x] Animate stars: drift via cam_angle rotation on bent sky coords each frame
- [x] Camera drift: star field rotates over time (cam_angle = time * 0.04)

---

## Phase 11 — Visual Polish (Interstellar look)
- [x] Bloom effect: separable box blur bloom, radius 14, Reinhard tone map
- [x] Tone mapping: HDR → displayable range (Reinhard: `c / (c + 1.0)`)
- [x] Warmer color palette: gold-white Interstellar palette, disk HDR values 4× for bright bloom
- [x] Disk texture: multi-octave sine (3 harmonics) for filament/turbulence look
- [x] Lensed back disk: `orbits == 1` → `LensedDisk` cache variant → arc above shadow at 45% brightness
- [x] Secondary image: rays that orbited once show dim disk color creating inner ring above shadow
- [x] 1000 stars via texture, O(1) lookup, drift with cam_angle = time * 0.04
- [x] Look-at camera centered on BH, elevation ~17°
- [ ] Disk thickness: soft vertical falloff based on ray angle
- [ ] Camera tilt ~15° like Interstellar

---

## Modularization — Split `main.rs` into modules (do after Phase 5)

> Do this AFTER core physics works. Don't split empty rooms.

- [ ] `src/camera.rs` — ray generation, FOV, screen-to-world coord transform
- [ ] `src/physics.rs` — geodesic ODE integrator (RK4), impact parameter, deflection
- [ ] `src/disk.rs` — accretion disk intersection, temperature profile, blackbody color
- [ ] `src/starfield.rs` — star generation, star lookup by ray direction
- [ ] `src/render.rs` — pixel color assembly, tone mapping, bloom
- [ ] `src/constants.rs` — all physical constants (`M`, `RS`, `CAM_DIST`, `ISCO`, etc.)
- [ ] Wire all modules back in `main.rs` with `mod` declarations

---

## Camera Controls
- [x] WASD — orbit camera around BH (A/D = yaw, W/S = pitch)
- [x] Mouse drag — left-click drag for yaw/pitch orbit (120ms throttled cache rebuild)
- [x] Scroll wheel — zoom in/out (cam_dist clamped RS*3..80)
- [x] Camera state: cam_dist + cam_pitch + cam_yaw as mutable vars
- [x] Default view at cam_dist=40 — full Gargantua view (disk oval + shadow visible)
- [x] Lensed arc b-limit — b < 1.8*b_crit prevents gold background at close distances

---

## Stretch Goals
- [ ] Tilted camera angle (disk at ~15° tilt like Interstellar)
- [ ] Secondary photon ring (light that orbited once before escaping)
- [ ] Kerr metric — rotating blackhole (frame dragging, ergosphere)
- [ ] Real-time parameter tuning via keyboard (mass, camera distance, disk tilt)

## 3D Upgrade (after modularization)
- [ ] Build `camera.rs` with full 3D ray generation (position, direction, FOV)
- [ ] Rays become 3D vectors, disk becomes a real tilted plane in 3D space
- [ ] BH is a sphere not a circle
- [ ] Tilted camera view — see disk top directly, disk bottom lensed over BH (Interstellar look)
- [ ] Star field in 3D — sample background sphere not flat plane
