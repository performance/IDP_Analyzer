use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::result::Result;
use std::io::{Read, Write, Seek};
use std::error::Error;
use byteorder;
// use byteorder::Error;
// use byteorder::Error::UnexpectedEOF;


use std::convert::From;

use super::stream::{
    ByteOrder,
    EndianReader,
    SmartReader
};
#[derive(Debug)]
pub enum PixelType {
    Short16,
    Float32
}

// pub type ImageError = String;

/// An enumeration of Image Errors
#[derive(Debug)]
pub enum ImageError {
    /// The Image is not formatted properly
    FormatError(String),
        /// An I/O Error occurred while decoding the image
    IoError(io::Error)
}



impl fmt::Display for ImageError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &ImageError::FormatError(ref e) => write!(fmt, "Format error: {}", e),
            &ImageError::IoError(ref e) => e.fmt(fmt)
        }
    }
}

impl Error for ImageError {
    fn description (&self) -> &str {
        match *self {
            ImageError::FormatError(..) => &"Format error",
            ImageError::IoError(..) => &"IO error"
        }
    }

    fn cause (&self) -> Option<&Error> {
        match *self {
            ImageError::IoError(ref e) => Some(e),
            _ => None
        }
    }
}

impl From<io::Error> for ImageError {
    fn from(err: io::Error) -> ImageError {
        ImageError::IoError(err)
    }
}


impl From<byteorder::Error> for ImageError {
    fn from(err: byteorder::Error) -> ImageError {
        match err {
            byteorder::Error::UnexpectedEOF => ImageError::FormatError( "Format error: ".to_string()  ),
            byteorder::Error::Io(err) => ImageError::IoError(err),
        }
    }
}
pub type ImageResult<T> = Result<T, ImageError>;

/// Result of a decoding process
pub enum DecodingResult {
    /// A vector of unsigned bytes
    U16(Vec<u16>),
    /// A vector of unsigned words
    F32(Vec<f32>)
}

/// The trait that all decoders implement
pub trait ImageDecoder: Sized {
    /// Returns a tuple containing the width and height of the image
    fn dimensions(&mut self) -> ImageResult<(u32, u32)>;

    /// Returns the color type of the image e.g RGB(8) (8bit RGB)
    fn pixel_type(&mut self) -> ImageResult<PixelType>;
    
    /// Decodes the entire image and return it as a Vector
    fn read_image(&mut self) -> ImageResult<DecodingResult>;
}




#[derive(Debug)]
pub struct IDPDecoder<R> where R: Read + Seek {
    reader: SmartReader<R>,
    byte_order: ByteOrder,
    width: u32,
    height: u32,
    pixel_type: PixelType,
}

// A buffer for image decoding
pub enum DecodingBuffer<'a> {
    /// A slice of unsigned bytes
    U16(&'a mut [u16]),
    /// A slice of unsigned words
    F32(&'a mut [f32])
}


impl<R: Read + Seek> IDPDecoder<R> {  
    /// Create a new decoder that decodes from the stream ```r```
    pub fn new(r: R) -> ImageResult<IDPDecoder<R>> {
        IDPDecoder {
            reader: SmartReader::wrap(r, ByteOrder::LittleEndian),
            byte_order: ByteOrder::LittleEndian,
            width: 0,
            height: 0,
            pixel_type: PixelType::Short16,
        }.init()
    }

    fn read_header(&mut self) -> ImageResult<()> {
        let fmt1 = try!(self.reader.read_u32() );
        let fmt2 = try!(self.reader.read_u32() );
        self.pixel_type = match ( fmt1, fmt2 ) {
            ( 0, 0 ) => PixelType::Short16,
            ( 0, 2 ) => PixelType::Float32,
            _ => panic!("Invalid IDP header found" )
        };
        self.width  = try!(self.reader.read_u32() );
        self.height = try!(self.reader.read_u32() );
        
        Ok(())
    }

    /// Initializes the decoder.
    pub fn init(self) -> ImageResult<IDPDecoder<R>> {
        self.next_image()
    }

    /// Reads in the next image.
    /// If there is no further image in the TIFF file a format error is return.
    /// To determine whether there are more images call `IDPDecoder::more_images` instead.
    pub fn next_image(mut self) -> ImageResult<IDPDecoder<R>> {
        try!(self.read_header());
        Ok(self)
    }

    /// Decompresses the strip into the supplied buffer.
    /// Returns the number of bytes read.
    fn expand_strip<'a>(&mut self, decode_buffer: DecodingBuffer<'a> ) -> ImageResult<()> {
        let pixel_type : PixelType = try!(self.pixel_type() );
        let num_pixels = self.width * self.height;
        // let num_bytes = num_pixels * match ( pixel_type ) {
        //     PixelType::Short16 => 2,
        //     PixelType::Float32 => 4,
        // };
        let mut reader: Box<EndianReader> = Box::new(SmartReader::wrap(&mut self.reader, self.byte_order ) ) ;

        Ok(match ( pixel_type, decode_buffer) {
            ( PixelType::Short16, DecodingBuffer::U16(ref mut buffer)) => {
                for datum in &mut buffer[..] {
                    *datum = try!(reader.read_u16());
                }
            },
            ( PixelType::Float32, DecodingBuffer::F32(ref mut buffer)) => {
                for datum in &mut buffer[..] {
                    *datum = try!(reader.read_f32());
                }
            },
            (type_, _) => return Err( ImageError::FormatError(
                    format!( "Pixel type is unsupported")    
                ) )
        })
    }
}



impl<R: Read + Seek> ImageDecoder for IDPDecoder<R> {
    fn dimensions(&mut self) -> ImageResult<(u32, u32)> {
        Ok((self.width, self.height))
    }

    fn pixel_type(&mut self) -> ImageResult<PixelType> {
        match self.pixel_type {
            PixelType::Short16 => Ok( PixelType::Short16 ),
            PixelType::Float32 => Ok( PixelType::Float32 ),    
        }
    }

    fn read_image(&mut self) -> ImageResult<DecodingResult> {
        let number_of_pixels =
              self.width  as usize
            * self.height as usize;
        let mut result = match ( self.pixel_type ) { 
            PixelType::Short16 => DecodingResult::U16( Vec::with_capacity(number_of_pixels)), 
            PixelType::Float32 => DecodingResult::F32( Vec::with_capacity(number_of_pixels)), 
        };
        // Safe since the uninizialized values are never read.
        match result {
            DecodingResult::U16(ref mut buffer) =>
            {
                unsafe { buffer.set_len(number_of_pixels) }
            },
            DecodingResult::F32(ref mut buffer) =>
            {
                unsafe { buffer.set_len(number_of_pixels) }
            },
        } 

        let _ignoreme = match result {
            DecodingResult::U16(ref mut buffer) => {
                try!(self.expand_strip(
                    DecodingBuffer::U16(&mut buffer)
                ))
            },
            DecodingResult::F32(ref mut buffer) => {
                try!(self.expand_strip(
                    DecodingBuffer::F32(&mut buffer)
                ))
            },
        };
        
        Ok(result)
    
    }
}
