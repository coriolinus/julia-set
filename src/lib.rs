extern crate crossbeam;
extern crate image;
extern crate num;

use image::ImageBuffer;
use num::complex::Complex64;
use std::sync::{Arc, Mutex};

/// A default julia set function chosen for its aesthetics
pub fn default_julia(z: Complex64) -> Complex64 {
    (z * z) - 0.221 - (0.713 * Complex64::i())
}

/// Count the number of applications of `function` required until either component of
/// the state value of repeated applications of `function(value)`
/// exceeds the threshold. If `bound` is set, don't iterate more than that number of times.
pub fn applications_until<F>(initial: Complex64,
                             function: &F,
                             threshold: f64,
                             bound: Option<usize>)
                             -> usize
    where F: Fn(Complex64) -> Complex64
{
    let mut value = initial;
    let mut count = 0;
    while count < bound.unwrap_or(std::usize::MAX) && value.norm_sqr() < (threshold * threshold) {
        count += 1;
        value = function(value);
    }
    count
}

/// Get an appropriate complex value from a pixel coordinate in a given output size
/// x, y: pixel coordinates
/// width, height: size in pixels of the image
/// min_x, max_x: inclusive range of the output x
/// min_y, max_y: inclusive range of the output y
fn interpolate_pixel(x: u32,
                     y: u32,
                     width: u32,
                     height: u32,
                     min_x: f64,
                     max_x: f64,
                     min_y: f64,
                     max_y: f64)
                     -> Complex64 {
    Complex64::new(min_x + ((x as f64 / (width - 1) as f64) * (max_x - min_x)),
                   min_y + ((y as f64 / (height - 1) as f64) * (max_y - min_y)))
}

/// Construct an image sequentially
pub fn sequential_image<F>(width: u32,
                           height: u32,
                           function: &F,
                           threshold: f64)
                           -> ImageBuffer<image::Luma<u8>, Vec<u8>>
    where F: Fn(Complex64) -> Complex64
{
    // julia sets are only really interesting in the region [-1...1]
    let interpolate = |x, y| interpolate_pixel(x, y, width, height, -1.0, 1.0, -1.0, 1.0);
    ImageBuffer::from_fn(width, height, |x, y| {
        // we know that the output will be in range [0...255], so let's cast it to u8
        // so it'll fill the brightness range properly
        image::Luma([applications_until(interpolate(x, y), function, threshold, Some(255)) as u8])
    })
}

/// Construct an image in a parallel manner
pub fn parallel_image<F>(width: u32,
                         height: u32,
                         function: &F,
                         threshold: f64)
                         -> ImageBuffer<image::Luma<u8>, Vec<u8>>
    where F: Sync + Fn(Complex64) -> Complex64
{
    // julia sets are only really interesting in the region [-1...1]
    let interpolate = Arc::new(|x, y| interpolate_pixel(x, y, width, height, -1.0, 1.0, -1.0, 1.0));
    let mut image = ImageBuffer::new(width, height);

    // open a new scope so we can mutably borrow `image` for the iterator, but also return it
    {
        let pixel_iter = Arc::new(Mutex::new(image.enumerate_pixels_mut()));

        const THREADS: usize = 4; // I'm on a four-real-core machine right now

        crossbeam::scope(|scope| {
            for _ in 0..THREADS {
                // Shadow the iterator here with clone to get an un-Arc'd version
                let pixel_iter = pixel_iter.clone();
                let interpolate = interpolate.clone();

                scope.spawn(move || {
                    // Suggested by reddit user u/Esption:
                    // https://www.reddit.com/r/rust/comments/4vd6vr/what_is_the_scope_of_a_lock_acquired_in_the/d5xjo6x?context=3
                    loop {
                        let step = pixel_iter.lock().unwrap().next();
                        match step {
                            Some((x, y, pixel)) => *pixel = image::Luma([applications_until(interpolate(x, y),
                                                                     function,
                                                                     threshold,
                                                                     Some(255))
                                                  as u8]),
                            None => break,
                        }
                    }
                });
            }
        });

        // Scoped threads take care of ensure everything joins here

    }
    image
}

/// Helper function to save the generated image as-is.
/// Selects file data based on the path name. Use .png
pub fn save_image<F>(width: u32,
                     height: u32,
                     function: &F,
                     threshold: f64,
                     path: &str)
                     -> std::io::Result<()>
    where F: Sync + Fn(Complex64) -> Complex64
{
    parallel_image(width, height, function, threshold).save(path)
}

#[cfg(test)]
mod tests {
    use num::complex::Complex64;
    use super::*;

    /// Fixing the normalization function puts these back to expected values, yay!
    #[test]
    fn test_applications_until() {
        assert_eq!(applications_until(Complex64::new(-1.0, 1.0), &default_julia, 2.0, Some(256)),
                   1);
        assert_eq!(applications_until(Complex64::new(0.0, 1.0), &default_julia, 2.0, Some(256)),
                   5);
        assert_eq!(applications_until(Complex64::new(1.0, 1.0), &default_julia, 2.0, Some(256)),
                   2);
        assert_eq!(applications_until(Complex64::new(-1.0, 0.0), &default_julia, 2.0, Some(256)),
                   3);
        assert_eq!(applications_until(Complex64::new(0.0, 0.0), &default_julia, 2.0, Some(256)),
                   112);
        assert_eq!(applications_until(Complex64::new(1.0, 0.0), &default_julia, 2.0, Some(256)),
                   3);
        assert_eq!(applications_until(Complex64::new(-1.0, -1.0), &default_julia, 2.0, Some(256)),
                   2);
        assert_eq!(applications_until(Complex64::new(0.0, -1.0), &default_julia, 2.0, Some(256)),
                   5);
        assert_eq!(applications_until(Complex64::new(1.0, -1.0), &default_julia, 2.0, Some(256)),
                   1);
    }
}
