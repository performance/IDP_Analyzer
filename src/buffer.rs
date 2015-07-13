use std::slice::{ Chunks, ChunksMut };
use std::ops::{ Deref, DerefMut, Index, IndexMut };
use std::marker::PhantomData;
use std::iter::repeat;
use std::path::Path;
use std::io;
use num::Zero;

use std::io::{BufReader, BufWriter};
use std::fs::File;
use stream::{
    ByteOrder,
    EndianWriter,
    SmartWriter,
    // SmartReader
};

use traits::{ Pixel, Primitive, GenericImage }; // , ImageDecoder };
//use color::{ Rgb, Rgba, Luma, LumaA, FromColor, ColorType };
use image::other::{
    PixelType,
    Pixels,
    GrayU16,
    GrayF32
}; 

// use image::GenericImage;
// use dynimage::{save_buffer,image_to_bytes};
// use utils::expand_packed;


/// Iterate over pixel refs.
pub struct PixelRefs<'a, P: Pixel + 'a> where P::Subpixel: 'a {
    chunks: Chunks<'a, P::Subpixel>
}

impl<'a, P: Pixel + 'a> Iterator for PixelRefs<'a, P> where P::Subpixel: 'a {
    type Item = &'a P;

    #[inline(always)]
    fn next(&mut self) -> Option<&'a P> {
        self.chunks.next().map(|v|
            <P as Pixel>::from_slice(v)
        )
    }
}

impl<'a, P: Pixel + 'a> DoubleEndedIterator for PixelRefs<'a, P> where P::Subpixel: 'a {

    #[inline(always)]
    fn next_back(&mut self) -> Option<&'a P> {
        self.chunks.next_back().map(|v|
            <P as Pixel>::from_slice(v)
        )
    }
}

/// Iterate over mutable pixel refs.
pub struct PixelsMut<'a, P: Pixel + 'a> where P::Subpixel: 'a {
    chunks: ChunksMut<'a, P::Subpixel>
}

impl<'a, P: Pixel + 'a> Iterator for PixelsMut<'a, P> where P::Subpixel: 'a {
    type Item = &'a mut P;

    #[inline(always)]
    fn next(&mut self) -> Option<&'a mut P> {
        self.chunks.next().map(|v|
            <P as Pixel>::from_slice_mut(v)
        )
    }
}

impl<'a, P: Pixel + 'a> DoubleEndedIterator for PixelsMut<'a, P> where P::Subpixel: 'a {
    #[inline(always)]
    fn next_back(&mut self) -> Option<&'a mut P> {
        self.chunks.next_back().map(|v|
            <P as Pixel>::from_slice_mut(v)
        )
    }
}

/// Enumerate the pixels of an image.
pub struct EnumeratePixels<'a, P: Pixel + 'a> where <P as Pixel>::Subpixel: 'a {
    pixels: PixelRefs<'a, P>,
    x:      u32,
    y:      u32,
    width:  u32
}

impl<'a, P: Pixel + 'a> Iterator for EnumeratePixels<'a, P> where P::Subpixel: 'a {
    type Item = (u32, u32, &'a P);

    #[inline(always)]
    fn next(&mut self) -> Option<(u32, u32, &'a P)> {
        if self.x >= self.width {
            self.x =  0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        match self.pixels.next() {
            None => None,
            Some(p) => Some((x, y, p))
        }
    }
}

/// Enumerate the pixels of an image.
pub struct EnumeratePixelsMut<'a, P: Pixel + 'a> where <P as Pixel>::Subpixel: 'a {
    pixels: PixelsMut<'a, P>,
    x:      u32,
    y:      u32,
    width:  u32
}

impl<'a, P: Pixel + 'a> Iterator for EnumeratePixelsMut<'a, P> where P::Subpixel: 'a {
    type Item = (u32, u32, &'a mut P);

    #[inline(always)]
    fn next(&mut self) -> Option<(u32, u32, &'a mut P)> {
        if self.x >= self.width {
            self.x =  0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        match self.pixels.next() {
            None => None,
            Some(p) => Some((x, y, p))
        }
    }
}

/// Generic image buffer
pub struct ImageBuffer<P: Pixel, Container> {
    width: u32,
    height: u32,
    _phantom: PhantomData<P>,
    data: Container,
}


impl<P, Container> GenericImage for ImageBuffer<P, Container>
where P: Pixel + 'static,
      Container: Deref<Target=[P::Subpixel]> + DerefMut,
      P::Subpixel: 'static {

    type Pixel = P;

    fn dimensions(&self) -> (u32, u32) {
        self.dimensions()
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width, self.height)
    }

