extern crate crossbeam;
extern crate image;
extern crate num;

use image::ImageBuffer;
use num::complex::Complex64;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod colorize;

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

/// Gets an appropriate complex value from a pixel coordinate
/// in a given output size.
///
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

/// Construct a closure which gets an appropriate complex value from a pixel coordinate
/// in a given output size.
///
/// This function examines the width and height of the image given, and compares them
/// to the domains of x and y. If the aspect ratios do not match, the range of x or y
/// is **increased** as appropriate to make the aspect ratio match.
///
/// x, y: pixel coordinates
/// width, height: size in pixels of the image
/// min_x, max_x: inclusive range of the output x
/// min_y, max_y: inclusive range of the output y
pub fn interpolate_rectilinear(width: u32,
                               height: u32,
                               mut min_x: f64,
                               mut max_x: f64,
                               mut min_y: f64,
                               mut max_y: f64)
                               -> Box<Fn(u32, u32) -> Complex64 + Send + Sync> {
    let viewport_ratio = width as f64 / height as f64;
    let range_ratio = (max_x - min_x) / (max_y - min_y);

    {
        let match_result = if viewport_ratio > range_ratio {
            // Viewport is wider than the given domain.
            // Need to increase the range of x.
            Some((&mut min_x, &mut max_x))
        } else if viewport_ratio < range_ratio {
            // Viewport is narrower than the given domain
            // Need to increase the range of y.
            Some((&mut min_y, &mut max_y))
        } else {
            None
        };

        if let Some((mut min, mut max)) = match_result {
            // adjust min and max here
            let center = (*min + *max) / 2.0;
            let arm = (*max - center) * viewport_ratio / range_ratio;
            *min = center - arm;
            *max = center + arm;
        }
    }

    Box::new(move |x, y| interpolate_pixel(x, y, width, height, min_x, max_x, min_y, max_y))
}

/// Construct a closure which gets an appropriate complex value from a pixel coordinate
/// in a given output size.
///
/// If the viewport aspect ratio does not match the aspect ratio of the given x, y range,
/// this interpolator stretches the range to fit the viewport.
///
/// x, y: pixel coordinates
/// width, height: size in pixels of the image
/// min_x, max_x: inclusive range of the output x
/// min_y, max_y: inclusive range of the output y
pub fn interpolate_stretch(width: u32,
                           height: u32,
                           min_x: f64,
                           max_x: f64,
                           min_y: f64,
                           max_y: f64)
                           -> Box<Fn(u32, u32) -> Complex64 + Send + Sync> {
    Box::new(move |x, y| interpolate_pixel(x, y, width, height, min_x, max_x, min_y, max_y))
}

/// Construct an image sequentially
pub fn sequential_image<F>(width: u32,
                           height: u32,
                           function: &F,
                           interpolate: &Fn(u32, u32) -> Complex64,
                           threshold: f64)
                           -> ImageBuffer<image::Luma<u8>, Vec<u8>>
    where F: Fn(Complex64) -> Complex64
{
    ImageBuffer::from_fn(width, height, |x, y| {
        // we know that the output will be in range [0...255], so let's cast it to u8
        // so it'll fill the brightness range properly
        image::Luma([applications_until(interpolate(x, y), function, threshold, Some(255)) as u8])
    })
}

/// Construct an image in a parallel manner using row-chunking
pub fn parallel_image<F>(width: u32,
                         height: u32,
                         function: &F,
                         interpolate: &(Fn(u32, u32) -> Complex64 + Send + Sync),
                         threshold: f64)
                         -> ImageBuffer<image::Luma<u8>, Vec<u8>>
    where F: Sync + Fn(Complex64) -> Complex64
{
    const THREADS: usize = 4; // I'm on a four-real-core machine right now
    let image_backend = Arc::new(Mutex::new(vec![0_u8; (width * height) as usize]));
    // let interpolate = Arc::new(*interpolate);
    let row_n = Arc::new(AtomicUsize::new(0));

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            // let interpolate = interpolate.clone();
            let image_backend = image_backend.clone();
            let row_n = row_n.clone();

            scope.spawn(move || {
                // thread-local non-shared storage for the current row
                let mut row = Vec::with_capacity(width as usize);

                loop {
                    let y = row_n.fetch_add(1, Ordering::SeqCst) as u32;
                    if y >= height {
                        break;
                    }

                    row.clear();

                    for x in 0..width as u32 {
                        row.push(applications_until(interpolate(x, y),
                                                    function,
                                                    threshold,
                                                    Some(255)) as u8);
                    }

                    // insert the row into the output buffer
                    let idx_start = (y * width) as usize;
                    let idx_end = ((y + 1) * width) as usize;
                    {
                        image_backend.lock().unwrap()[idx_start..idx_end].clone_from_slice(&row);
                    }
                }
            });
        }
    });

    // Scoped threads take care of ensuring everything joins here
    // Now, unpack the shared backend
    let image_backend = Arc::try_unwrap(image_backend).unwrap().into_inner().unwrap();
    ImageBuffer::from_raw(width, height, image_backend).unwrap()
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

    #[test]
    fn test_serial_parallel_agree() {
        let (width, height) = (200, 200);
        let threshold = 2.0;
        let interpolate = interpolate_stretch(width, height, -1.0, 1.0, -1.0, 1.0);

        assert!(parallel_image(width, height, &default_julia, &*interpolate, threshold)
            .pixels()
            .zip(sequential_image(width, height, &default_julia, &*interpolate, threshold)
                .pixels())
            .all(|(p, s)| p == s));
    }
}
