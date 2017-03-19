use std::fs::File;

use decode_fbx;

#[test]
fn blender_cube() {
    let mut cube_file = File::open("testcases/cube.fbx").unwrap();
    let cube = decode_fbx(&mut cube_file).unwrap();
    println!("{:?}", cube);
}