    fn get_pixel(&self, x: u32, y: u32) -> P {
        *self.get_pixel(x, y)
    }

    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut P {
        self.get_pixel_mut(x, y)
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: P) {
        *self.get_pixel_mut(x, y) = pixel
    }


    fn pixels(&self) -> Pixels<Self> {
        let (width, height) = self.dimensions();

        Pixels::new ( self, 0, 0, width, height )
    }

}


// generic implementation, shared along all image buffers
impl<P, Container> ImageBuffer<P, Container>
where P: Pixel + 'static,
      P::Subpixel: 'static,
      Container: Deref<Target=[P::Subpixel]> {

    /// Contructs a buffer from a generic container
    /// (for example a `Vec` or a slice)
    /// Returns None if the container is not big enough
    pub fn from_raw(width: u32, height: u32, buf: Container)
                    -> Option<ImageBuffer<P, Container>> {
        if width as usize
           * height as usize
           <= buf.len() {
            Some(ImageBuffer {
                data: buf,
                width: width,
                height: height,
                _phantom: PhantomData,
            })
        } else {
            println!("Entered from_raw at line {:?} in file {:?}", line!(), file!() );
            None
        }
    }

    /// Returns the underlying raw buffer
    pub fn into_raw(self) -> Container {
        self.data
    }

    /// The width and height of this image.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The width of this image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns an iterator over the pixels of this image.
    pub fn pixels<'a>(&'a self) -> PixelRefs<'a, P> {
        PixelRefs {
            chunks: self.data.chunks( 1 )
        }
    }

    /// Enumerates over the pixels of the image.
    /// The iterator yields the coordinates of each pixel
    /// along with a reference to them.
    pub fn enumerate_pixels<'a>(&'a self) -> EnumeratePixels<'a, P> {
        EnumeratePixels {
            pixels: self.pixels(),
            x: 0,
            y: 0,
            width: self.width
        }
    }

    /// Gets a reference to the pixel at location `(x, y)`
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds `(width, height)`.
    pub fn get_pixel(&self, x: u32, y: u32) -> &P {
        let no_channels = 1; 
        let index  = no_channels * (y * self.width + x) as usize;
        <P as Pixel>::from_slice(
            &self.data[index .. index + no_channels]
        )
    }
}

impl<P, Container> ImageBuffer<P, Container>
where P: Pixel + 'static,
      P::Subpixel: 'static,
      Container: Deref<Target=[P::Subpixel]> + DerefMut {

    /// Returns an iterator over the mutable pixels of this image.
    /// The iterator yields the coordinates of each pixel
    /// along with a mutable reference to them.
    pub fn pixels_mut(&mut self) -> PixelsMut<P> {
        PixelsMut {
            chunks: self.data.chunks_mut( 1 )
        }
    }

    /// Enumerates over the pixels of the image.
    pub fn enumerate_pixels_mut<'a>(&'a mut self) -> EnumeratePixelsMut<'a, P> {
        let width = self.width;
        EnumeratePixelsMut {
            pixels: self.pixels_mut(),
            x: 0,
            y: 0,
            width: width
        }
    }

    /// Gets a reference to the mutable pixel at location `(x, y)`
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds `(width, height)`.
    pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut P {
        let no_channels = 1; 
        let index  = no_channels * (y * self.width + x) as usize;
        <P as Pixel>::from_slice_mut(
            &mut self.data[index .. index + no_channels]
        )
    }

    /// Puts a pixel at location `(x, y)`
    ///
    /// # Panics
    ///
    /// Panics if `(x, y)` is out of the bounds (width, height)`.
    pub fn put_pixel(&mut self, x: u32, y: u32, pixel: P) {
        *self.get_pixel_mut(x, y) = pixel
    }
}


