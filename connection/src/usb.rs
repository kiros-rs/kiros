use super::{ConnectionHandler, ConnectionType};
use serialport::TTYPort;

impl ConnectionHandler for TTYPort {
    fn write(&mut self, packet: Vec<u8>) -> Result<usize, std::io::Error> {
        <TTYPort as std::io::Write>::write(self, &packet)
    }

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        match <TTYPort as std::io::Read>::read(self, buf) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(e),
        }
    }

    // fn open(&mut self, port: &str, baud_rate: usize) -> Result<Box<dyn ConnectionHandler>, String> {
    //     match serialport::new(port, baud_rate as u32).open() {
    //         Ok(port) => Ok(Box::new(<port as ConnectionHandler>)),
    //         Err(e) => Err(e.to_string()),
    //     }
    // }

    fn get_connection_type(&self) -> crate::ConnectionType {
        ConnectionType::USB
    }
}
