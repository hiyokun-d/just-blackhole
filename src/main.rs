use std::f32::consts::PI;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::random_range;
use rayon::prelude::*;

const SCREEN_WIDTH: usize = 800; // usize, unsigned pointer of integer
const SCREEN_HEIGHT: usize = 600;

const M: f32 = 1.0; // mass in geometric units
const RS: f32 = 2.0 * M; // Schwarzschild radius (G=c=1, so rs = 2M)
const CAM_DIST: f32 = 15.0; // camera sits this far from BH (in units of M)

const DISK_INNER: f32 = 3.0 * RS;
const DISK_OUTER: f32 = 12.0 * RS;
const T_MAX: f32 = 1.0;

enum RayResult {
    Captured,
    Escaped(f32),
    HitDisk(f32),
}

fn deriv(u: f32, w: f32) -> (f32, f32) {
    return (w, (3.0 / 2.0) * RS * u.powi(2) - u);
}
// Runge-kutta 4th order
// to calculate a single step update using the 4th order runge-kutta method, something like this
// formula for Runge-Kutta: dy/dx = f(x, y) with an initial condition y(x0) = y0
fn geodesic_step(u: f32, w: f32, dphi: f32) -> (f32, f32) {
    let k1 = deriv(u, w);
    let k2 = deriv(u + k1.0 * dphi / 2.0, w + k1.1 * dphi / 2.0);
    let k3 = deriv(u + k2.0 * dphi / 2.0, w + k2.1 * dphi / 2.0);
    let k4 = deriv(u + k3.0 * dphi, w + k3.1 * dphi);

    let new_u = u + (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0) * dphi / 6.0;
    let new_w = w + (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1) * dphi / 6.0;

    return (new_u, new_w);
}

fn trace_tray(mut u0: f32, mut w0: f32) -> RayResult {
    let mut phi: f32 = 0.0;

    for _ in 0..1000 {
        (u0, w0) = geodesic_step(u0, w0, 0.01);
        phi += 0.01;

        let r = 1.0 / u0;

        if u0 > 1.0 / RS {
            return RayResult::Captured;
        } else if u0 < 0.0001 {
            return RayResult::Escaped(phi);
        } //else if r >= DISK_INNER && r <= DISK_OUTER && w0 < 0.0 {
          //     return RayResult::HitDisk(r);
          // }
    }

    return RayResult::Escaped(phi);
}

// to manage the color of the disk
fn disk_color(r: f32) -> u32 {
    // calculate the tempreature color, so we can see how it should be
    let t_compute =
        T_MAX * (DISK_INNER / r).powf(0.75) * (1.0 - (DISK_INNER / r).sqrt()).powf(0.25);
    let t = t_compute.clamp(0.0, T_MAX);

    let r = (255.0) as u8;
    let g = (100.0 + 155.0 * t) as u8;
    let b = (20.0 + 235.0 * t) as u8;

    return 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    // mutable variable, a variable that can be change and it's not constant
    let mut window = Window::new(
        "Blackhole simulation",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);

    // change this if the stars is not enough
    let stars: Vec<(f32, f32)> = (0..200)
        .map(|_| {
            let u = random_range(-0.75_f32..0.75_f32);
            let v = random_range(-0.75_f32..0.75_f32);
            (u, v)
        })
        .collect();

    // impact the critical ring
    let b_crit = (3.0 * 3.0_f32.sqrt() / 2.0) * RS;

    while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
        let rows_per_chunk = 10;
        let chunk_size = SCREEN_WIDTH * rows_per_chunk;
        buffer
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(index, chunk)| {
                let start_y = (index * rows_per_chunk) as f32;

                for (pixel_idx, pixel) in chunk.iter_mut().enumerate() {
                    let x = (pixel_idx % SCREEN_WIDTH) as f32;
                    let y = start_y + (pixel_idx / SCREEN_WIDTH) as f32;

                    let u = (x - (SCREEN_WIDTH as f32) / 2.0) / SCREEN_HEIGHT as f32;
                    let v = (y - (SCREEN_HEIGHT as f32) / 2.0) / SCREEN_HEIGHT as f32;

                    let u0 = 1.0 / CAM_DIST;

                    let b = (u.powi(2) + v.powi(2)).sqrt() * CAM_DIST; // impact
                    let w0 = 1.0 / b;

                    if b < b_crit {
                        *pixel = 0xFF000000;
                        continue;
                    }

                    *pixel = match trace_tray(u0, w0) {
                        RayResult::Captured => 0xFF000000,
                        RayResult::HitDisk(_r) => 0xFFFFFFFF,
                        RayResult::Escaped(phi) => {
                            let alpha = phi - PI;
                            let u_bent = u * alpha.cos() - v * alpha.sin();
                            let v_bent = u * alpha.sin() + v * alpha.cos();

                            let hit = stars.iter().any(|&(sx, sy)| {
                                ((u_bent - sx).powi(2) + (v_bent - sy).powi(2)).sqrt() < 0.003
                            });
                            if hit {
                                0xFFFFFFFF
                            } else {
                                0xFF000000
                            }
                        }
                    }
                }
            });

        // if sthe app is failed, we simply want the app to exit and unwrap itself, may different
        // do something different here
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
