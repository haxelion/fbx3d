use std::fs::File;

use decode_fbx;

#[test]
fn blender_cube() {
    let mut cube_file = File::open("testcases/cube.fbx").unwrap();
    let cube = decode_fbx(&mut cube_file).unwrap();
    println!("{:?}", cube);
}

#[test]
fn blender_multiples() {
    let mut multiples_file = File::open("testcases/multiples.fbx").unwrap();
    let multiples = decode_fbx(&mut multiples_file).unwrap();
    println!("{:?}", multiples);
}

