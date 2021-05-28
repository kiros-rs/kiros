use phf;
use serde_yaml;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

fn main() {
    let dxl: &ControlTableData<u8> = DYNAMIXELS.get("test").unwrap();
    println!("{:?}", DYNAMIXELS.get("test").unwrap());
    let servos = vec![dxl];
    println!("{}", serde_yaml::to_string(&servos).unwrap());
}
