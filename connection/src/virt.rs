use crate::*;
use std::io::{Error, Read, Write};
use std::fs::DirEntry;

pub enum VirtualMode {
    Dataset (DirEntry),
    Random { min: usize, max: usize },
    Constant (i64),
    Broken (Error), // Constantly returns err
    Empty,
}

pub struct VirtualConnection {
    mode: VirtualMode,
}

impl Read for VirtualConnection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.mode {
            VirtualMode::Constant(val) => {
                let bytes = val.to_le_bytes();
                for i in 0..bytes.len() {
                    buf[i] = bytes[i]
                }
            },
            _ => todo!(),
        }

        Ok(4)
    }
}

impl Write for VirtualConnection {
    fn write(&mut self, _buf: &[u8]) -> Result<usize, Error> {
        todo!();
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

// NOTE: This function currently assumes that all configuration is valid
// When adding errors to the library, make sure to provide detailed info when the configuration is invalid
// Also need to expand to support all virtual modes
impl Connect for VirtualConnection {
    fn connect(_settings: &ConnectionSettings) -> Self {
        VirtualConnection {
            mode: VirtualMode::Empty,
        }
    }
}
