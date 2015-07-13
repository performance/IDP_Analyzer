extern crate byteorder;
extern crate num;

use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
// use byteorder::{ ReadBytesExt, BigEndian, LittleEndian};
mod stream;
mod decoder; 
mod buffer;
mod image;
mod traits;

use stream::{
    ByteOrder,
    EndianWriter,
    SmartWriter,
    SmartReader
};

use image::error::{
    ImageResult
};

use image::other::{
    DecodingResult
};


use decoder::{
    IDPDecoder,
    ImageDecoder
};




fn make_test_idp( input_path: &Path) {
    println!("This shold create a test IDP file!");
    // let inpfile = r#"dsr_test_f32.idp"#;
    let f = match File::create( input_path ) {
        Ok( file ) => file,
        Err( msg ) => { println!("{}", msg); panic!( "room" ); }
    };

    let w = BufWriter::new( &f );
    let mut wtr = SmartWriter::wrap(w, ByteOrder::LittleEndian);

    let fmt1 = 0u32;
    let fmt2 = 2u32;
      
    let number_of_columns :u32 = 20u32;
    let number_of_rows: u32    = 10u32;
    
    wtr.write_u32( fmt1 ).unwrap();
    wtr.write_u32( fmt2 ).unwrap();
    wtr.write_u32( number_of_columns ).unwrap();
    wtr.write_u32( number_of_rows    ).unwrap();
    for row in (0..number_of_rows ) {
        for col in ( 0..number_of_columns ) {
            wtr.write_f32( ( col * 100 + row ) as f32  ).unwrap()
        }
    }
}


fn read_test_idp( input_path: &Path) -> ImageResult<DecodingResult> {
    let f = match File::open( input_path ) {
        Ok( file ) => file,
        Err( msg ) => { println!("{}", msg); panic!( "could not open input file" ); }
    };

    let bufr = BufReader::new( &f );
    let rdr = SmartReader::wrap( bufr, ByteOrder::LittleEndian );
    let mut idp_decoder = IDPDecoder::new( rdr ).unwrap();
    let decoding_result = idp_decoder.read_image().unwrap();

//    match decoding_result {
//        DecodingResult::U16(ref mut buffer) =>
//        {
//            // Make a U16 Image buffer
//        },
//        DecodingResult::F32(ref mut buffer) =>
//        {
//            // make a F32 Image buffer
//        },
//    } 

    Ok( decoding_result )
}


fn main() {
    let inpfile = Path::new( r#"dsr_test_f32.idp"# );
    make_test_idp( inpfile );
    read_test_idp( inpfile ).unwrap();
}
