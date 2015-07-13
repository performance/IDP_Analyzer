
use traits::{ Primitive, Pixel, GenericImage };
use buffer::ImageBuffer;
use std::ops::{ Index, IndexMut };
use std::mem;


#[derive(Copy, PartialEq, Eq, Debug, Clone)]
pub enum PixelType {
    Short16,
    Float32
}


/// Result of a decoding process
pub enum DecodingResult {
    /// A vector of unsigned bytes
    U16(Vec<u16>),
    /// A vector of f32s
    F32(Vec<f32>)
}

// A buffer for image decoding
pub enum DecodingBuffer<'a> {
    /// A slice of unsigned words
    U16(&'a mut [u16]),
    /// A slice of f32
    F32(&'a mut [f32]),
}



/// Immutable pixel iterator
pub struct Pixels<'a, I: 'a> {
    image:  &'a I,
    x:      u32,
    y:      u32,
    width:  u32,
    height: u32
}

impl<'a, I: 'a> Pixels<'a, I > {
    pub fn new(    
        image:  &'a I,
        x:      u32,
        y:      u32,
        width:  u32,
        height: u32) -> Pixels< 'a, I > {
        Pixels {            
            image: image,
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }
}

impl<'a, I: GenericImage> Iterator for Pixels<'a, I> {
    type Item = (u32, u32, I::Pixel);

    fn next(&mut self) -> Option<(u32, u32, I::Pixel)> {
        if self.x >= self.width {
            self.x =  0;
            self.y += 1;
        }

        if self.y >= self.height {
            None
        } else {
            let pixel = self.image.get_pixel(self.x, self.y);
            let p = (self.x, self.y, pixel);

            self.x += 1;

            Some(p)
        }
    }
}


///////////////////////////////////////////////////


/// A View into another image
pub struct SubImage <'a, I: 'a> {
    image:   &'a mut I,
    xoffset: u32,
    yoffset: u32,
    xstride: u32,
    ystride: u32,
}

// TODO: Do we really need the 'static bound on `I`? Can we avoid it?
impl<'a, I: GenericImage + 'static> SubImage<'a, I>
    where I::Pixel: 'static,
          <I::Pixel as Pixel>::Subpixel: 'static {

    /// Construct a new subimage
    pub fn new(image: &mut I, x: u32, y: u32, width: u32, height: u32) -> SubImage<I> {
        SubImage {
            image:   image,
            xoffset: x,
            yoffset: y,
            xstride: width,
            ystride: height,
        }
    }

    /// Returns a mutable reference to the wrapped image.
    pub fn inner_mut(&mut self) -> &mut I {
        &mut (*self.image)
    }

    /// Change the coordinates of this subimage.
    pub fn change_bounds(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.xoffset = x;
        self.yoffset = y;
        self.xstride = width;
        self.ystride = height;
    }

    /// Convert this subimage to an ImageBuffer
    pub fn to_image(&self) -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>> {
        let mut out = ImageBuffer::new(self.xstride, self.ystride);

        for y in (0..self.ystride) {
            for x in (0..self.xstride) {
                let p = self.get_pixel(x, y);
                out.put_pixel(x, y, p);
            }
        }

        out
    }
}

#[allow(deprecated)]
// TODO: Is the 'static bound on `I` really required? Can we avoid it?
impl<'a, I: GenericImage + 'static> GenericImage for SubImage<'a, I>
    where I::Pixel: 'static,
          <I::Pixel as Pixel>::Subpixel: 'static {

    type Pixel = I::Pixel;

    fn dimensions(&self) -> (u32, u32) {
        (self.xstride, self.ystride)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (self.xoffset, self.yoffset, self.xstride, self.ystride)
    }

    fn get_pixel(&self, x: u32, y: u32) -> I::Pixel {
        self.image.get_pixel(x + self.xoffset, y + self.yoffset)
    }

    fn put_pixel(&mut self, x: u32, y: u32, pixel: I::Pixel) {
        self.image.put_pixel(x + self.xoffset, y + self.yoffset, pixel)
    }

    fn get_pixel_mut(&mut self, x: u32, y: u32) -> &mut I::Pixel {
        self.image.get_pixel_mut(x + self.xoffset, y + self.yoffset)
    }
    fn pixels(&self) -> Pixels<Self> {
        let (width, height) = self.dimensions();

        Pixels {
            image:  self,
            x:      0,
            y:      0,
            width:  width,
            height: height,
        }
    }

}

////////////////////////////////////////////////////////////////////////


// type Short16 = PixelType::Short16;
// type Float32 = PixelType::Float32;


macro_rules! define_colors {
    {$(
        $ident:ident,
        $pixel_type: expr,
        #[$doc:meta];
    )*} => {

$( // START Structure definitions

#[$doc]
#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
#[repr(C)]
#[allow(missing_docs)]
pub struct $ident<T: Primitive> { pub data: [T] }
#[allow(non_snake_case, missing_docs)]
pub fn $ident<T: Primitive>(data: [T]) -> $ident<T> {
    $ident {
        data: data
    }
}

impl<T: Primitive + 'static> Pixel for $ident<T> {

    type Subpixel = T;

    fn pixel_type() -> PixelType {
        // match 
        $pixel_type 
//        {
//            PixelType::Short16 => PixelType::Short16,
//            PixelType::Float32 => PixelType::Float32 
//        }
    }
    #[inline(always)]
    fn values(&self) -> &[T] {
        &self.data
    }
    #[inline(always)]
    fn values_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
    fn from_slice<'a>(slice: &'a [T]) -> &'a $ident<T> {
        assert_eq!(slice.len(), 1);
        unsafe { mem::transmute(slice.as_ptr()) }
    }
    fn from_slice_mut<'a>(slice: &'a mut [T]) -> &'a mut $ident<T> {
        assert_eq!(slice.len(), 1);
        unsafe { mem::transmute(slice.as_ptr()) }
    }

    fn map<F>(& self, f: F) -> $ident<T> where F: Fn(T) -> T {
        let mut this = (*self).clone();
        this.apply(f);
        this
    }

    fn apply<F>(&mut self, f: F) where F: Fn(T) -> T {
        for v in self.data.iter_mut() {
            *v = f(*v)
        }
    }

    fn map2<F>(&self, other: &Self, f: F) -> $ident<T> where F: Fn(T, T) -> T {
        let mut this = (*self).clone();
        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &$ident<T>, f: F) where F: Fn(T, T) -> T {
        for (a, &b) in self.data.iter_mut().zip(other.data.iter()) {
            *a = f(*a, b)
        }

    }
}

impl<T: Primitive> Index<usize> for $ident<T> {
    type Output = T;
    #[inline(always)]
    fn index<'a>(&'a self, _index: usize) -> &'a T {
        &self.data[_index]
    }
}

impl<T: Primitive> IndexMut<usize> for $ident<T> {
    #[inline(always)]
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut T {
        &mut self.data[_index]
    }
}

)* // END Structure definitions

    }
}


define_colors! {
    GrayU16, PixelType::Short16, #[doc = "GrayScale 16 bit"];
    GrayF32, PixelType::Float32, #[doc = "GrayScale 32 bit float"];
}





