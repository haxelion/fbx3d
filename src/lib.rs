//! The `fbx` crate allow to parse and load `fbx` 3D model files. It is based on the `blender` 
//! implementation of the `FBX` file format.

extern crate bytepack;
extern crate flate2;

use std::io::{Read, Result, Error, ErrorKind, Seek};
use std::u64;

use bytepack::LEUnpacker;

pub mod types;

use types::{Node, decode_node_list};

/// Decode a FBX file to a [`Node`](types/struct.Node.html) hierarchy.
pub fn decode_fbx<R: Read + Seek>(r: &mut R) -> Result<Vec<Node>> {
    let mut header = [0u8; 23];
    r.read_exact(&mut header[..])?;
    if &header != b"Kaydara FBX Binary  \x00\x1a\x00" {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid FBX header magic"));
    }
    let version = r.unpack::<u32>()?;
    if version >= 7500 {
        return Err(Error::new(ErrorKind::InvalidData, "Unsuported FBX version"));
    }
    return decode_node_list(r, u64::MAX);
}

#[cfg(test)]
mod tests;
