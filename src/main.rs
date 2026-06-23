use std::f32::consts::PI;
use std::time::{Duration, Instant};

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use rand::random_range;
use rayon::prelude::*;
use glam::Vec3;

const SCREEN_WIDTH:   usize = 800;
const SCREEN_HEIGHT:  usize = 600;
const M:              f32   = 1.0;
const RS:             f32   = 2.0 * M;
const CAM_DIST:       f32   = 40.0;
const DISK_INNER:     f32   = 3.0 * RS;
const DISK_OUTER:     f32   = 12.0 * RS;
const T_MAX:          f32   = 1.0;
const PHOTON_SPHERE:  f32   = 1.5 * RS;
const BLOOM_RADIUS:   usize = 10;
const BLOOM_STRENGTH: f32   = 0.25;
const BLOOM_THRESH:   f32   = 0.72;

enum PixelCache {
    Black,
    Background { u_bent: f32, v_bent: f32 },
    PhotonGlow(u32),
    Disk       { r: f32, hit: Vec3, ray_dir: Vec3, angle: f32 },
    LensedDisk { r: f32, hit: Vec3, ray_dir: Vec3, angle: f32 },
}

enum RayResult {
    Captured,
    Escaped(f32, u32),
}

fn deriv(u: f32, w: f32) -> (f32, f32) {
    (w, (3.0 / 2.0) * RS * u.powi(2) - u)
}

fn geodesic_step(u: f32, w: f32, dphi: f32) -> (f32, f32) {
    let k1 = deriv(u, w);
    let k2 = deriv(u + k1.0 * dphi / 2.0, w + k1.1 * dphi / 2.0);
    let k3 = deriv(u + k2.0 * dphi / 2.0, w + k2.1 * dphi / 2.0);
    let k4 = deriv(u + k3.0 * dphi,       w + k3.1 * dphi);
    (
        u + (k1.0 + 2.0*k2.0 + 2.0*k3.0 + k4.0) * dphi / 6.0,
        w + (k1.1 + 2.0*k2.1 + 2.0*k3.1 + k4.1) * dphi / 6.0,
    )
}

fn trace_ray(mut u0: f32, mut w0: f32) -> RayResult {
    let mut phi    = 0.0f32;
    let mut orbits = 0u32;
    let mut prev_r = 1.0 / u0;

    for _ in 0..600 {
        (u0, w0) = geodesic_step(u0, w0, 0.01);
        phi += 0.01;
        let r = 1.0 / u0;
        if (prev_r > PHOTON_SPHERE && r < PHOTON_SPHERE)
        || (prev_r < PHOTON_SPHERE && r > PHOTON_SPHERE) {
            orbits += 1;
        }
        prev_r = r;
        if u0 > 1.0 / RS   { return RayResult::Captured; }
        if u0 < 0.0001     { return RayResult::Escaped(phi, orbits); }
    }
    RayResult::Escaped(phi, orbits)
}

fn doppler_factor(r: f32, hit: Vec3, ray_dir: Vec3) -> f32 {
    let v       = (M / r).sqrt();
    let v_dir   = Vec3::new(-hit.z, 0.0, hit.x).normalize();
    let cos_psi = v_dir.dot(-ray_dir);
    let gamma   = 1.0 / (1.0 - v * v).sqrt();
    1.0 / (gamma * (1.0 - v * cos_psi))
}

fn disk_hdr(r: f32, hit: Vec3, ray_dir: Vec3, spin: f32, angle: f32) -> [f32; 3] {
    let t_raw = T_MAX * (DISK_INNER / r).powf(0.75) * (1.0 - (DISK_INNER / r).sqrt()).powf(0.25);
    let t     = (t_raw * (1.0 + 0.25 * (angle * 4.0).sin())).clamp(0.0, 1.0);
    let d4    = doppler_factor(r, hit, ray_dir).powi(4).clamp(0.0, 8.0);
    let g     = (1.0 - RS / r).sqrt();
    let bri   = d4 * g * spin * 2.0;
    [
        (0.6  + 0.4 * t) * bri,
        (0.25 + 0.6 * t) * bri,
        (0.02 + 0.2 * t) * bri,
    ]
}

