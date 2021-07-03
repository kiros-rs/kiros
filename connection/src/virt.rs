use crate::*;
use std::fs::DirEntry;
use std::io::{Error, Read, Write};

pub enum VirtualMode {
    Dataset(DirEntry),
    Random { min: usize, max: usize },
    Constant(i64),
    Broken(Error), // Constantly returns err
    Loop,
    Output,
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
                buf[..bytes.len()].clone_from_slice(&bytes[..]);
            }
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
