use connection::ConnectionHandler;
use sensor::DataSensor;
use std::collections::HashMap;

pub mod protocol_one;

// Extend this with protocol 2 packet
pub enum Packet {
    ProtocolOne(protocol_one::Packet),
}

pub enum Protocol {
    One,
    Two,
}

pub enum ControlTableType {
    Sensor,
    ServoInformation,
    Component,
    Constraint,
}

pub enum AccessLevel {
    Read,
    Write,
    ReadWrite,
}

pub struct ControlTableData {
    pub address: u8,
    pub size: u8,
    pub description: Option<String>,
    pub access: AccessLevel,
    pub initial_value: Option<String>,
    pub range: Option<(u8, u8)>,
    pub units: Option<sensor::DataUnit>,
}

pub struct Dynamixel {
    pub connection_handler: Box<dyn ConnectionHandler>,
    pub control_table: HashMap<String, ControlTableType>,
    pub sensors: HashMap<String, Box<dyn DataSensor<isize>>>,
    pub components: HashMap<String, ()>, // should become a custom datatype/enum
    pub information: HashMap<String, ControlTableData>,
    pub constraints: HashMap<String, ControlTableData>,
    /* Commenting these out until I figure out if they will be useful
    last_packet: PacketType
    sent_packets: Vec<u8>
    */
}

pub enum DynamixelMode {
    Wheel,
    Joint,
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

// paramters should also include implementor of ConnectionHandler
pub trait ProtocolOne {
    fn ping(&mut self) -> Packet;
    fn read(&self, address: u8) -> Packet;
    fn write(&mut self, address: u8, value: u8) -> Packet;
    fn register_write(&mut self, address: u8, value: u8) -> Packet;
    fn action(&mut self) -> Packet;
    fn reset(&mut self) -> Packet;
    fn reboot(&mut self) -> Packet;
    fn sync_write(&mut self, address: u8, value: u8) -> Packet;
    fn bulk_read(&self) -> Vec<Packet>;
}
