use std::iter::repeat;

use num::{ Bounded, Num, NumCast };
use image::other::{
    PixelType,
    Pixels,
    SubImage
};


/// Primitive trait from old stdlib, added max_value
pub trait Primitive: Copy + NumCast + Num + PartialOrd<Self> + Clone + Bounded {
}

impl Primitive for usize {
}
impl Primitive for u8 {
}
impl Primitive for u16 {
}
impl Primitive for u32 {
}
impl Primitive for u64 {
}
impl Primitive for isize {
}
impl Primitive for i8 {
}
impl Primitive for i16 {
}
impl Primitive for i32 {
}
impl Primitive for i64 {
}
impl Primitive for f32 {
}
impl Primitive for f64 {
}



/// A generalized pixel.
///
/// A pixel object is usually not used standalone but as a view into an image buffer.
pub trait Pixel: Copy + Clone {
    /// The underlying subpixel type.
    type Subpixel: Primitive;

    /// Returns the components as a slice.
    fn values(&self) -> &Self::Subpixel;

    /// Returns the components as a mutable slice
    fn values_mut(&mut self) -> &mut Self::Subpixel;

    /// Returns a string that can help to interprete the meaning each channel
    /// See [gimp babl](http://gegl.org/babl/).
    // fn color_model() -> &'static str;

    /// Returns the ColorType for this pixel format
    fn pixel_type() -> PixelType;

    /// Returns a view into a slice.
    ///
    /// Note: The slice length is not checked on creation. Thus the caller has to ensure
    /// that the slice is long enough to precent panics if the pixel is used later on.
    fn from_slice<'a>(slice: &'a [Self::Subpixel]) -> &'a Self;

    /// Returns mutable view into a mutable slice.
    ///
    /// Note: The slice length is not checked on creation. Thus the caller has to ensure
    /// that the slice is long enough to precent panics if the pixel is used later on.
    fn from_slice_mut<'a>(slice: &'a mut [Self::Subpixel]) -> &'a mut Self;


    /// Apply the function ```f``` to each channel of this pixel.
    fn map<F>(& self, f: F) -> Self where F: Fn(Self::Subpixel) -> Self::Subpixel;


    /// Apply the function ```f``` to each channel of this pixel.
    fn apply<F>(&mut self, f: F) where F: Fn(Self::Subpixel) -> Self::Subpixel;

    /// Apply the function ```f``` to each channel of this pixel and
    /// ```other``` pairwise.
    fn map2<F>(&self, other: &Self, f: F) -> Self
        where F: Fn(Self::Subpixel, Self::Subpixel) -> Self::Subpixel;

    /// Apply the function ```f``` to each channel of this pixel and
    /// ```other``` pairwise. Works in-place.
    fn apply2<F>(&mut self, other: &Self, f: F)
        where F: Fn(Self::Subpixel, Self::Subpixel) -> Self::Subpixel;
}



/// A trait for manipulating images.
pub trait GenericImage: Sized {
    /// The type of pixel.
    type Pixel: Pixel;

    /// The width and height of this image.
    fn dimensions(&self) -> (u32, u32);

    /// The width of this image.
    fn width(&self) -> u32 {
        let (w, _) = self.dimensions();
        w
    }

    /// The height of this image.
    fn height(&self) -> u32 {
        let (_, h) = self.dimensions();
        h
    }

    /// The bounding rectangle of this image.
    fn bounds(&self) -> (u32, u32, u32, u32);

    /// Returns true if this x, y coordinate is contained inside the image.
    fn in_bounds(&self, x: u32, y: u32) -> bool {
        let (ix, iy, iw, ih) = self.bounds();
        if x < ix || x >= ix + iw {
            false
        } else if y < iy || y >= iy + ih {
            false
        } else {
            true
        }
    }

    /// Returns the pixel located at (x, y)
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of bounds.
    /// TODO: change this signature to &P
    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel;

    /// Puts a pixel at location (x, y)
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of bounds.
    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut Self::Pixel;

    /// Returns the pixel located at (x, y)
    ///
    /// This function can be implemented in a way that ignores bounds checking.
    unsafe fn unsafe_get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        self.get_pixel(x, y)
    }

    /// Put a pixel at location (x, y)
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of bounds.
    fn put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel);

    /// Puts a pixel at location (x, y)
    ///
    /// This function can be implemented in a way that ignores bounds checking.
    unsafe fn unsafe_put_pixel(&mut self, x: u32, y: u32, pixel: Self::Pixel) {
        self.put_pixel(x, y, pixel);
    }

    /// Returns an Iterator over the pixels of this image.
    /// The iterator yields the coordinates of each pixel
    /// along with their value
    fn pixels(&self) -> Pixels<Self>
    {
       let (width, height) = self.dimensions();

       Pixels {
           image:  self,
           x:      0,
           y:      0,
           width:  width,
           height: height,
       }
    }

    /// Copies all of the pixels from another image into this image.
    ///
    /// The other image is copied with the top-left corner of the
    /// other image placed at (x, y).
    ///
    /// In order to copy only a pice of the other image, use `sub_image`.
    ///
    /// # Returns
    /// `true` if the copy was successful, `false` if the image could not
    /// be copied due to size constraints.
    fn copy_from<O>(&mut self, other: &O, x: u32, y:u32) -> bool
    where O: GenericImage<Pixel=Self::Pixel> {
        // Do bounds checking here so we can use the non-bounds-checking
        // functions to copy pixels.
        if self.width() < other.width() + x {
            return false;
        } else if self.height() < other.height() + y {
            return false;
        }

        for i in 0 .. other.width() {
            for k in 0 .. other.height() {
                unsafe {
                    let p = other.unsafe_get_pixel(i, k);
                    self.unsafe_put_pixel(i + x, k + y, p);
                }
            }
        }
        true
    }

    /// Returns a subimage that is a view into this image.
    fn sub_image<'a>(&'a mut self, x: u32, y: u32, width: u32, height: u32)
    -> SubImage<'a, Self>
    where Self: 'static, <Self::Pixel as Pixel>::Subpixel: 'static,
    Self::Pixel: 'static {
        SubImage::new(self, x, y, width, height)
    }
}

