pub mod usb;

/// A generic API that exposes basic functionality from the method of
/// connection to enable abstraction over the connection method
pub trait ConnectionHandler {
    fn write(&mut self, packet: Vec<u8>) -> Result<usize, std::io::Error>;
    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error>;
    // fn open(&mut self, port: &str, baudrate: usize) -> Result<Box<dyn ConnectionHandler>, String>; // This needs to have an abstract way of passing location (port, pins etc)
    fn get_connection_type(&self) -> ConnectionType;
    // flush?
}

/// All different methods of connection
pub enum ConnectionType {
    USB,
    Analogue,
    Digital,
    Network(NetworkConnectionType),
    Virtual(VirtualConnectionType),
}

/// All different methods of connection over a network
pub enum NetworkConnectionType {
    TCP,
    Bluetooth,
    // Should expand this with mesh network systems
}

/// All different methods of virtual connection
pub enum VirtualConnectionType {
    Dataset(String),
    Random { min: usize, max: usize },
    Constant(usize),
}
