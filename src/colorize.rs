//! Functions to colorize an image.
//!
//! The fundamental problem is that the nature of the computation we use
//! to generate the images is monochromatic; the only truly natural mapping
//! of its output is to grayscale. Colorization is useful both to assist in
//! distinguishing fine details, and to make the results simply look prettier.

extern crate hsl;

use image::{GenericImage, ImageBuffer, Pixel, Rgb, Rgba};
use self::hsl::HSL;
use std::marker::PhantomData;

/// A colorizer is anything which can map from one pixel type to another.
pub trait Colorizer: 'static {
    type Image: GenericImage;

    /// Colorize a single pixel
    fn colorize_pixel(&self,
                      x: u32,
                      y: u32,
                      pixel: <<Self as Colorizer>::Image as GenericImage>::Pixel)
                      -> Rgb<<<<Self as Colorizer>::Image as GenericImage>::Pixel
                            as Pixel>::Subpixel>;

    /// Colorize this pixel with alpha information.
    ///
    /// Override this if you'd like `colorize_alpha` to produce results without maximum opacity.
    ///
    /// Default implementation simply calls `colorize` and assigns all pixels full opacity.
    fn colorize_pixel_alpha(&self,
                            x: u32,
                            y: u32,
                            pixel: <<Self as Colorizer>::Image as GenericImage>::Pixel)
                            -> Rgba<<<<Self as Colorizer>::Image as GenericImage>::Pixel
                                    as Pixel>::Subpixel> {
        self.colorize_pixel(x, y, pixel).to_rgba()
    }

    fn colorize
        (&self,
         image: &Self::Image)
         -> ImageBuffer<
            Rgb<<<<Self as Colorizer>::Image as GenericImage>::Pixel  as Pixel>::Subpixel>,
            Vec<<<<Self as Colorizer>::Image as GenericImage>::Pixel as Pixel>::Subpixel>> {
        let (width, height) = image.dimensions();
        let mut buffer = ImageBuffer::new(width, height);

        for (x, y, pixel) in image.pixels() {
            buffer.put_pixel(x, y, self.colorize_pixel(x, y, pixel));
        }

        buffer
    }

    fn colorize_alpha
        (&self,
         image: &Self::Image)
         -> ImageBuffer<
            Rgba<<<<Self as Colorizer>::Image as GenericImage>::Pixel as Pixel>::Subpixel>,
            Vec<<<<Self as Colorizer>::Image as GenericImage>::Pixel as Pixel>::Subpixel>> {
        let (width, height) = image.dimensions();
        let mut buffer = ImageBuffer::new(width, height);

        for (x, y, pixel) in image.pixels() {
            buffer.put_pixel(x, y, self.colorize_pixel_alpha(x, y, pixel));
        }

        buffer
    }
}

// Colorizer which uses HSL color theory to produce pretty colorization.
//
// [HSL] represents the colors as a cylinder. The bottom of the cylinder is black,
// and the top is white. The outside is fully saturated; the center is
// desaturated (i.e. grayscale). Hues vary as you rotate around the axis of
// the cylinder.
//
// Our mapping is a spiral around the outside of the cylinder; S remains
// constant at 1. Black maps to black at L0; white to white at L1.
// H values vary such that darks produce deep blues, and lights produce
// bright yellows. This is a purely aesthetic choice.
//
// [HSL]: https://en.wikipedia.org/wiki/HSL_and_HSV
pub struct HSLColorizer<T> {
    _image_type: PhantomData<T>,
}

impl<T> HSLColorizer<T> {
    pub fn new() -> HSLColorizer<T> {
        HSLColorizer { _image_type: PhantomData }
    }

    fn interpolate(&self, begin: f64, end: f64, t: f64) -> f64 {
        begin + (t * (end - begin))
    }

    // Note that this is a naive interpolater: it doesn't wrap, ever.
    // `t` must be in the range [0, 1]; it describes how far along the range
    // from `begin` to `end` the target color is.
    fn interpolate_hsl(&self, begin: HSL, end: HSL, t: f64) -> HSL {
        HSL {
            h: self.interpolate(begin.h, end.h, t),
            s: self.interpolate(begin.s, end.s, t),
            l: self.interpolate(begin.l, end.l, t),
        }
    }
}

impl<GI> Colorizer for HSLColorizer<GI>
    where GI: GenericImage + 'static,
          GI::Pixel: Pixel<Subpixel = u8>,
          <<GI as GenericImage>::Pixel as Pixel>::Subpixel: 'static
{
    type Image = GI;

    fn colorize_pixel(&self,
                      _: u32,
                      _: u32,
                      pixel: <<Self as Colorizer>::Image as GenericImage>::Pixel)
                      -> Rgb<<<<Self as Colorizer>::Image as GenericImage>::Pixel
                        as Pixel>::Subpixel> {
        // start deep under the dark blues, almost violet
        const BEGIN: HSL = HSL {
            h: 310_f64,
            s: 1_f64,
            l: 0_f64,
        };

        // end just over the region where yellow is becoming orange
        const END: HSL = HSL {
            h: 30_f64,
            s: 1_f64,
            l: 1_f64,
        };

        // we're only dealing with black-and-white inputs, here; there will be
        // exactly one channel as we continue.
        let pixel = pixel.to_luma();
        // we know the pixel's subtype must have bounds, so let's work from
        // there to find our value of t
        // Note that we always assume that 0 is the min bound, not whatever the
        // type's actual min bound is. This is in case someone backs the pixel
        // with a negatable type for some reason.
        let subpixel = pixel.channels()[0];
        let t = subpixel as f64 / u8::max_value() as f64;
        let (r, g, b) = self.interpolate_hsl(BEGIN, END, t).to_rgb();
        Rgb([r, g, b])
    }
}
