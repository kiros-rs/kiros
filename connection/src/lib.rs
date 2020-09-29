pub trait ConnectionHandler {
    fn write(&self, packet: Vec<u8>) -> Result<std::io::Error, u8>;
    fn read(&self, buf: &mut Vec<u8>) -> Result<std::io::Error, ()>;
    // flush?
}

pub enum ConnectionType {
    USB,
    Analogue,
    Digital,
    Network(NetworkConnectionType),
    Virtual(VirtualConnectionType),
}

// Should expand this with mesh network systems
pub enum NetworkConnectionType {
    TCP,
    Bluetooth,
}

pub enum VirtualConnectionType {
    Dataset(String),
    Random { min: usize, max: usize },
    Constant(usize),
}