impl<P, Container> ImageBuffer<P, Container>
where P: Pixel + 'static , P::Subpixel: Primitive + 'static
//where P: Pixel<Subpixel=f32> + 'static,
     // Container: Deref<Target=[f32]> 
     {
   /// Saves the buffer to a file at the path specified.
   ///
   /// The image format is derived from the file extension.
   /// Currently only jpeg and png files are supported.
   pub fn save<Q>(&self, output_path: Q) -> io::Result<()> where Q: AsRef<Path> {
       let f = match File::create( output_path ) {
            Ok( file ) => file,
            Err( msg ) => { println!("{}", msg); panic!( "room" ); }
        };
    
        let w = BufWriter::new( &f );
        let mut wtr = SmartWriter::wrap(w, ByteOrder::LittleEndian);
    
        let pixel_type = <P as Pixel>::pixel_type();
        let fmt1 = 0u32;
        
        let fmt2 = match pixel_type {
            PixelType::Short16 => 0u32,
            PixelType::Float32 => 2u32 
        };
        
       wtr.write_u32(fmt1);
       wtr.write_u32(fmt2);
       wtr.write_u32( self.width );
       wtr.write_u32( self.height );
       
       Ok( match pixel_type {
            PixelType::Short16 => {
                for p in self.pixels() {
                    try!(wtr.write_u16( p.value() ));
                }
            },
            PixelType::Float32 => {
                for p in self.pixels() {
                    try!(wtr.write_f32( p.value() ));
                }
            }
        })
   }
}

impl<P, Container> Deref for ImageBuffer<P, Container>
where P: Pixel + 'static,
      P::Subpixel: 'static,
      Container: Deref<Target=[P::Subpixel]> {
    type Target = [P::Subpixel];

    fn deref<'a>(&'a self) -> &'a <Self as Deref>::Target {
        &*self.data
    }
}

impl<P, Container> DerefMut for ImageBuffer<P, Container>
where P: Pixel + 'static,
      P::Subpixel: 'static,
      Container: Deref<Target=[P::Subpixel]> + DerefMut {
    fn deref_mut<'a>(&'a mut self) -> &'a mut <Self as Deref>::Target {
        &mut *self.data
    }
}

impl<P, Container> Index<(u32, u32)> for ImageBuffer<P, Container>
where P: Pixel + 'static,
      P::Subpixel: 'static,
      Container: Deref<Target=[P::Subpixel]> {
    type Output = P;

    fn index(&self, (x, y): (u32, u32)) -> &P {
        self.get_pixel(x, y)
    }
}

impl<P, Container> IndexMut<(u32, u32)> for ImageBuffer<P, Container>
where P: Pixel + 'static,
      P::Subpixel: 'static,
      Container: Deref<Target=[P::Subpixel]> + DerefMut {

    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut P {
        self.get_pixel_mut(x, y)
    }
}

impl<P, Container> Clone for ImageBuffer<P, Container>
where P: Pixel,
      Container: Deref<Target=[P::Subpixel]> + Clone {

    fn clone(&self) -> ImageBuffer<P, Container> {
        ImageBuffer {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
            _phantom: PhantomData,
        }
    }
}


