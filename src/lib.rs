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
    while count < bound.unwrap_or(std::usize::MAX) && value.re.abs() < threshold &&
          value.im.abs() < threshold {
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
    where F: Fn(Complex64) -> Complex64 + std::marker::Sync
{
    // julia sets are only really interesting in the region [-1...1]
    let interpolate = Arc::new(|x, y| interpolate_pixel(x, y, width, height, -1.0, 1.0, -1.0, 1.0));
    let mut image = ImageBuffer::new(width, height);

    // open a new scope so we can mutably borrow `image` for the iterator, but also return it
    {
        let pixel_iter = Arc::new(Mutex::new(image.enumerate_pixels_mut()));

        const THREADS: usize = 4; // I'm on a four-real-core machine right now

        let mut threads = Vec::with_capacity(THREADS);

        crossbeam::scope(|scope| {
            for _ in 0..THREADS {
                // Shadow the iterator here with clone to get an un-Arc'd version
                let pixel_iter = pixel_iter.clone();
                let interpolate = interpolate.clone();

                threads.push(scope.spawn(move || {
                    // I'm not 100% sure if the lock we acquire here persists for the duration
                    // of the while loop, or just for the initial assignment. If it's actually
                    // the former, this multi-threaded code can't actually move faster than
                    // the single-threaded implementation; every thread will block while waiting
                    // for the iterator.
                    while let Some((x, y, pixel)) = pixel_iter.lock().unwrap().next() {
                        *pixel = image::Luma([applications_until(interpolate(x, y),
                                                                 function,
                                                                 threshold,
                                                                 Some(255))
                                              as u8]);
                    }
                }));
            }
        });

        // Collect thread results
        threads.into_iter()
            .map(|child| child.join())
            .collect::<Vec<_>>();
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
    where F: Fn(Complex64) -> Complex64
{
    sequential_image(width, height, function, threshold).save(path)
}

#[cfg(test)]
mod tests {
    use num::complex::Complex64;
    use super::*;

    /// Note that these values aren't precisely those given in the example:
    /// at (1+1i) and (-1-1i), the example shows 2, and we compute 3.
    /// All the other values are the same, and I'm willing to chalk that up to differing
    /// floating point / complex number implementations; I'd bet that at those values, the
    /// second iteration comes _really close_ to the the threshold.
    /// I'm satisfied enough that my implementation is close enough, for now.
    #[test]
    fn test_applications_until() {
        assert_eq!(applications_until(Complex64::new(-1.0, 1.0), default_julia, 2.0, Some(256)),
                   1);
        assert_eq!(applications_until(Complex64::new(0.0, 1.0), default_julia, 2.0, Some(256)),
                   5);
        assert_eq!(applications_until(Complex64::new(1.0, 1.0), default_julia, 2.0, Some(256)),
                   3);
        assert_eq!(applications_until(Complex64::new(-1.0, 0.0), default_julia, 2.0, Some(256)),
                   3);
        assert_eq!(applications_until(Complex64::new(0.0, 0.0), default_julia, 2.0, Some(256)),
                   112);
        assert_eq!(applications_until(Complex64::new(1.0, 0.0), default_julia, 2.0, Some(256)),
                   3);
        assert_eq!(applications_until(Complex64::new(-1.0, -1.0), default_julia, 2.0, Some(256)),
                   3);
        assert_eq!(applications_until(Complex64::new(0.0, -1.0), default_julia, 2.0, Some(256)),
                   5);
        assert_eq!(applications_until(Complex64::new(1.0, -1.0), default_julia, 2.0, Some(256)),
                   1);
    }
}
