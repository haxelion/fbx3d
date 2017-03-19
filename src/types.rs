use std::io::{Read, Result, Error, ErrorKind, Seek, SeekFrom};
use std::iter::repeat;
use std::mem::{forget, size_of, zeroed};

use bytepack::{LEUnpacker, Packed};

use flate2::{Decompress, Flush};

/// Represent a typed property of the FBX file format.
#[derive(Clone, Debug)]
pub enum Property {
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
    I32Array(Vec<i32>),
    I64Array(Vec<i64>),
    F32Array(Vec<f32>),
    F64Array(Vec<f64>),
}

/// Represent a Node of the FBX file format. Each node has a `name` (or id) and is composed of a 
/// list of [`Property`](enum.Property.html) and a list of sub [`Node`](struct.Node.html).
#[derive(Clone, Debug)]
pub struct Node {
    /// Name of the node.
    pub name: String,
    /// List of properties of the node.
    pub properties: Vec<Property>,
    /// List of sub-nodes.
    pub subnodes: Vec<Node>,
}

/// Decode a `Property::RawArray`.
fn decode_raw_array<R: Read>(r : &mut R) -> Result<Vec<u8>> {
    let length = r.unpack::<u32>()? as usize;
    let mut array : Vec<u8> = repeat(0u8).take(length).collect();
    r.read_exact(&mut array[..])?;
    return Ok(array);
}

/// Decode a `Property::String`.
fn decode_string<R: Read>(r : &mut R) -> Result<String> {
    match String::from_utf8(decode_raw_array(r)?) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8 characters in string"))
    }
}

/// Decode a `Property::*Array`.
fn decode_array<R: Read, T: Packed + Clone>(r : &mut R) -> Result<Vec<T>> {
    let length = r.unpack::<u32>()? as usize;
    let encoding = r.unpack::<u32>()?;
    let compressed_length = r.unpack::<u32>()? as usize;
    if encoding == 0 {
        let zero : T = unsafe { zeroed() };
        let mut array : Vec<T> = repeat(zero).take(length).collect();
        r.unpack_exact(&mut array[..])?;
        return Ok(array);
    }
    else if encoding == 1 {
        let mut compressed : Vec<u8> = repeat(0u8).take(compressed_length).collect();
        let mut decompressed : Vec<u8> = repeat(0u8).take(length * size_of::<T>()).collect();
        let mut deflater = Decompress::new(true);

        r.read_exact(&mut compressed)?;
        if let Err(_) = deflater.decompress(&compressed, &mut decompressed, Flush::Finish) {
            return Err(Error::new(ErrorKind::InvalidData, "Failed to deflate array"));
        }

        // Safe because we made sure the length and capacity of decompressed is length * size_of::<T>() 
        // and we properly forget decompressed
        decompressed.shrink_to_fit();
        unsafe {
            let converted = Vec::<T>::from_raw_parts(decompressed.as_mut_ptr() as *mut T, length, length);
            forget(decompressed);
            return Ok(converted);
        }
    }
    else {
        return Err(Error::new(ErrorKind::InvalidData, "Unknown array encoding"));
    }
}

/// Decode a [`Property`](enum.Property.html).
pub fn decode_property<R: Read>(r: &mut R) -> Result<Property> {
    match r.unpack()? {
        b'C' => Ok(Property::B(r.unpack::<u8>()? == 1)),
        b'Y' => Ok(Property::I16(r.unpack()?)),
        b'I' => Ok(Property::I32(r.unpack()?)),
        b'L' => Ok(Property::I64(r.unpack()?)),
        b'F' => Ok(Property::F32(r.unpack()?)),
        b'D' => Ok(Property::F64(r.unpack()?)),
        b'R' => Ok(Property::RawArray(decode_raw_array(r)?)),
        b'S' => Ok(Property::String(decode_string(r)?)),
        b'b' =>  Ok(Property::BArray(decode_array::<R, bool>(r)?)),
        b'c' =>  Ok(Property::I8Array(decode_array::<R, i8>(r)?)),
        b'i' =>  Ok(Property::I32Array(decode_array::<R, i32>(r)?)),
        b'l' =>  Ok(Property::I64Array(decode_array::<R, i64>(r)?)),
        b'f' =>  Ok(Property::F32Array(decode_array::<R, f32>(r)?)),
        b'd' =>  Ok(Property::F64Array(decode_array::<R, f64>(r)?)),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid property type marker"))
    }
}

/// Decode a [`Node`](struct.Node.html).
pub fn decode_node<R: Read + Seek>(r: &mut R) -> Result<Option<Node>> {
    let end_offset = r.unpack::<u32>()? as u64;
    let property_number = r.unpack::<u32>()? as usize;
    let properties_size = r.unpack::<u32>()? as usize;
    let name_length = r.unpack::<u8>()? as usize;

    // NULL node, end of node list
    if end_offset == 0 && property_number == 0 && properties_size == 0 && name_length == 0 {
        return Ok(None);
    }

    let mut name_buffer : Vec<u8> = repeat(0u8).take(name_length).collect();
    r.read_exact(&mut name_buffer[..])?;
    let name = match String::from_utf8(name_buffer.clone()) {
        Ok(s) => s,
        Err(_) => {
            println!("Name Buffer: {:?}", name_buffer);
            return Err(Error::new(ErrorKind::InvalidData, "Invalid UTF-8 characters in node name"))
        }
    };

    let mut properties = Vec::<Property>::with_capacity(property_number);
    for _ in 0..property_number {
        properties.push(decode_property(r)?);
    }

    return Ok(Some(Node {
        name: name,
        properties: properties,
        subnodes: decode_node_list(r, end_offset)?
    }));
}

/// Decode a list of [`Node`](struct.Node.html).
pub fn decode_node_list<R: Read + Seek>(r: &mut R, end : u64) -> Result<Vec<Node>> {
    let mut nodes = Vec::<Node>::new();
    while let Some(node) = decode_node(r)? {
        nodes.push(node);
        let pos = r.seek(SeekFrom::Current(0))?;
        if pos >= end {
            break;
        }
    }
    return Ok(nodes);
}
