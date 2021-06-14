pub mod virt;
pub mod wired;

use std::io::{Read, Write};
use std::time::{Duration, Instant};

// Add send/sync to this?
pub trait Connect: Read + Write {
    fn connect(settings: &ConnectionSettings) -> Self
    where
        Self: Sized; // Is this the best way to do it?
}

#[derive(Clone, Debug)]
pub struct ConnectionSettings {
    baudrate: Option<usize>,
    path: Option<String>,
    timeout: Option<Duration>,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        ConnectionSettings {
            baudrate: Some(1_000_000),
            path: Some(String::from("/dev/ttyACM1")), // This should be programatically found (based on target triple?)
            timeout: Some(Duration::from_millis(5)),
        }
    }
}

pub struct Connection {
    pub connected_at: Option<Instant>,
    pub connection_type: ConnectionType,
    pub connection_method: ConnectionMethod,
    pub connection_mode: ConnectionMode,
    pub interface: Box<dyn Connect>,
    pub connection_settings: ConnectionSettings,
}

impl Connection {
    pub fn new(
        connection_type: ConnectionType,
        connection_method: ConnectionMethod,
        connection_settings: ConnectionSettings,
    ) -> Self {
        Connection {
            connected_at: Some(Instant::now()),
            connection_type,
            connection_method,
            interface: match connection_method {
                ConnectionMethod::USB => Box::new(wired::TTYPort::connect(&connection_settings)),
                _ => todo!(),
            },
            connection_settings,
            connection_mode: ConnectionMode::Polling, // This needs to be properly implemented
        }
    }
}

/// All the different methods of connection between nodes
#[derive(Clone, Copy, Debug)]
pub enum ConnectionType {
    Wired,
    Wireless,
    Virtual,
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionMethod {
    USB,
    Ethernet,
    AnaloguePin,
    DigitalPin,
    TCP,
    Bluetooth,
    Virtual,
}

// This needs some work
#[derive(Clone, Copy, Debug)]
pub enum ConnectionMode {
    Generated,
    Polling,
}
