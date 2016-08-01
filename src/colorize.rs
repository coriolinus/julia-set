//! Functions to colorize an image.
//!
//! The fundamental problem is that the nature of the computation we use
//! to generate the images is monochromatic; the only truly natural mapping
//! of its output is to grayscale. Colorization is useful both to assist in
//! distinguishing fine details, and to make the results simply look prettier.

extern crate hsl;
extern crate image;

use image::{GenericImage, ImageBuffer, Pixel, Rgb, Rgba};

/// A colorizer is anything which can map from one pixel type to another.
pub trait Colorizer<GI>
    where GI: GenericImage,
          <<GI as GenericImage>::Pixel as Pixel>::Subpixel: 'static
{
    /// Colorize a single pixel
    fn colorize_pixel(&self,
                      x: u32,
                      y: u32,
                      pixel: GI::Pixel)
                      -> Rgb<<<GI as GenericImage>::Pixel as Pixel>::Subpixel>;

    /// Colorize this pixel with alpha information.
    ///
    /// Override this if you'd like `colorize_alpha` to produce results without maximum opacity.
    ///
    /// Default implementation simply calls `colorize` and assigns all pixels full opacity.
    fn colorize_pixel_alpha(&self,
            x: u32,
            y: u32, pixel: GI::Pixel)
                            -> Rgba<<<GI as GenericImage>::Pixel as Pixel>::Subpixel> {
        self.colorize_pixel(x, y, pixel).to_rgba()
    }

    fn colorize(&self, image: &GI)
        -> ImageBuffer<Rgb<<<GI as GenericImage>::Pixel as Pixel>::Subpixel>,
                       Vec<<<GI as GenericImage>::Pixel as Pixel>::Subpixel>>
    {
        let (width, height) = image.dimensions();
        let mut buffer = ImageBuffer::new(width, height);

        for (x, y, pixel) in image.pixels() {
            buffer.put_pixel(x, y, self.colorize_pixel(x, y, pixel));
        }

        buffer
    }

    fn colorize_alpha(&self, image: &GI)
        -> ImageBuffer<Rgba<<<GI as GenericImage>::Pixel as Pixel>::Subpixel>,
                       Vec<<<GI as GenericImage>::Pixel as Pixel>::Subpixel>>
    {
        let (width, height) = image.dimensions();
        let mut buffer = ImageBuffer::new(width, height);

        for (x, y, pixel) in image.pixels() {
            buffer.put_pixel(x, y, self.colorize_pixel_alpha(x, y, pixel));
        }

        buffer
    }
}

/// Colorizer which uses HSL color theory to produce pretty colorization.
///
/// [HSL] represents the colors as a cylinder. The bottom of the cylinder is black,
/// and the top is white. The outside is fully saturated; the center is
/// desaturated (i.e. grayscale). Hues vary as you rotate around the axis of
/// the cylinder.
///
/// Our mapping is a spiral around the outside of the cylinder; S remains
/// constant at 1. Black maps to black at L0; white to white at L1.
/// H values vary such that darks produce deep blues, and lights produce
/// bright yellows. This is a purely aesthetic choice.
///
/// [HSL]: https://en.wikipedia.org/wiki/HSL_and_HSV
pub struct HSLColorizer;

impl HSLColorizer {
    pub fn new() -> HSLColorizer {
        HSLColorizer {}
    }
}

impl<GI> Colorizer<GI> for HSLColorizer
    where GI: GenericImage,
          <<GI as GenericImage>::Pixel as Pixel>::Subpixel: 'static
{
    fn colorize_pixel(&self,
                      _: u32,
                      _: u32,
                      pixel: GI::Pixel)
                      -> Rgb<<<GI as GenericImage>::Pixel as Pixel>::Subpixel> {
        unimplemented!()
    }
}
