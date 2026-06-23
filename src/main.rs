use std::f32::consts::PI;
use std::time::Instant;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::random_range;
use rayon::prelude::*;

use glam::Vec3;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 600;

const M: f32 = 1.0;
const RS: f32 = 2.0 * M;
const CAM_DIST: f32 = 15.0;

const DISK_INNER: f32 = 3.0 * RS;
const DISK_OUTER: f32 = 12.0 * RS;
const T_MAX: f32 = 1.0;
const PHOTON_SPHERE: f32 = 1.5 * RS;

// Pre-computed per-pixel ray result — computed once at startup
enum PixelCache {
    Black,
    Star,
    PhotonGlow(u32),
    Disk { r: f32, hit: Vec3, ray_dir: Vec3, angle: f32 },
}

enum RayResult {
    Captured,
    Escaped(f32, u32),
    HitDisk(f32),
}

fn deriv(u: f32, w: f32) -> (f32, f32) {
    return (w, (3.0 / 2.0) * RS * u.powi(2) - u);
}

fn geodesic_step(u: f32, w: f32, dphi: f32) -> (f32, f32) {
    let k1 = deriv(u, w);
    let k2 = deriv(u + k1.0 * dphi / 2.0, w + k1.1 * dphi / 2.0);
    let k3 = deriv(u + k2.0 * dphi / 2.0, w + k2.1 * dphi / 2.0);
    let k4 = deriv(u + k3.0 * dphi, w + k3.1 * dphi);

    let new_u = u + (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0) * dphi / 6.0;
    let new_w = w + (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1) * dphi / 6.0;

    return (new_u, new_w);
}

fn trace_ray(mut u0: f32, mut w0: f32) -> RayResult {
    let mut phi: f32 = 0.0;
    let mut orbits: u32 = 0;
    let mut prev_r = 1.0 / u0;

    for _ in 0..600 {
        (u0, w0) = geodesic_step(u0, w0, 0.01);
        phi += 0.01;

        let r = 1.0 / u0;

        if (prev_r > PHOTON_SPHERE && r < PHOTON_SPHERE)
            || (prev_r < PHOTON_SPHERE && r > PHOTON_SPHERE)
        {
            orbits += 1;
        }

        prev_r = r;

        if u0 > 1.0 / RS {
            return RayResult::Captured;
        } else if u0 < 0.0001 {
            return RayResult::Escaped(phi, orbits);
        }
    }

    return RayResult::Escaped(phi, orbits);
}

fn doppler_factor(r: f32, hit_point: Vec3, ray_dir: Vec3) -> f32 {
    let v = (M / r).sqrt();
    let v_dir = Vec3::new(-hit_point.z, 0.0, hit_point.x).normalize();
    let x = v_dir.dot(-ray_dir);
    let gamma = 1.0 / (1.0 - v * v).sqrt();
    return 1.0 / (gamma * (1.0 - v * x));
}

