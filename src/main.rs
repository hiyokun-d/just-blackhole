use minifb::{Key, KeyRepeat, Window, WindowOptions};
use rayon::prelude::*;

const SCREEN_WIDTH: usize = 800; // usize, unsigned pointer of integer
const SCREEN_HEIGHT: usize = 600;

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

    let mut frame_count = 0;

    while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
        let rows_per_chunk = 10;
        let chunk_size = SCREEN_WIDTH * rows_per_chunk;
        buffer
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(index, chunk)| {
                let start_y = (index * rows_per_chunk) as f32;

                for (pixel_idx, pixel) in chunk.iter_mut().enumerate() {
                    // 1. Hitung koordinat X dan Y global untuk piksel saat ini
                    // WIDTH adalah konstanta lebar window kamu (misal: 800)
                    let x = (pixel_idx % SCREEN_WIDTH) as f32;
                    let y = start_y + (pixel_idx / SCREEN_WIDTH) as f32;

                    // 2. Kalkulasi animasi gradien menggunakan fungsi sin/cos agar pergerakannya halus
                    // Kita normalisasi nilainya agar berada di rentang 0.0 - 1.0, lalu kalikan 255.0
                    let r = (((x / SCREEN_WIDTH as f32 + frame_count as f32).sin() * 0.5 + 0.5)
                        * 255.0) as u32;
                    let g = (((y / SCREEN_HEIGHT as f32 + frame_count as f32 * 1.5).cos() * 0.5
                        + 0.5)
                        * 255.0) as u32;
                    let b = ((((x + y) / (SCREEN_WIDTH + SCREEN_HEIGHT) as f32
                        + frame_count as f32 * 2.0)
                        .sin()
                        * 0.5
                        + 0.5)
                        * 255.0) as u32;

                    // 3. Gabungkan komponen warna menjadi format ARGB (0xFF untuk Alpha penuh / tidak transparan)
                    let pixel_color = (0xFF << 24) | (r << 16) | (g << 8) | b;

                    // 4. Masukkan warna ke dalam pixel buffer
                    *pixel = pixel_color;
                }
            });

        if window.is_key_down(Key::A) {
            frame_count += 1;
        } else if window.is_key_down(Key::D) {
            frame_count -= 1;
        };

        // if sthe app is failed, we simply want the app to exit and unwrap itself, may different
        // do something different here
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
