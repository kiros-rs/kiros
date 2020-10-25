use connection::ConnectionHandler;
use sensor::DataSensor;
use std::collections::HashMap;

pub mod protocol_one;

// Extend this with protocol 2 packet when implemented
pub enum Packet {
    ProtocolOne(protocol_one::Packet),
}

/// The abstract categories an item in the control table
/// can be part of.
pub enum ControlTableType {
    Sensor,
    ServoInformation,
    Component,
    Constraint,
}

/// The levels of permission a user is granted in terms of an item in the
/// control table.
pub enum AccessLevel {
    Read,
    Write,
    ReadWrite,
}

/// A representation of an item in the control table, where only information
/// is stored. When applicable, items in the control table are represented in
/// this format, along with any optional data such as range or description.
pub struct ControlTableData {
    pub address: u8,
    pub size: u8,
    pub description: Option<String>,
    pub access: AccessLevel,
    pub initial_value: Option<String>,
    pub range: Option<(u8, u8)>,
    pub units: Option<sensor::DataUnit>,
}

/// An abstract representation of a Dynamixel servo
/// The servo contains the following basic types of data in its control table:
/// - Sensor (temperature, voltage)
/// - Servo Information (model, id)
/// - Component (led, alarm)
/// - Constraint (cw limit, max speed)
/// The servo stores this abstracted representation of its control table
/// within the aforementioned fields. Additionally, the structure stores
/// an index of the control table (based on the data name column) to enable
/// users to quickly locate a categorised item programmatically.
///
/// Finally, the Dynamixel structure stores a list of packets if the
/// `collects_packets` boolean is set to true.
pub struct Dynamixel {
    pub connection_handler: Box<dyn ConnectionHandler>,
    pub control_table: HashMap<String, ControlTableType>,
    pub sensors: HashMap<String, Box<dyn DataSensor<isize>>>,
    pub components: HashMap<String, ()>, // should become a custom datatype/enum
    pub information: HashMap<String, ControlTableData>,
    pub constraints: HashMap<String, ControlTableData>,
    pub last_packet: Packet,
    pub sent_packets: Vec<Packet>,
    pub collects_packets: bool,
}

/// A Dynamixel can be either a wheel (CW & CCW limits set to 0) or a joint
/// (CW or CCW limits nonzero). This enum contains a representation of these 2 states.
pub enum DynamixelMode {
    Wheel,
    Joint,
}

/// Packets can either be addressed to a single Dynamixel or all dynamixels
pub enum DynamixelID {
    Broadcast,
    ID(u8),
}

pub trait PacketCommunications {
    fn write(data: Vec<u8>) -> Packet;
    fn read() -> Packet;
}

impl PacketCommunications for Dynamixel {
    fn write(data: Vec<u8>) -> Packet {
        unimplemented!();
    }
    fn read() -> Packet {
        unimplemented!();
    }
}

/// This trait exposes all functionality possessed by Protocol One servos. It
/// is worth noting that bulk_read is only available to MX series servos, and
/// all other models should produce an error when called.
pub trait ProtocolOne {
    fn ping(&mut self, id: DynamixelID) -> Packet;
    // fn read(&self, address: u8) -> Packet;
    // fn write(&mut self, address: u8, value: u8) -> Packet;
    // fn register_write(&mut self, address: u8, value: u8) -> Packet;
    // fn action(&mut self) -> Packet;
    // fn reset(&mut self) -> Packet;
    // fn reboot(&mut self) -> Packet;
    // fn sync_write(&mut self, address: u8, value: u8) -> Packet;
    // fn bulk_read(&self) -> Result<Vec<Packet>, String>;
}

impl ProtocolOne for Dynamixel {
    fn ping(&mut self, id: DynamixelID) -> Packet {
        let dxl_id: u8 = match id {
            DynamixelID::Broadcast => 0xFE,
            DynamixelID::ID(val) => val,
        };

        let pck = protocol_one::Packet::new(dxl_id, protocol_one::PacketType::Instruction(protocol_one::InstructionType::Ping), vec![]);
        println!("{:?}", pck.generate());

        Packet::ProtocolOne(pck)
    }
    // fn read(&self, address: u8) -> Packet;
    // fn write(&mut self, address: u8, value: u8) -> Packet;
    // fn register_write(&mut self, address: u8, value: u8) -> Packet;
    // fn action(&mut self) -> Packet;
    // fn reset(&mut self) -> Packet;
    // fn reboot(&mut self) -> Packet;
    // fn sync_write(&mut self, address: u8, value: u8) -> Packet;
    // fn bulk_read(&self) -> Result<Vec<Packet>, String>;
}
