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

/// As this isn't really a user-facing program so much as a dev tool,
/// we just hard-code a bunch of constants here and recompile if we
/// want to change them.
fn main() {
    const LOW: f64 = -1.5;
    const HIGH: f64 = 1.5;
    const STEPS: u32 = 7;
    const INTERVAL: f64 = (HIGH - LOW) / (STEPS - 1) as f64; // 0.5 in range [-1.5..1.5] with 7

    let path = {
        let mut path = env::current_dir().unwrap();
        path.push("tiles");
        path
    };

    const TILE_EDGE: u32 = 200;

    let interpolate = interpolate_stretch(TILE_EDGE, TILE_EDGE, -1.0, 1.0, -1.0, 1.0);

    for threshold in (1_u8..6).map(|t| t as f64 / 2.0) {
        println!("For threshold {}:", threshold);
        let mut output: ImageBuffer<image::Luma<u8>, Vec<u8>> = ImageBuffer::new(TILE_EDGE * STEPS,
                                                                                 TILE_EDGE * STEPS);

        for (y, imag) in (0..STEPS).map(|s| (s * TILE_EDGE, LOW + (s as f64 * INTERVAL))) {
            for (x, real) in (0..STEPS).map(|s| (s * TILE_EDGE, LOW + (s as f64 * INTERVAL))) {
                println!("\tGenerating tile for ({} + {}i)", real, imag);
                let fcz = reify_fcz(Complex64::new(real, imag));
                let tile = parallel_image(TILE_EDGE, TILE_EDGE, &*fcz, &*interpolate, threshold);
                if !output.copy_from(&tile, x, y) {
                    println!("FATAL: Failed to copy tile into output.");
                    println!("\tTile at ({}, {}) sized ({}, {})",
                             x,
                             y,
                             TILE_EDGE,
                             TILE_EDGE);
                    let (width, height) = output.dimensions();
                    println!("\tOutput container dimensions ({}, {})", width, height);
                    panic!();
                }
            }
        }

        let file_name = {
            let mut file_name = path.clone();
            file_name.push(&format!("julia_threshold_{}_range_{}..{}.png", threshold, LOW, HIGH));
            file_name.to_string_lossy().into_owned()
        };

        println!("\tSaving as {:?}", file_name);
        if let Err(error) = output.save(file_name) {
            println!("FATAL: Failed to save image.");
            println!("\t{}", error);
            panic!();
        }
    }
}