// concrete implementation for `Vec`-baked buffers
// TODO: I think that rustc does not "see" this impl any more: the impl with
// Container meets the same requirements. At least, I got compile errors that
// there is no such function as `into_vec`, whereas `into_raw` did work, and
// `into_vec` is redundant anyway, because `into_raw` will give you the vector,
// and it is more generic.
impl<P: Pixel + 'static> ImageBuffer<P, Vec<P::Subpixel>>
where P::Subpixel: 'static {

    /// Creates a new image buffer based on a `Vec<P::Subpixel>`.
    pub fn new(width: u32, height: u32) -> ImageBuffer<P, Vec<P::Subpixel>> {
        ImageBuffer {
            data: repeat(Zero::zero()).take(
                    (width as u64
                     * height as u64
                    ) as usize
                ).collect(),
            width: width,
            height: height,
            _phantom: PhantomData,
        }
    }

    /// Constructs a new ImageBuffer by copying a pixel
    pub fn from_pixel(width: u32, height: u32, pixel: P)
                      -> ImageBuffer<P, Vec<P::Subpixel>> {
        let mut buf = ImageBuffer::new(width, height);
        for p in buf.pixels_mut() {
            *p = pixel
        }
        buf
    }

    /// Constructs a new ImageBuffer by repeated application of the supplied function.
    /// The arguments to the function are the pixel's x and y coordinates.
    pub fn from_fn<F>(width: u32, height: u32, f: F)
                      -> ImageBuffer<P, Vec<P::Subpixel>>
                      where F: Fn(u32, u32) -> P {
        let mut buf = ImageBuffer::new(width, height);
        for (x, y,  p) in buf.enumerate_pixels_mut() {
            *p = f(x, y)
        }
        buf
    }

    /// Creates an image buffer out of an existing buffer.
    /// Returns None if the buffer is not big enough.
    pub fn from_vec(width: u32, height: u32, buf: Vec<P::Subpixel>)
                    -> Option<ImageBuffer<P, Vec<P::Subpixel>>> {
        ImageBuffer::from_raw(width, height, buf)
    }

    /// Consumes the image buffer and returns the underlying data
    /// as an owned buffer
    pub fn into_vec(self) -> Vec<P::Subpixel> {
        self.into_raw()
    }
}

/// Sendable grayscale image buffer
pub type Gray16Image = ImageBuffer<GrayU16<u16>, Vec<u16>>;
/// Sendable grayscale + alpha channel image buffer
pub type GrayFloatImage = ImageBuffer<GrayF32<f32>, Vec<f32>>;

#[cfg(test)]
mod test {

    use super::{ImageBuffer, RgbImage, GrayImage, ConvertBuffer, Pixel};
    use color;
    use test;

    #[test]
    /// Tests if image buffers from slices work
    fn slice_buffer() {
        let data = [0; 9];
        let buf: ImageBuffer<color::Luma<u8>, _> = ImageBuffer::from_raw(3, 3, &data[..]).unwrap();
        assert_eq!(&*buf, &data[..])
    }

    #[test]
    fn test_get_pixel() {
        let mut a: RgbImage = ImageBuffer::new(10, 10);
        {
            let b = a.get_mut(3 * 10).unwrap();
            *b = 255;
        }
        assert_eq!(a.get_pixel(0, 1)[0], 255)

    }

    #[test]
    fn test_mut_iter() {
        let mut a: RgbImage = ImageBuffer::new(10, 10);
        {
            let val = a.pixels_mut().next().unwrap();
            *val = color::Rgb([42, 0, 0]);
        }
        assert_eq!(a.data[0], 42)
    }

    #[bench]
    fn bench_conversion(b: &mut test::Bencher) {
        let mut a: RgbImage = ImageBuffer::new(1000, 1000);
        for mut p in a.pixels_mut() {
            let rgb = p.channels_mut();
            rgb[0] = 255;
            rgb[1] = 23;
            rgb[2] = 42;
        }
        assert!(a.data[0] != 0);
        b.iter(|| {
            let b: GrayImage = a.convert();
            assert!(0 != b.data[0]);
            assert!(a.data[0] != b.data[0]);
            test::black_box(b);
        });
        b.bytes = 1000*1000*3
    }
}
