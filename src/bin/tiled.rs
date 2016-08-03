extern crate image;
extern crate julia_set;
extern crate num;

use image::{ImageBuffer, GenericImage};
use julia_set::{parallel_image, interpolate_stretch};
use num::complex::Complex64;
use std::env;

/// Construct a boxed function which computes the Julia set
/// J(f_c(z)) where f_c(z) = z^2 + c.
fn reify_fcz(c: Complex64) -> Box<Fn(Complex64) -> Complex64 + Sync> {
    Box::new(move |z| (z * z) + c)
}

fn generate_tiled(low: f64,
                  steps: u32,
                  interval: f64,
                  tile_edge: u32,
                  threshold: f64)
                  -> ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let interpolate = interpolate_stretch(tile_edge, tile_edge, -1.0, 1.0, -1.0, 1.0);

    let mut output = ImageBuffer::new(tile_edge * steps, tile_edge * steps);

    for (y, imag) in (0..steps).map(|s| (s * tile_edge, low + (s as f64 * interval))) {
        for (x, real) in (0..steps).map(|s| (s * tile_edge, low + (s as f64 * interval))) {
            println!("\tGenerating tile for ({} + {}i)", real, imag);
            let fcz = reify_fcz(Complex64::new(real, imag));
            let tile = parallel_image(tile_edge, tile_edge, &*fcz, &*interpolate, threshold);
            if !output.copy_from(&tile, x, y) {
                println!("FATAL: Failed to copy tile into output.");
                println!("\tTile at ({}, {}) sized ({}, {})",
                         x,
                         y,
                         tile_edge,
                         tile_edge);
                let (width, height) = output.dimensions();
                println!("\tOutput container dimensions ({}, {})", width, height);
                panic!();
            }
        }
    }

    output
}

/// As this isn't really a user-facing program so much as a dev tool,
/// we just hard-code a bunch of constants here and recompile if we
/// want to change them.
fn main() {
    const LOW: f64 = -1.5;
    const HIGH: f64 = 1.5;
    const STEPS: u32 = 7;
    const INTERVAL: f64 = (HIGH - LOW) / (STEPS - 1) as f64; // 0.5 in range [-1.5..1.5] with 7
    const TILE_EDGE: u32 = 200;
    const THRESHOLD: f64 = 2.0;

    let output = generate_tiled(LOW, STEPS, INTERVAL, TILE_EDGE, THRESHOLD);

    let file_name = {
        let mut path = env::current_dir().unwrap();
        path.push("tiles");
        path.push(&format!("julia_range_{}..{}.png", LOW, HIGH));
        path.to_string_lossy().into_owned()
    };

    println!("\tSaving as {:?}", file_name);
    if let Err(error) = output.save(file_name) {
        println!("FATAL: Failed to save image.");
        println!("\t{}", error);
        panic!();
    }

}
