use std::io::{Read, Result, Error, ErrorKind, Seek, SeekFrom};
use std::iter::repeat;
use std::mem::{forget, size_of};
use std::u64;

use byteorder::{ReadBytesExt, LittleEndian};

use flate2::{Decompress, Flush};

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Node {
    name: String,
    properties: Vec<Record>,
    subnodes: Vec<Node>,
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

fn decode_string<R: Read>(r : &mut R) -> Result<String> {
    match String::from_utf8(try!(decode_raw_array(r))) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8 Characters in String"))
    }
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
        let mut deflater = Decompress::new(true);

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

pub fn decode_record<R: Read>(r: &mut R) -> Result<Record> {
    match try!(r.read_u8()) {
        b'C' => Ok(Record::B(try!(r.read_u8()) == 1)),
        b'Y' => Ok(Record::I16(try!(r.read_i16::<LittleEndian>()))),
        b'I' => Ok(Record::I32(try!(r.read_i32::<LittleEndian>()))),
        b'L' => Ok(Record::I64(try!(r.read_i64::<LittleEndian>()))),
        b'F' => Ok(Record::F32(try!(r.read_f32::<LittleEndian>()))),
        b'D' => Ok(Record::F64(try!(r.read_f64::<LittleEndian>()))),
        b'R' => Ok(Record::RawArray(try!(decode_raw_array(r)))),
        b'S' => Ok(Record::String(try!(decode_string(r)))),
        b'b' =>  Ok(Record::BArray(try!(decode_array::<R, bool>(r)))),
        b'c' =>  Ok(Record::I8Array(try!(decode_array::<R, i8>(r)))),
        b'i' =>  Ok(Record::I32Array(try!(decode_array::<R, i32>(r)))),
        b'l' =>  Ok(Record::I64Array(try!(decode_array::<R, i64>(r)))),
        b'f' =>  Ok(Record::F32Array(try!(decode_array::<R, f32>(r)))),
        b'd' =>  Ok(Record::F64Array(try!(decode_array::<R, f64>(r)))),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid Record Type Marker"))
    }
}

pub fn decode_node<R: Read + Seek>(r: &mut R) -> Result<Option<Node>> {
    let end_offset = try!(r.read_u32::<LittleEndian>()) as u64;
    let property_number = try!(r.read_u32::<LittleEndian>()) as usize;
    let properties_size = try!(r.read_u32::<LittleEndian>()) as usize;
    let name_length = try!(r.read_u8()) as usize;

    // NULL node, end of node list
    if end_offset == 0 && property_number == 0 && properties_size == 0 && name_length == 0 {
        return Ok(None);
    }

    let mut name_buffer : Vec<u8> = repeat(0u8).take(name_length).collect();
    try!(r.read_exact(&mut name_buffer[..]));
    let name = match String::from_utf8(name_buffer.clone()) {
        Ok(s) => s,
        Err(_) => {
            println!("Name Buffer: {:?}", name_buffer);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8 Characters in Node Name"))
        }
    };

    let mut properties = Vec::<Record>::with_capacity(property_number);
    for _ in 0..property_number {
        properties.push(try!(decode_record(r)));
    }

    return Ok(Some(Node {
        name: name,
        properties: properties,
        subnodes: try!(decode_node_list(r, end_offset))
    }));
}

pub fn decode_node_list<R: Read + Seek>(r: &mut R, end : u64) -> Result<Vec<Node>> {
    let mut nodes = Vec::<Node>::new();
    while let Some(node) = try!(decode_node(r)) {
        nodes.push(node);
        let pos = r.seek(SeekFrom::Current(0))?;
        if pos >= end {
            break;
        }
    }
    return Ok(nodes);
}

pub fn decode_fbx<R: Read + Seek>(r: &mut R) -> Result<Vec<Node>> {
    let mut header = [0u8; 23];
    try!(r.read_exact(&mut header[..]));
    if &header != b"Kaydara FBX Binary  \x00\x1a\x00" {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid FBX Header Magic"));
    }
    let version = try!(r.read_u32::<LittleEndian>());
    return decode_node_list(r, u64::MAX);
}
