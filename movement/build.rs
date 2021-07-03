use phf_codegen::Map;
use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    // Clone the control tables
    Command::new("git")
        .args(&[
            "clone",
            "https://github.com/kiros-rs/control-tables.git",
            &format!("{}/control-tables", &env::var("OUT_DIR").unwrap()),
        ])
        .output()
        .unwrap();

    let mut phf: Map<_> = Map::new();
    let mut entries: Vec<walkdir::DirEntry> = Vec::new();
    let path = format!("{}/control-tables/objects", &env::var("OUT_DIR").unwrap());

    for e in WalkDir::new(&path) {
        let entry = e.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }

        entries.push(entry);
    }

    for entry in 0..entries.len() {
        let name = entries[entry]
            .file_name()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap();

        // Prepend a backslash to each double-quote (")
        let f = read_to_string(entries[entry].path()).unwrap();
        let val = f.split('"').collect::<Vec<&str>>().join("\\\"");

        phf.entry(name, &format!("\"{}\"", val));
    }

    writeln!(
        &mut file,
        "static DYNAMIXELS: phf::Map<&'static str, &'static str> = \n{};\n",
        phf.build()
    )
    .unwrap();

    // These could be turned into a programmatically generated enum
    writeln!(
        &mut file,
        "static CONTROL_TABLE_TYPES: phf::Map<&'static str, ControlTableType> = \n{};\n",
        Map::new()
            .entry("Alarm LED", "ControlTableType::Component")
            .entry("Baud Rate", "ControlTableType::Value")
            .entry("CCW Angle Limit", "ControlTableType::Value")
            .entry("CCW Compliance Margin", "ControlTableType::Value")
            .entry("CCW Compliance Slope", "ControlTableType::Value")
            .entry("CW Angle Limit", "ControlTableType::Value")
            .entry("CW Compliance Margin", "ControlTableType::Value")
            .entry("CW Compliance Slope", "ControlTableType::Value")
            .entry("Firmware Version", "ControlTableType::Value")
            .entry("Goal Position", "ControlTableType::Value")
            .entry("ID", "ControlTableType::Value")
            .entry("LED", "ControlTableType::Component")
            .entry("Lock", "ControlTableType::Value")
            .entry("Max Torque", "ControlTableType::Value")
            .entry("Max Voltage Limit", "ControlTableType::Value")
            .entry("Min Voltage Limit", "ControlTableType::Value")
            .entry("Model Number", "ControlTableType::Value")
            .entry("Moving", "ControlTableType::Value")
            .entry("Moving Speed", "ControlTableType::Sensor")
            .entry("Present Load", "ControlTableType::Sensor")
            .entry("Present Position", "ControlTableType::Sensor")
            .entry("Present Speed", "ControlTableType::Sensor")
            .entry("Present Temperature", "ControlTableType::Sensor")
            .entry("Present Voltage", "ControlTableType::Sensor")
            .entry("Punch", "ControlTableType::Sensor")
            .entry("Registered", "ControlTableType::Value")
            .entry("Return Delay Time", "ControlTableType::Value")
            .entry("Shutdown", "ControlTableType::Component")
            .entry("Status Return Level", "ControlTableType::Value")
            .entry("Temperature Limit", "ControlTableType::Value")
            .entry("Torque Enable", "ControlTableType::Value")
            .entry("Torque Limit", "ControlTableType::Value")
            .build()
    )
    .unwrap();
}
