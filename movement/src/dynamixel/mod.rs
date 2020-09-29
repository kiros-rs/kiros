use std::collections::HashMap;

pub mod protocol_one;

// Extend this with protocol 2 packet
pub enum PacketType {
    ProtocolOne(protocol_one::Packet),
}

pub enum Protocol {
    ProtocolOne,
    ProtocolTwo,
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
/*
Note to future self:
The control table should be populated with not just string associations,
but a more complex mapping between name & data stored.

There should be an enum with a few different kinds of items:
- Sensor (temperature, voltage etc)
- Stored value (id, model etc)
- Component (led, alarm etc)
- Constraint (cw limit, compliance margin)
*/
