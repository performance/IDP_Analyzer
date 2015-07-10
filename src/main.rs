extern crate byteorder;
use std::io::BufWriter;
use std::fs::File;
use std::io::{Write};
// use byteorder::{ ReadBytesExt, BigEndian, LittleEndian};
mod stream;
mod decoder; 
 
use stream::{
    ByteOrder,
    EndianWriter,
    SmartWriter
};




fn main() {
    println!("This shold create a test IDP file!");
    let inpfile = r#"dsr_test_f32.idp"#;
    let f = match File::create( inpfile ) {
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
