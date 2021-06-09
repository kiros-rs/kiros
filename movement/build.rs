use phf_codegen;
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

    let mut phf: phf_codegen::Map<_> = phf_codegen::Map::new();
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
}
