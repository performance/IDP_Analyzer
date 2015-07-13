use std::io;
use std::fmt;
use std::convert::From;
use std::error::Error;
use byteorder;


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