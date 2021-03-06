use crate::{ConnectionInfo, ConnectionType, WiredConnectionType};
use serialport::{SerialPort, TTYPort};
use std::time::Duration;

impl ConnectionInfo for TTYPort {
    fn get_connection_type(&self) -> ConnectionType {
        ConnectionType::Wired(WiredConnectionType::USB)
    }
}

// TODO: Change these params to suit abstract configuration?
pub fn connect_usb(path: &str, baudrate: u32) -> Box<dyn SerialPort> {
    // This config is too specific to dynamixels, maybe include in config
    serialport::new(path, baudrate)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(5))
        .open()
        .unwrap()
}

// TODO: Function that creates vec of all available ports
