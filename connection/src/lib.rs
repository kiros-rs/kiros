/// A generic API that exposes basic functionality from the method of
/// connection to enable abstraction over the connection method
pub trait ConnectionHandler {
    /// Write a packet over the connection
    fn write(&mut self, packet: Vec<u8>) -> Result<usize, std::io::Error>;
    /// Read a packet over the connection
    fn read(&mut self) -> Result<usize, std::io::Error>;
    /// Connect & perform any initialisation required
    fn connect(&mut self) -> Option<std::io::Error>;
}

/// Additional functionality available when accessing buffered connections
pub trait BufferedConnectionHandler {
    /// Flush the connection buffer
    fn flush(&mut self);
    /// Read a buffered packet over the connection
    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error>;
}

/// An API to get basic connection info
pub trait ConnectionInfo {
    fn get_connection_type(&self) -> ConnectionType;
    // connection configuration?
}

/// All the different methods of connection between nodes
pub enum ConnectionType {
    Wired(WiredConnectionType),
    Wireless(WirelessConnectionType),
    Virtual(VirtualConnectionType),
}

/// All the wired methods of connection between nodes
pub enum WiredConnectionType {
    USB,
    Ethernet,
    Pin(PinConnectionType),
}

/// All the pin-based methods of connection between nodes
pub enum PinConnectionType {
    Analogue,
    Digital,
}

/// All the wireless methods of connection between nodes
pub enum WirelessConnectionType {
    TCP,
    Bluetooth,
}

/// All the virtual methods of connection
pub enum VirtualConnectionType {
    Dataset(String),
    Random { min: usize, max: usize },
    Constant(usize),
}
