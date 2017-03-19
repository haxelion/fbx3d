fbx3d
=====

[![Crates.io](https://img.shields.io/crates/v/fbx3d.svg)](https://crates.io/crates/fbx3d)
[![Build Status](https://travis-ci.org/haxelion/fbx3d.svg?branch=master)](https://travis-ci.org/haxelion/fbx3d)
[![Docs.rs](https://docs.rs/fbx3d/badge.svg)](https://docs.rs/fbx3d)

The `fbx3d` crate allow to parse and load `FBX` 3D model files. It is based on the `blender` 
implementation of the `FBX` file format.

It is currently only able to parse the file format into a Rust representation. I plan to write 
helpers to extract rendering information (VAO with UV and normals, indices, animation key, ...) 
from that structure.

In the future, writing `FBX` file could also be implemented, feel free to contribute.