fn disk_color(r: f32, hit_point: Vec3, ray_dir: Vec3, spin: f32, angle: f32) -> u32 {
    let t_compute =
        T_MAX * (DISK_INNER / r).powf(0.75) * (1.0 - (DISK_INNER / r).sqrt()).powf(0.25);
    let t = t_compute.clamp(0.0, T_MAX);
    let t_azimuth = t * (1.0 + 0.3 * (angle * 4.0).sin());

    let d = doppler_factor(r, hit_point, ray_dir);
    let d4 = d.powi(4).clamp(0.0, 4.0);

    let g = (1.0 - RS / r).sqrt();

    const MAX_COLOR: f32 = 255.0;
    let rc = (255.0 * d4 * g * spin).clamp(0.0, MAX_COLOR) as u8;
    let gc = ((100.0 + 155.0 * t_azimuth) * d4 * g * spin).clamp(0.0, MAX_COLOR) as u8;
    let bc = ((20.0 + 235.0 * t_azimuth) * d4 * g * spin).clamp(0.0, MAX_COLOR) as u8;

    return 0xFF000000 | ((rc as u32) << 16) | ((gc as u32) << 8) | (bc as u32);
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let mut window = Window::new(
        "Blackhole simulation",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);

    let stars: Vec<(f32, f32)> = (0..200)
        .map(|_| {
            let u = random_range(-0.75_f32..0.75_f32);
            let v = random_range(-0.75_f32..0.75_f32);
            (u, v)
        })
        .collect();

    let b_crit = (3.0 * 3.0_f32.sqrt() / 2.0) * RS;
    let ray_origin = Vec3::new(0.0, 8.0, CAM_DIST);

    // Pre-compute ray paths ONCE — expensive but only happens at startup
    let pixel_cache: Vec<PixelCache> = (0..SCREEN_WIDTH * SCREEN_HEIGHT)
        .into_par_iter()
        .map(|idx| {
            let x = (idx % SCREEN_WIDTH) as f32;
            let y = (idx / SCREEN_WIDTH) as f32;

            let u = (x - (SCREEN_WIDTH as f32) / 2.0) / SCREEN_HEIGHT as f32;
            let v = (y - (SCREEN_HEIGHT as f32) / 2.0) / SCREEN_HEIGHT as f32;

            let ray_dir = Vec3::new(u, -v, -1.0).normalize();
            let u0 = 1.0 / CAM_DIST;
            let b = ray_origin.cross(ray_dir).length();
            let w0 = 1.0 / b;

            if b < b_crit {
                return PixelCache::Black;
            }

            // 3D disk intersection
            if ray_dir.y.abs() > 0.0001 {
                let t = -ray_origin.y / ray_dir.y;
                if t > 0.0 {
                    let hit = ray_origin + ray_dir * t;
                    let r_hit = (hit.x * hit.x + hit.z * hit.z).sqrt();
                    if r_hit >= DISK_INNER && r_hit <= DISK_OUTER {
                        let angle = hit.z.atan2(hit.x);
                        return PixelCache::Disk { r: r_hit, hit, ray_dir, angle };
                    }
                }
            }

            match trace_ray(u0, w0) {
                RayResult::Captured => PixelCache::Black,
                RayResult::HitDisk(_) => PixelCache::Black,
                RayResult::Escaped(phi, orbits) => {
                    if orbits >= 2 {
                        return PixelCache::PhotonGlow(orbits);
                    }
                    let alpha = phi - PI;
                    let u_bent = u * alpha.cos() - v * alpha.sin();
                    let v_bent = u * alpha.sin() + v * alpha.cos();
                    let hit_star = stars.iter().any(|&(sx, sy)| {
                        ((u_bent - sx).powi(2) + (v_bent - sy).powi(2)).sqrt() < 0.001
                    });
                    if hit_star { PixelCache::Star } else { PixelCache::Black }
                }
            }
        })
        .collect();

    let start = Instant::now();

    // Render loop — only recomputes disk colors each frame, everything else is instant lookup
    while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
        let time = start.elapsed().as_secs_f32();

        buffer.par_iter_mut().zip(pixel_cache.par_iter()).for_each(|(pixel, cache)| {
            *pixel = match cache {
                PixelCache::Black => 0xFF000000,
                PixelCache::Star => 0xFFFFFFFF,
                PixelCache::PhotonGlow(orbits) => {
                    let glow = (*orbits as f32 * 80.0).clamp(0.0, 255.0) as u8;
                    let glow_b = (*orbits as f32 * 120.0).clamp(0.0, 255.0) as u8;
                    0xFF000000 | ((glow as u32) << 16) | ((glow as u32) << 8) | (glow_b as u32)
                }
                PixelCache::Disk { r, hit, ray_dir, angle } => {
                    let rotated_angle = angle + time * 0.3;
                    let spin = rotated_angle.sin() * 0.8 + 1.0;
                    let edge_in = ((r - DISK_INNER) / RS).clamp(0.0, 1.0);
                    let edge_out = ((DISK_OUTER - r) / (RS * 2.0)).clamp(0.0, 1.0);
                    disk_color(*r, *hit, *ray_dir, spin * edge_in * edge_out, rotated_angle)
                }
            };
        });

        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