fn main() {
    let mut buffer:  Vec<u32>     = vec![0;       SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut hdr:     Vec<[f32;3]> = vec![[0.0;3]; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut bloom_h: Vec<[f32;3]> = vec![[0.0;3]; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut bloom_v: Vec<[f32;3]> = vec![[0.0;3]; SCREEN_WIDTH * SCREEN_HEIGHT];

    let mut window = Window::new(
        "Blackhole simulation",
        SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions::default(),
    ).unwrap_or_else(|e| panic!("{}", e));
    window.set_target_fps(60);

    const STAR_TEX: usize = 1024;
    let mut star_tex = vec![0u8; STAR_TEX * STAR_TEX];
    for _ in 0..1000 {
        let sx = random_range(-1.0_f32..1.0_f32);
        let sy = random_range(-1.0_f32..1.0_f32);
        let cx = ((sx + 1.0) * 0.5 * STAR_TEX as f32) as usize;
        let cy = ((sy + 1.0) * 0.5 * STAR_TEX as f32) as usize;
        if cx < STAR_TEX && cy < STAR_TEX {
            star_tex[cy * STAR_TEX + cx] = 255;
        }
    }

    let b_crit        = (3.0 * 3.0_f32.sqrt() / 2.0) * RS;
    let mut cam_dist  = CAM_DIST;
    let mut cam_pitch = 0.2_f32;
    let mut cam_yaw   = 0.0_f32;

    let build_cache = |cam_dist: f32, cam_pitch: f32, cam_yaw: f32| -> Vec<PixelCache> {
        let ray_origin = Vec3::new(
            cam_dist * cam_yaw.sin() * cam_pitch.cos(),
            cam_dist * cam_pitch.sin(),
            cam_dist * cam_yaw.cos() * cam_pitch.cos(),
        );
        let forward  = (-ray_origin).normalize();
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let right    = forward.cross(world_up).normalize();
        let cam_up   = right.cross(forward).normalize();
        let u0_cam   = 1.0 / ray_origin.length();

        (0..SCREEN_WIDTH * SCREEN_HEIGHT)
        .into_par_iter()
        .map(|idx| {
            let x = (idx % SCREEN_WIDTH) as f32;
            let y = (idx / SCREEN_WIDTH) as f32;

            let u = (x - SCREEN_WIDTH  as f32 / 2.0) / SCREEN_HEIGHT as f32;
            let v = (y - SCREEN_HEIGHT as f32 / 2.0) / SCREEN_HEIGHT as f32;

            let ray_dir = (forward + u * right - v * cam_up).normalize();
            let b       = ray_origin.cross(ray_dir).length();
            let w0      = 1.0 / b;

            if b < b_crit { return PixelCache::Black; }

            // Direct disk intersection
            if ray_dir.y.abs() > 0.0001 {
                let t = -ray_origin.y / ray_dir.y;
                if t > 0.0 {
                    let hit   = ray_origin + ray_dir * t;
                    let r_hit = (hit.x * hit.x + hit.z * hit.z).sqrt();
                    if r_hit >= DISK_INNER && r_hit <= DISK_OUTER {
                        let angle = hit.z.atan2(hit.x);
                        return PixelCache::Disk { r: r_hit, hit, ray_dir, angle };
                    }
                }
            }

            match trace_ray(u0_cam, w0) {
                RayResult::Captured => PixelCache::Black,
                RayResult::Escaped(phi, orbits) => {
                    if orbits >= 2 {
                        return PixelCache::PhotonGlow(orbits);
                    }

                    let r_cam        = ray_origin.length();
                    let phi_straight = 2.0 * (r_cam / b).atan();
                    let deflection   = (phi - phi_straight).max(0.0);

                    // Lensed secondary image: b < 1.8*b_crit keeps this a thin arc
                    // at all camera distances — without b limit, close cameras make
                    // the whole sky qualify (phi_straight shrinks → deflection grows)
                    if deflection > 0.9 && b < b_crit * 1.8 {
                        let n       = ray_origin.cross(ray_dir).normalize();
                        let tangent = n.cross(ray_dir);
                        let dir_def = (ray_dir * deflection.cos()
                                      + tangent   * deflection.sin()).normalize();
                        if dir_def.y.abs() > 0.0001 {
                            let t = -ray_origin.y / dir_def.y;
                            if t > 0.0 {
                                let hit   = ray_origin + dir_def * t;
                                let r_hit = (hit.x*hit.x + hit.z*hit.z).sqrt();
                                if r_hit >= DISK_INNER && r_hit <= DISK_OUTER {
                                    let angle = hit.z.atan2(hit.x);
                                    return PixelCache::LensedDisk {
                                        r: r_hit, hit, ray_dir: dir_def, angle,
                                    };
                                }
                            }
                        }
                    }

                    let alpha  = phi - PI;
                    let u_bent = u * alpha.cos() - v * alpha.sin();
                    let v_bent = u * alpha.sin() + v * alpha.cos();
                    PixelCache::Background { u_bent, v_bent }
                }
            }
        })
        .collect()
    };

    let mut pixel_cache  = build_cache(cam_dist, cam_pitch, cam_yaw);
    let mut mouse_prev: Option<(f32, f32)> = None;
    let mut last_rebuild = Instant::now();
    let rebuild_throttle = Duration::from_millis(120);
    let start            = Instant::now();

    while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
        let time = start.elapsed().as_secs_f32();

        let mut dirty = false;

        // Keyboard zoom / pitch / yaw
        if window.is_key_pressed(Key::Equal, KeyRepeat::Yes) {
            cam_dist = (cam_dist - 1.0).max(RS * 3.0); dirty = true;
        }
        if window.is_key_pressed(Key::Minus, KeyRepeat::Yes) {
            cam_dist = (cam_dist + 1.0).min(80.0); dirty = true;
        }
        if window.is_key_pressed(Key::W, KeyRepeat::Yes) {
            cam_pitch = (cam_pitch + 0.04).min(1.45); dirty = true;
        }
        if window.is_key_pressed(Key::S, KeyRepeat::Yes) {
            cam_pitch = (cam_pitch - 0.04).max(0.02); dirty = true;
        }
        if window.is_key_pressed(Key::A, KeyRepeat::Yes) {
            cam_yaw -= 0.04; dirty = true;
        }
        if window.is_key_pressed(Key::D, KeyRepeat::Yes) {
            cam_yaw += 0.04; dirty = true;
        }

        // Mouse drag — left button orbits yaw / pitch
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
                if let Some((px, py)) = mouse_prev {
                    let dx = mx - px;
                    let dy = my - py;
                    cam_yaw   += dx * 0.005;
                    cam_pitch  = (cam_pitch - dy * 0.003).clamp(0.02, 1.45);
                    if dx.abs() > 0.5 || dy.abs() > 0.5 { dirty = true; }
                }
                mouse_prev = Some((mx, my));
            }
        } else {
            mouse_prev = None;
        }

        // Scroll wheel zoom
        if let Some((_, sy)) = window.get_scroll_wheel() {
            if sy.abs() > 0.0 {
                cam_dist = (cam_dist - sy * 2.0).clamp(RS * 3.0, 80.0);
                dirty = true;
            }
        }

        // Throttled rebuild — max ~8 rebuilds/s during drag, instant on key tap
        if dirty && last_rebuild.elapsed() >= rebuild_throttle {
            pixel_cache  = build_cache(cam_dist, cam_pitch, cam_yaw);
            last_rebuild = Instant::now();
        }

        // --- Pass 1: fill HDR buffer ---
        hdr.par_iter_mut().zip(pixel_cache.par_iter()).for_each(|(hdr_px, cache)| {
            *hdr_px = match cache {
                PixelCache::Black => [0.0; 3],

                PixelCache::Background { u_bent, v_bent } => {
                    // cam_yaw offsets star rotation so stars follow the disk when orbiting
                    let cam_angle = time * 0.04 + cam_yaw;
                    let u_rot = u_bent * cam_angle.cos() - v_bent * cam_angle.sin();
                    let v_rot = u_bent * cam_angle.sin() + v_bent * cam_angle.cos();
                    let tx = ((u_rot + 1.0) * 0.5 * STAR_TEX as f32) as usize;
                    let ty = ((v_rot + 1.0) * 0.5 * STAR_TEX as f32) as usize;
                    if tx < STAR_TEX && ty < STAR_TEX && star_tex[ty * STAR_TEX + tx] > 0 {
                        [0.9, 0.9, 1.1]
                    } else {
                        [0.0; 3]
                    }
                }

                PixelCache::PhotonGlow(orbits) => {
                    let i = (*orbits as f32 * 1.5).min(4.0);
                    [i * 1.0, i * 0.88, i * 0.55]
                }

                PixelCache::Disk { r, hit, ray_dir, angle } => {
                    let ra       = angle + time * 0.25;
                    let spin     = (ra.sin()
                        + (ra * 3.0 + time * 0.08).sin() * 0.25
                        + (ra * 7.0 - time * 0.04).sin() * 0.1)
                        * 0.7 + 1.1;
                    let edge_in  = ((r - DISK_INNER) / RS).clamp(0.0, 1.0);
                    let edge_out = ((DISK_OUTER - r) / (RS * 2.0)).clamp(0.0, 1.0);
                    disk_hdr(*r, *hit, *ray_dir, spin * edge_in * edge_out, ra)
                }

                PixelCache::LensedDisk { r, hit, ray_dir, angle } => {
                    let ra       = angle + time * 0.25;
                    let spin     = (ra.sin()
                        + (ra * 3.0 + time * 0.08).sin() * 0.25) * 0.5 + 0.9;
                    let edge_in  = ((r - DISK_INNER) / RS).clamp(0.0, 1.0);
                    let edge_out = ((DISK_OUTER - r) / (RS * 2.0)).clamp(0.0, 1.0);
                    let c        = disk_hdr(*r, *hit, *ray_dir, spin * edge_in * edge_out, ra);
                    [c[0] * 0.28, c[1] * 0.28, c[2] * 0.28]
                }

            };
        });

        // --- Pass 2: bloom horizontal ---
        bloom_h.par_chunks_mut(SCREEN_WIDTH).enumerate().for_each(|(y, row)| {
            for x in 0..SCREEN_WIDTH {
                let mut acc = [0.0f32; 3];
                let mut n   = 0.0f32;
                for dx in -(BLOOM_RADIUS as i32)..=(BLOOM_RADIUS as i32) {
                    let nx = x as i32 + dx;
                    if nx >= 0 && (nx as usize) < SCREEN_WIDTH {
                        let p    = hdr[y * SCREEN_WIDTH + nx as usize];
                        let luma = 0.2126*p[0] + 0.7152*p[1] + 0.0722*p[2];
                        if luma > BLOOM_THRESH {
                            acc[0] += p[0]; acc[1] += p[1]; acc[2] += p[2];
                        }
                        n += 1.0;
                    }
                }
                row[x] = [acc[0]/n, acc[1]/n, acc[2]/n];
            }
        });

        // --- Pass 3: bloom vertical ---
        bloom_v.par_chunks_mut(SCREEN_WIDTH).enumerate().for_each(|(y, row)| {
            for x in 0..SCREEN_WIDTH {
                let mut acc = [0.0f32; 3];
                let mut n   = 0.0f32;
                for dy in -(BLOOM_RADIUS as i32)..=(BLOOM_RADIUS as i32) {
                    let ny = y as i32 + dy;
                    if ny >= 0 && (ny as usize) < SCREEN_HEIGHT {
                        let p = bloom_h[ny as usize * SCREEN_WIDTH + x];
                        acc[0] += p[0]; acc[1] += p[1]; acc[2] += p[2];
                        n += 1.0;
                    }
                }
                row[x] = [
                    acc[0] / n * BLOOM_STRENGTH,
                    acc[1] / n * BLOOM_STRENGTH,
                    acc[2] / n * BLOOM_STRENGTH,
                ];
            }
        });

        // --- Pass 4: composite + Reinhard tone map + pack ---
        buffer.par_iter_mut().enumerate().for_each(|(idx, pixel)| {
            let h  = hdr[idx];
            let bl = bloom_v[idx];
            let r  = h[0] + bl[0];
            let g  = h[1] + bl[1];
            let b  = h[2] + bl[2];
            let r  = (r / (r + 1.0) * 255.0) as u8;
            let g  = (g / (g + 1.0) * 255.0) as u8;
            let b  = (b / (b + 1.0) * 255.0) as u8;
            *pixel = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        });

        window.update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    }
}
