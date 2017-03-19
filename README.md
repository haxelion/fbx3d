fbx
===

[![Crates.io](https://img.shields.io/crates/v/fbx.svg)](https://crates.io/crates/fbx)
[![Build Status](https://travis-ci.org/haxelion/fbx.svg?branch=master)](https://travis-ci.org/haxelion/fbx)
[![Docs.rs](https://docs.rs/fbx/badge.svg)](https://docs.rs/fbx)

The `fbx` crate allow to parse and load `fbx` 3D model files. It is based on the `blender` 
implementation of the `FBX` file format.

It is currently only able to parse the file format into a Rust representation. I plan to write 
helpers to extract rendering information (VAO with UV and normals, indices, animation key, ...) 
from that structure.

In the future, writing `FBX` file could also be implemented, feel free to contribute.
