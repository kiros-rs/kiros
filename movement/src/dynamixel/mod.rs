pub mod protocol_one;
pub mod servo_connection;

use connection::Connect;
use num_traits::Num;
use phf;
use ron::de::from_str;
use sensor::DataSensor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

// Extend this with protocol 2 packet when implemented
/// A protocol-agnostic representation of a Dynamixel packet
pub enum Packet {
    ProtocolOne(protocol_one::Packet),
}

pub enum Protocol {
    ProtocolOne,
    ProtocolTwo,
}

/// The abstract categories an item in the control table
/// can be part of.
#[derive(Clone, Copy, Debug)]
pub enum ControlTableType {
    Sensor,
    ServoInformation,
    Component,
    Value,
    Uncategorized,
}

/// The levels of permission a user is granted in terms of an item in the
/// control table.
#[derive(Serialize, Deserialize, Debug)]
pub enum AccessLevel {
    Read,
    Write,
    ReadWrite,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ModbusByte {
    Low,
    High,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModbusAddress {
    pub address: usize,
    pub byte: Option<ModbusByte>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DynamixelAddress<T> {
    Standard(T),
    Modbus(T),
}

/// A representation of an item in the control table, where only information
/// is stored. When applicable, items in the control table are represented in
/// this format, along with any optional data such as range or description.
#[derive(Serialize, Deserialize, Debug)]
pub struct ControlTableData<T> {
    pub address: DynamixelAddress<T>,
    pub size: T,
    pub data_name: Option<String>,
    pub description: Option<String>,
    pub access: AccessLevel,
    pub initial_value: Option<String>,
    pub range: Option<(T, T)>, // There might be an actual range struct
    pub units: Option<sensor::DataUnit>,
    pub modbus: Option<ModbusAddress>,
}

/// An abstract representation of a Dynamixel servo
/// The servo contains the following basic types of data in its control table:
/// - Sensor (temperature, voltage)
/// - Servo Information (model, id)
/// - Component (led, alarm)
/// - Value (cw limit, max speed)
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
pub struct Dynamixel<C: Connect, T: Num> {
    pub connection_handler: Box<C>,
    pub control_table: HashMap<String, ControlTableType>,
    pub sensors: HashMap<String, Box<dyn DataSensor<isize>>>,
    pub components: HashMap<String, ()>, // should become a custom datatype/enum
    pub information: HashMap<String, ControlTableData<T>>,
    pub parameters: HashMap<String, ControlTableData<T>>,
    pub protocol: Protocol,
}

#[derive(Debug, Error)]
pub enum DynamixelError {
    #[error("Unable to find template for Dynamixel: {0}")]
    InvalidTemplate(String),
    #[error("No data name for row")]
    NoDataName,
}

// There should be a builder pattern for this struct
impl<C, T> Dynamixel<C, T>
where
    C: Connect,
    T: Num,
{
    /// Create a new Dynamixel servo from template
    pub fn from_template(name: &str, connection_handler: C, protocol: Protocol) -> Result<Self, DynamixelError> {
        let data: &str = match DYNAMIXELS.get(name) {
            Some(val) => val,
            None => return Err(DynamixelError::InvalidTemplate(name.to_string())),
        };
        // This should probably be changed to use num-traits in the future
        let rows: Vec<ControlTableData<u64>> = from_str(data).unwrap();

        // There is duplication of the "Data Name" value which should be removed
        // One solution is to store by address instead of data name?
        let mut control_table: HashMap<String, ControlTableType> = HashMap::new();
        for row in rows {
            // In the future, there needs to be handling for None
            let name = match row.data_name {
                Some(val) => val,
                None => return Err(DynamixelError::NoDataName)
            };
            let value = match CONTROL_TABLE_TYPES.get(&*name) {
                Some(value) => *value,
                None => ControlTableType::Uncategorized,
            };

            control_table.insert(name, value);
        }

        Ok(Self {
            connection_handler: Box::new(connection_handler),
            control_table,
            sensors: HashMap::new(),
            components: HashMap::new(),
            information: HashMap::new(),
            parameters: HashMap::new(),
            protocol,
        })
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

impl From<DynamixelID> for u8 {
    fn from(item: DynamixelID) -> Self {
        match item {
            DynamixelID::Broadcast => 0xFE,
            DynamixelID::ID(id) => id,
        }
    }
}
// TODO: Rename this to something better
pub trait PacketManipulation {
    fn checksum(id: u8, length: u8, parameters: &[u8], opcode: u8) -> u8;
    fn generate(&self) -> Vec<u8>;
}

// Remove 'get' prefix?
pub trait DynamixelInformation {
    fn get_id(&self) -> DynamixelID;
    // fn get_baudrate(&self) -> u64;
}

// return values should be wrappen in Option
impl<C, T> DynamixelInformation for Dynamixel<C, T>
where
    C: Connect,
    T: Num,
{
    fn get_id(&self) -> DynamixelID {
        // Temporary until this can be properly implemented later
        DynamixelID::ID(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ParameterType {
    Signed(i64),
    Unsigned(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Parameter {
    pub param_type: ParameterType,
    pub len: usize,
}

impl Parameter {
    // This should return a result if bytes of value is larger than len
    pub fn signed(value: i64, len: usize) -> Self {
        Self {
            param_type: ParameterType::Signed(value),
            len,
        }
    }

    // This should return a result if bytes of value is larger than len
    pub fn unsigned(value: u64, len: usize) -> Self {
        Self {
            param_type: ParameterType::Unsigned(value),
            len,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let bytes = match self.param_type {
            ParameterType::Signed(val) => val.to_le_bytes(),
            ParameterType::Unsigned(val) => val.to_le_bytes(),
        };

        // This should panic if any data is being discarded
        bytes[..self.len].into()
    }

    pub fn from_slice(value: &[Parameter]) -> Vec<u8> {
        value.iter().map(|i| i.as_bytes()).flatten().collect()
    }
}
