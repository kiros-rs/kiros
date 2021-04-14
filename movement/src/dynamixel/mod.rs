pub mod protocol_one;
pub mod servo_connection;

use std::collections::HashMap;

use sensor::DataSensor;
use std::io::{Read, Write};

use byteorder::{LittleEndian, WriteBytesExt};

// Extend this with protocol 2 packet when implemented
/// A protocol-agnostic representation of a Dynamixel packet
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
///
/// Note that if you wish to broadcast to all servos, you will need to create
/// an empty Dynamixel
pub struct Dynamixel<C: Read + Write> {
    pub connection_handler: Box<C>,
    pub control_table: HashMap<String, ControlTableType>,
    pub sensors: HashMap<String, Box<dyn DataSensor<isize>>>,
    pub components: HashMap<String, ()>, // should become a custom datatype/enum
    pub information: HashMap<String, ControlTableData>,
    pub constraints: HashMap<String, ControlTableData>,
    pub last_packet: Option<Packet>,
    pub sent_packets: Vec<Packet>,
    pub collects_packets: bool,
}

impl<C> Dynamixel<C>
where
    C: Read + Write,
{
    /// Create a new Dynamixel servo
    pub fn new(
        connection_handler: C,
        control_table: HashMap<String, ControlTableType>,
        sensors: HashMap<String, Box<dyn DataSensor<isize>>>,
        information: HashMap<String, ControlTableData>,
        constraints: HashMap<String, ControlTableData>,
        collects_packets: bool,
    ) -> Self {
        Dynamixel {
            connection_handler: Box::new(connection_handler),
            control_table,
            sensors,
            components: HashMap::new(),
            information,
            constraints,
            last_packet: None,
            sent_packets: vec![],
            collects_packets,
        }
    }

    // HACK: Should be removed when sensors are completed
    pub fn new_empty(connection_handler: C) -> Self {
        Dynamixel {
            connection_handler: Box::new(connection_handler),
            control_table: HashMap::new(),
            sensors: HashMap::new(),
            components: HashMap::new(),
            information: HashMap::new(),
            constraints: HashMap::new(),
            last_packet: None,
            sent_packets: vec![],
            collects_packets: false,
        }
    }
}

/// A representation of the 2 movement states a Dynamixel can be in:
/// - Wheel
/// - Joint
pub enum DynamixelMode {
    Wheel,
    Joint,
}

/// Packets can either be addressed to a single Dynamixel or all dynamixels
#[derive(Clone, Copy, PartialEq, Eq, Hash)] // TODO: Remove these derives
pub enum DynamixelID {
    Broadcast,
    ID(u8),
}

// Consider using the num-traits crate to make this more broad?
impl From<DynamixelID> for u8 {
    fn from(item: DynamixelID) -> u8 {
        match item {
            DynamixelID::Broadcast => 0xFE,
            DynamixelID::ID(id) => id,
        }
    }
}

// TODO: Rename this to something better
pub trait PacketManipulation {
    fn checksum(id: &u8, length: &u8, parameters: &Vec<u8>, opcode: &u8) -> u8;
    fn generate(&self) -> Result<Vec<u8>, String>;
}

// Remove 'get' prefix?
pub trait DynamixelInformation {
    fn get_id(&self) -> DynamixelID;
    // fn get_baudrate(&self) -> u64;
}

// return values should be wrappen in Option
impl<C> DynamixelInformation for Dynamixel<C>
where
    C: Read + Write,
{
    fn get_id(&self) -> DynamixelID {
        // Temporary until this can be properly implemented later
        DynamixelID::ID(1)
    }
}

/// A packet used to address the same instruction to a group of servos
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyncPacket {
    pub id: u8, // Cannot be DynamixelID enum as only non-broadcast IDs allowed
    pub data: u64,
    pub address: u8,
}

/// A packet used to read from multiple servos at the same time (MX series only)
pub struct BulkReadPacket {
    pub id: u8,
    pub length: u8,
    pub address: u8,
}

// TODO: convert this into a procedural macro that parses the enum and removes need for macro calls
#[macro_export]
macro_rules! impl_to_databytes {
    ($t: ty, $enum:ident $(:: $enum_path:ident)*) => {
        impl From<$t> for DataBytes {
            fn from(num: $t) -> DataBytes {
                $enum $(:: $enum_path)* (num)
            }
        }
    };
}

#[derive(Debug)]
pub enum DataBytes {
    One(u8),
    OneSigned(i8),
    Two(u16),
    TwoSigned(i16),
    Four(u32),
    FourSigned(i32),
}

// This should also be incorporated into the proc macro
impl From<DataBytes> for Vec<u8> {
    fn from(bytes: DataBytes) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        match bytes {
            DataBytes::One(n) => buf.write_u8(n).unwrap(),
            DataBytes::OneSigned(n) => buf.write_i8(n).unwrap(),
            DataBytes::Two(n) => buf.write_u16::<LittleEndian>(n).unwrap(),
            DataBytes::TwoSigned(n) => buf.write_i16::<LittleEndian>(n).unwrap(),
            DataBytes::Four(n) => buf.write_u32::<LittleEndian>(n).unwrap(),
            DataBytes::FourSigned(n) => buf.write_i32::<LittleEndian>(n).unwrap(),
        }

        buf
    }
}

// impl From<&DataBytes> for Vec<u8> {
//     fn from(bytes: &DataBytes) -> Vec<u8> {
//         let mut buf: Vec<u8> = Vec::new();
//         match bytes {
//             DataBytes::One(n) => buf.write_u8(*n).unwrap(),
//             DataBytes::OneSigned(n) => buf.write_i8(*n).unwrap(),
//             DataBytes::Two(n) => buf.write_u16::<LittleEndian>(*n).unwrap(),
//             DataBytes::TwoSigned(n) => buf.write_i16::<LittleEndian>(*n).unwrap(),
//             DataBytes::Four(n) => buf.write_u32::<LittleEndian>(*n).unwrap(),
//             DataBytes::FourSigned(n) => buf.write_i32::<LittleEndian>(*n).unwrap(),
//         }

//         buf
//     }
// }

impl_to_databytes!(u8, DataBytes::One);
impl_to_databytes!(i8, DataBytes::OneSigned);
impl_to_databytes!(u16, DataBytes::Two);
impl_to_databytes!(i16, DataBytes::TwoSigned);
impl_to_databytes!(u32, DataBytes::Four);
impl_to_databytes!(i32, DataBytes::FourSigned);
