use super::{ConnectionHandler, ConnectionType, VirtualConnectionType};

pub struct VirtualConnection {
    connection_type: VirtualConnectionType,
}

impl ConnectionHandler for VirtualConnection {
    fn write(&mut self, packet: Vec<u8>) -> Result<usize, std::io::Error> {
        println!("{:?}", packet);

        Ok(0)
    }

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        match self.connection_type {
            VirtualConnectionType::Constant(val) => Ok(val),
            _ => unimplemented!(),
        }
    }

    fn get_connection_type(&self) -> ConnectionType {
        ConnectionType::Virtual
    }
}
