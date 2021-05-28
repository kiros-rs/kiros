use phf_codegen;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    writeln!(
        &mut file,
         "use movement::dynamixel::{{AccessLevel, ControlTableData}};\n\
         static DYNAMIXELS: phf::Map<&'static str, ControlTableData<u8>> = \n{};\n",
         phf_codegen::Map::new()
             .entry("test", "ControlTableData {
                address: 1,
                size: 1,
                data_name: None,
                description: None,
                access: AccessLevel::ReadWrite,
                initial_value: None,
                range: None,
                units: None,
                modbus: None,
            }")
             .build()
    ).unwrap();
}
