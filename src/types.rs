use std::io::{Read, Result, Error, ErrorKind};
use std::iter::repeat;
use std::mem::{forget, size_of};

use byteorder::{ReadBytesExt, LittleEndian};

use flate2::{Decompress, Flush};

pub enum Record {
    B(bool),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    RawArray(Vec<u8>),
    String(String),
    BArray(Vec<bool>),
    I8Array(Vec<i8>),
    I16Array(Vec<i16>),
    I32Array(Vec<i32>),
    I64Array(Vec<i64>),
    F32Array(Vec<f32>),
    F64Array(Vec<f64>),
}

trait Decodable : Sized {
    fn decode<R : Read>(r : &mut R) -> Result<Self>;
}

impl Decodable for bool {
    fn decode<R : Read>(r : &mut R) -> Result<bool> {
        Ok(try!(r.read_u8()) == 1)
    }
}

impl Decodable for i8 {
    fn decode<R : Read>(r : &mut R) -> Result<i8> {
        Ok(try!(r.read_i8()))
    }
}

impl Decodable for i16 {
    fn decode<R : Read>(r : &mut R) -> Result<i16> {
        Ok(try!(r.read_i16::<LittleEndian>()))
    }
}

impl Decodable for i32 {
    fn decode<R : Read>(r : &mut R) -> Result<i32> {
        Ok(try!(r.read_i32::<LittleEndian>()))
    }
}

impl Decodable for i64 {
    fn decode<R : Read>(r : &mut R) -> Result<i64> {
        Ok(try!(r.read_i64::<LittleEndian>()))
    }
}

impl Decodable for f32 {
    fn decode<R : Read>(r : &mut R) -> Result<f32> {
        Ok(try!(r.read_f32::<LittleEndian>()))
    }
}

impl Decodable for f64 {
    fn decode<R : Read>(r : &mut R) -> Result<f64> {
        Ok(try!(r.read_f64::<LittleEndian>()))
    }
}

fn decode_raw_array<R: Read>(r : &mut R) -> Result<Vec<u8>> {
    let length = try!(r.read_u32::<LittleEndian>()) as usize;
    let mut array : Vec<u8> = repeat(0u8).take(length).collect();
    try!(r.read_exact(&mut array[..]));
    return Ok(array);
}

fn decode_array<R: Read, T : Decodable>(r : &mut R) -> Result<Vec<T>> {
    let length = try!(r.read_u32::<LittleEndian>()) as usize;
    let encoding = try!(r.read_u32::<LittleEndian>());
    let compressed_length = try!(r.read_u32::<LittleEndian>()) as usize;
    if encoding == 0 {
        let mut array = Vec::<T>::with_capacity(length);
        for _ in 0..length {
            array.push(try!(T::decode(r)));
        }
        return Ok(array);
    }
    else if encoding == 1 {
        let mut compressed : Vec<u8> = repeat(0u8).take(compressed_length).collect();
        let mut decompressed : Vec<u8> = repeat(0u8).take(length * size_of::<T>()).collect();
        let mut deflater = Decompress::new(false);

        try!(r.read_exact(&mut compressed));
        if let Err(_) = deflater.decompress(&compressed, &mut decompressed, Flush::Finish) {
            return Err(Error::new(ErrorKind::InvalidData, "Failed to Deflate Array"));
        }

        // Safe because we made sure the length and capacity of decompressed is length * size_of::<T>() 
        // and we properly forget decompressed
        decompressed.shrink_to_fit();
        unsafe {
            let mut converted = Vec::<T>::from_raw_parts(decompressed.as_mut_ptr() as *mut T, length, length);
            forget(decompressed);
            return Ok(converted);
        }
    }
    else {
        return Err(Error::new(ErrorKind::InvalidData, "Unknown Array Encoding"));
    }
}

pub fn decode_record<R: Read>(r : &mut R) -> Result<Record> {
    match try!(r.read_u8()) {
        b'C' => Ok(Record::B(try!(r.read_u8()) == 1)),
        b'Y' => Ok(Record::I16(try!(r.read_i16::<LittleEndian>()))),
        b'I' => Ok(Record::I32(try!(r.read_i32::<LittleEndian>()))),
        b'L' => Ok(Record::I64(try!(r.read_i64::<LittleEndian>()))),
        b'F' => Ok(Record::F32(try!(r.read_f32::<LittleEndian>()))),
        b'D' => Ok(Record::F64(try!(r.read_f64::<LittleEndian>()))),
        b'R' => Ok(Record::RawArray(try!(decode_raw_array(r)))),
        b'S' => match String::from_utf8(try!(decode_raw_array(r))) {
            Ok(s) => Ok(Record::String(s)),
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8 Characters in String"))
        },
        b'b' =>  Ok(Record::BArray(try!(decode_array::<R, bool>(r)))),
        b'c' =>  Ok(Record::I8Array(try!(decode_array::<R, i8>(r)))),
        b'i' =>  Ok(Record::I32Array(try!(decode_array::<R, i32>(r)))),
        b'l' =>  Ok(Record::I64Array(try!(decode_array::<R, i64>(r)))),
        b'f' =>  Ok(Record::F32Array(try!(decode_array::<R, f32>(r)))),
        b'd' =>  Ok(Record::F64Array(try!(decode_array::<R, f64>(r)))),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Record Type Marker"))
    }
}
