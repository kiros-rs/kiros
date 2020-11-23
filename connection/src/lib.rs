pub mod usb;

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
