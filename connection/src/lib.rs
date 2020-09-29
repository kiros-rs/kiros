pub trait ConnectionHandler {
    fn write(packet: Vec<u8>) -> Result<std::io::Error, u8>;
    fn read(buf: &mut Vec<u8>) -> Result<std::io::Error, ()>;
    // flush?
}

pub enum ConnectionType {
    USB,
    Analogue,
    Digital,
    Virtual(VirtualConnectionType),
}

pub enum VirtualConnectionType {
    Dataset(String),
    Random { min: usize, max: usize },
    Constant(usize),
}
