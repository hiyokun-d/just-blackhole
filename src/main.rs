use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rand::random_range;
use rayon::prelude::*;

const SCREEN_WIDTH: usize = 800; // usize, unsigned pointer of integer
const SCREEN_HEIGHT: usize = 600;

const M: f32 = 1.0; // mass in geometric units
const RS: f32 = 2.0 * M; // Schwarzschild radius (G=c=1, so rs = 2M)
const CAM_DIST: f32 = 10.0; // camera sits this far from BH (in units of M)

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

    let stars: Vec<(f32, f32)> = (0..100)
        .map(|_| {
            let u = random_range(-0.75_f32..0.75_f32);
            let v = random_range(-0.75_f32..0.75_f32);
            (u, v)
        })
        .collect();

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

                    // rendering white dot in center of the screen
                    let is_horizon = (u.powi(2) + v.powi(2)).sqrt() < RS / CAM_DIST;

                    let stars_in = stars.iter().any(|&(su, sv)| {
                        // sqrt((u - su)² + (v - sv)²) < 0.003
                        ((u - su).powi(2) + (v - sv).powi(2)).sqrt() < 0.003
                    });

                    *pixel = if is_horizon {
                        0xFFFFFF
                    } else if stars_in {
                        0xFFFFFF
                    } else {
                        0x00000
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
