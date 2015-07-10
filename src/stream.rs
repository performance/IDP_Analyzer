//! All IO functionality needed for TIFF decoding

use std::io;
use std::io::{Read, Write, Seek};
use byteorder::{self, WriteBytesExt, ReadBytesExt, BigEndian, LittleEndian};

/// Byte order of the TIFF file.
#[derive(Clone, Copy, Debug)]
pub enum ByteOrder {
    /// little endian byte order
    LittleEndian,
    /// big endian byte order
    BigEndian
}



/// Reader that is aware of the byte order.
pub trait EndianReader: Read {
    /// Byte order that should be adhered to
    fn byte_order(&self) -> ByteOrder;

    /// Reads an u16
    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16, byteorder::Error> {
        match self.byte_order() {
            ByteOrder::LittleEndian => <Self as ReadBytesExt>::read_u16::<LittleEndian>(self),
            ByteOrder::BigEndian    => <Self as ReadBytesExt>::read_u16::<BigEndian>(self)
        }
    }

    /// Reads an u32
    #[inline(always)]
    fn read_u32(&mut self) -> Result<u32, byteorder::Error> {
        match self.byte_order() {
            ByteOrder::LittleEndian => <Self as ReadBytesExt>::read_u32::<LittleEndian>(self),
            ByteOrder::BigEndian    => <Self as ReadBytesExt>::read_u32::<BigEndian>(self)
        }
    }
    
    
    /// Reads an f32
    #[inline(always)]
    fn read_f32(&mut self ) -> Result<f32, byteorder::Error> {
        match self.byte_order() {
            ByteOrder::LittleEndian => <Self as ReadBytesExt>::read_f32::<LittleEndian>(self),
            ByteOrder::BigEndian    => <Self as ReadBytesExt>::read_f32::<BigEndian>(   self)
        }
    }
    
}

/// Writer that is aware of the byte order.
pub trait EndianWriter: Write {
    /// Byte order that should be adhered to
    fn byte_order(&self) -> ByteOrder;

    /// Writes an u16
    #[inline(always)]
    fn write_u16(&mut self, n: u16) -> Result<(), byteorder::Error> {
        match self.byte_order() {
            ByteOrder::LittleEndian => <Self as WriteBytesExt>::write_u16::<LittleEndian>(self, n),
            ByteOrder::BigEndian => <Self as WriteBytesExt>::write_u16::<BigEndian>(self, n)
        }
    }

    /// Writes an u32
    #[inline(always)]
    fn write_u32(&mut self, n: u32) -> Result<(), byteorder::Error> {
        match self.byte_order() {
            ByteOrder::LittleEndian => <Self as WriteBytesExt>::write_u32::<LittleEndian>(self, n),
            ByteOrder::BigEndian => <Self as WriteBytesExt>::write_u32::<BigEndian>(self, n)
        }
    }

    /// Writes an u32
    #[inline(always)]
    fn write_f32(&mut self, n: f32) -> Result<(), byteorder::Error> {
        match self.byte_order() {
            ByteOrder::LittleEndian => <Self as WriteBytesExt>::write_f32::<LittleEndian>(self, n),
            ByteOrder::BigEndian    => <Self as WriteBytesExt>::write_f32::<BigEndian>(   self, n)
        }
    }
}



/// Reader that is aware of the byte order.
#[derive(Debug)]
pub struct SmartReader<R> where R: Read + Seek {
    reader: R,
    pub byte_order: ByteOrder
}

impl<R> SmartReader<R> where R: Read + Seek {
    /// Wraps a reader
    pub fn wrap(reader: R, byte_order: ByteOrder) -> SmartReader<R> {
        SmartReader {
            reader: reader,
            byte_order: byte_order
        }
    }
}

impl<R> EndianReader for SmartReader<R> where R: Read + Seek {
    #[inline(always)]
    fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }
}

impl<R: Read + Seek> Read for SmartReader<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Read + Seek> Seek for SmartReader<R> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}




/// Reader that is aware of the byte order.
#[derive(Debug)]
pub struct SmartWriter<W> where W: Write + Seek {
    writer: W,
    pub byte_order: ByteOrder
}

impl<W> SmartWriter<W> where W: Write + Seek {
    /// Wraps a writer
    pub fn wrap(writer: W, byte_order: ByteOrder) -> SmartWriter<W> {
        SmartWriter {
            writer: writer,
            byte_order: byte_order
        }
    }
}

impl<W> EndianWriter for SmartWriter<W> where W: Write + Seek {
    #[inline(always)]
    fn byte_order(&self) -> ByteOrder {
        self.byte_order
    }
}

impl<W: Write + Seek> Write for SmartWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
    
    
}

impl<W: Write + Seek> Seek for SmartWriter<W> {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.writer.seek(pos)
    }
}



