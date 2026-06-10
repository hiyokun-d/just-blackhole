use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rayon::prelude::*;

const SCREEN_WIDTH: usize = 800; // usize, unsigned pointer of integer
const SCREEN_HEIGHT: usize = 600;

const M: f32 = 1.0; // mass in geometric units
const RS: f32 = 2.0 * M; // Schwarzschild radius (G=c=1, so rs = 2M)
const CAM_DIST: f32 = 10.0; // camera sits this far from BH (in units of M)

// DO NOT REMOVE THIS!
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
                    *pixel = if (u.powi(2) + v.powi(2)).sqrt() < RS / CAM_DIST {
                        0xFFFFFFFF
                    } else {
                        0xFF000000
                    };
                }
            });

        // if sthe app is failed, we simply want the app to exit and unwrap itself, may different
        // do something different here
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
