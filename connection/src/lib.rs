/// A generic API that exposes basic functionality from the method of
/// connection to enable abstraction over the connection method
pub trait ConnectionHandler {
    fn write(&self, packet: Vec<u8>) -> Result<std::io::Error, u8>;
    fn read(&self, buf: &mut Vec<u8>) -> Result<std::io::Error, ()>;
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
