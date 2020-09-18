use std::collections::HashMap;

pub mod protocol_one;

// Extend this with protocol 2 packet
pub enum PacketProtocol {
    ProtocolOne(protocol_one::Packet),
}

pub enum Protocol {
    ProtocolOne,
    ProtocolTwo,
}

pub trait PortHandler {
    fn write(packet: Vec<u8>) -> Result<std::io::Error, u8>;
    fn read(buf: &mut Vec<u8>) -> Result<std::io::Error, ()>;
}

// Need to extend this further at some point & find better name
// Trait so we can have functionality from both protocol versions
pub trait PacketBuilder {
    fn checksum(&self) -> u8;
    fn build(&self) -> Result<Vec<u8>, String>;
}

pub struct Dynamixel {
    pub id: u8,
    pub protocol: Protocol,
    pub baudrate: usize,
    pub control_table: HashMap<u8, u8>, // This needs to become its own proper data structure
                                        /* Commenting these out until I figure out if they will be useful
                                        last_packet: PacketType
                                        sent_packets: Vec<u8>
                                        */
}
