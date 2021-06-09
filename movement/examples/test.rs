use phf;
use ron::de::from_str;

use movement::dynamixel::{AccessLevel, ControlTableData, DynamixelAddress};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

fn main() {
    let dxl: &str = DYNAMIXELS.get("ax-12a").unwrap();
    let data = from_str::<'_, Vec<ControlTableData<u64>>>(dxl).unwrap();

    for line in data {
        println!("{:?}", line.data_name);
    }
}
