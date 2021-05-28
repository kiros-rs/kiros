use phf;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

fn main() {
    println!("{:?}", DYNAMIXELS.get("test").unwrap().address)
}
