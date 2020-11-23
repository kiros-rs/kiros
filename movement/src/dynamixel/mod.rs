pub mod protocol_one;

use byteorder::{ByteOrder, LittleEndian};
use std::collections::HashMap;

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
///
/// Note that if you wish to broadcast to all servos, you will need to create
/// an empty Dynamixel
pub struct Dynamixel {
    // pub connection_handler: Box<dyn ConnectionHandler>,
    // pub control_table: HashMap<String, ControlTableType>,
    // pub sensors: HashMap<String, Box<dyn DataSensor<isize>>>,
    // pub components: HashMap<String, ()>, // should become a custom datatype/enum
    // pub information: HashMap<String, ControlTableData>,
    // pub constraints: HashMap<String, ControlTableData>,
    // pub last_packet: Packet,
    // pub sent_packets: Vec<Packet>,
    // pub collects_packets: bool,
    pub id: DynamixelID, // This is a TEMPORARY fix
}

/// A Dynamixel can be either a wheel (CW & CCW limits set to 0) or a joint
/// (CW or CCW limits nonzero). This enum contains a representation of these 2 states.
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

pub trait PacketCommunications {
    fn write(data: Vec<u8>) -> Packet;
    fn read() -> Packet;
}

// Remove 'get' prefix?
pub trait DynamixelInformation {
    fn get_id(&self) -> DynamixelID;
    // fn get_baudrate(&self) -> u64;
}

impl DynamixelInformation for Dynamixel {
    fn get_id(&self) -> DynamixelID {
        // Temporary until this can be properly implemented later
        self.id
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyncPacket {
    pub id: u8, // Cannot be DynamixelID enum as only non-broadcast IDs allowed
    pub data: u64,
    pub address: u8,
}

/// This trait exposes all functionality possessed by Protocol One servos. For
/// more information, please refer to <https://emanual.robotis.com/docs/en/dxl/protocol1/#instruction-details>
// TODO: Refactor doctests using fully-implemented API
pub trait ProtocolOne {
    /// Creates a packet to ping the dynamixel, returning the crafted packet
    ///
    /// This function implements section [4.1](https://emanual.robotis.com/docs/en/dxl/protocol1/#ping)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, ProtocolOne, DynamixelID};
    ///
    /// fn main() {
    ///     let dxl = Dynamixel {id: DynamixelID::ID(1)};
    ///     let Packet::ProtocolOne(packet) = dxl.ping();
    ///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x02, 0x01, 0xFB]);
    /// }
    ///
    /// ```
    fn ping(&self) -> Packet;

    /// Creates a packet to read from an address on the dynamixel, returning the crafted packet
    ///
    /// This function implements section [4.2](https://emanual.robotis.com/docs/en/dxl/protocol1/#read)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, ProtocolOne, DynamixelID};
    ///
    /// fn main() {
    ///     let dxl = Dynamixel {id: DynamixelID::ID(1)};
    ///     let Packet::ProtocolOne(packet) = dxl.read(43, 1);
    ///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x04, 0x02, 0x2B, 0x01, 0xCC]);
    /// }
    ///
    /// ```
    fn read(&self, address: u64, length: u64) -> Packet;

    /// Creates a packet to write a value to the dynamixel at a given address,
    /// returning the crafted packet
    ///
    /// This function implements section [4.3](https://emanual.robotis.com/docs/en/dxl/protocol1/#write)
    // TODO: Create doctest using working id() function
    fn write(&self, address: u64, value: u64) -> Packet;

    /// Creates a packet to register a value to write to the dynamixel at a
    /// given address, returning the crafted packet
    ///
    /// This function implements section [4.4](https://emanual.robotis.com/docs/en/dxl/protocol1/#reg-write)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, ProtocolOne, DynamixelID};
    ///
    /// fn main() {
    ///     let dxl = Dynamixel {id: DynamixelID::ID(1)};
    ///     let Packet::ProtocolOne(packet) = dxl.register_write(30, 500);
    ///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x05, 0x04, 0x1E, 0xF4, 0x01, 0xE2]);
    /// }
    ///
    /// ```
    fn register_write(&self, address: u64, value: u64) -> Packet;

    /// Creates a packet to action the registered value change, returning the
    /// crafted packet
    ///
    /// This function implements sction [4.5](https://emanual.robotis.com/docs/en/dxl/protocol1/#action)
    // TODO: Create doctest using working id() function
    fn action(&self) -> Packet;

    /// Creates a packet to reset the servo, returning the crafted packet
    ///
    /// This function implements section [4.6](https://emanual.robotis.com/docs/en/dxl/protocol1/#reset)
    // TODO: Create doctest using working id() function
    fn reset(&self) -> Packet;

    /// Creates a packet to reboot the servo, returning the crafted packet
    ///
    /// This function implements section [4.7](https://emanual.robotis.com/docs/en/dxl/protocol1/#reboot)
    fn reboot(&self) -> Packet;

    /// Creates a packet to synchronously write to multiple servos at once,
    /// returning a result wrapping the crafted packet or an error message.
    /// The result will be an `Err` value in the following situations:
    /// - The `packets` parameter is empty
    /// - Servos do not write to the same addresses
    /// - The addresses are non-consecutive
    /// - The amount of addresses differs between packets
    /// - Any servo in the chain does not support synchronous write
    ///
    /// This function implements section [4.8](https://emanual.robotis.com/docs/en/dxl/protocol1/#sync-write)
    /// ```
    /// use movement::dynamixel::{Dynamixel, DynamixelID, Packet, ProtocolOne, SyncPacket};
    /// fn main() {
    ///     let dxl = Dynamixel {
    ///         id: DynamixelID::Broadcast,
    ///     };
    ///     let packets: Vec<SyncPacket> = vec![
    ///         SyncPacket {
    ///             id: 0,
    ///             data: 0x010,
    ///             address: 0x1E,
    ///         },
    ///         SyncPacket {
    ///             id: 0,
    ///             data: 0x150,
    ///             address: 0x20,
    ///         },
    ///         SyncPacket {
    ///             id: 1,
    ///             data: 0x220,
    ///             address: 0x1E,
    ///         },
    ///         SyncPacket {
    ///             id: 1,
    ///             data: 0x360,
    ///             address: 0x20,
    ///         },
    ///     ];
    ///
    ///     let Packet::ProtocolOne(packet) = dxl.sync_write(packets, 2).unwrap();
    ///     assert_eq!(
    ///         packet.generate().unwrap(),
    ///         vec![
    ///             0xFF, 0xFF, 0xFE, 0x0E, 0x83, 0x1E, 0x04, 0x00, 0x10, 0x00, 0x50, 0x01, 0x01, 0x20,
    ///             0x02, 0x60, 0x03, 0x67
    ///         ]
    ///     );
    /// }
    /// ```
    // TODO: These functions below don't really belong to a single servo, consider
    // options for removing dependence on a single dynamixel
    fn sync_write(&self, packets: Vec<SyncPacket>, bytesize: usize) -> Result<Packet, String>;
    // fn bulk_read(&self) -> Result<Vec<Packet>, String>;
}

// impl for any implementor of DynamixelInformation?
impl ProtocolOne for Dynamixel {
    fn ping(&self) -> Packet {
        let dxl_id = u8::from(self.get_id());

        Packet::ProtocolOne(protocol_one::Packet::new(
            dxl_id,
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::Ping),
            vec![],
        ))
    }

    fn read(&self, address: u64, length: u64) -> Packet {
        Packet::ProtocolOne(protocol_one::Packet::new(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::Read),
            vec![address, length],
        ))
    }

    fn write(&self, address: u64, value: u64) -> Packet {
        Packet::ProtocolOne(protocol_one::Packet::new(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::Write),
            vec![address, value],
        ))
    }

    fn register_write(&self, address: u64, value: u64) -> Packet {
        Packet::ProtocolOne(protocol_one::Packet::new(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::RegWrite),
            vec![address, value],
        ))
    }

    fn action(&self) -> Packet {
        Packet::ProtocolOne(protocol_one::Packet::new(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::Action),
            vec![],
        ))
    }

    fn reset(&self) -> Packet {
        Packet::ProtocolOne(protocol_one::Packet::new(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::Reset),
            vec![],
        ))
    }

    fn reboot(&self) -> Packet {
        Packet::ProtocolOne(protocol_one::Packet::new(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::Reboot),
            vec![],
        ))
    }
    fn sync_write(&self, mut packets: Vec<SyncPacket>, bytesize: usize) -> Result<Packet, String> {
        // The sync_write method has more conditions that must be satisfied
        // The servo must support sync_write
        // There must be at least 1 Dynamixel
        if packets.len() == 0 {
            return Err(String::from("Must have at least 1 Dynamixel!"));
        }

        // There must be the same amount of data for every servo
        let mut instruction_count: HashMap<u8, u8> = HashMap::new();
        for pck in packets.iter() {
            let entry = instruction_count.entry(pck.id).or_insert(0);
            *entry += 1;
        }

        let first_length = instruction_count.values().next().unwrap();
        for length in instruction_count.values() {
            if length != first_length {
                return Err(String::from(format!(
                    "Must have consistent data length! (Found {} and {})",
                    first_length, length
                )));
            }
        }
        // Each servo must receive the same set of addresses
        // The addresses must be consecutive

        // Sort the packets by ID then by address
        packets.sort_by(|a, b| a.id.cmp(&b.id).cmp(&b.address.cmp(&a.address)));

        let mut params: Vec<u8> = vec![];
        params.push(packets[0].address);
        params.push(*first_length * bytesize as u8);

        for i in 0..packets.len() {
            // Check if this packet is the first one for the servo
            if i % *first_length as usize == 0 {
                params.push(packets[i].id);
            }

            let mut buf: Vec<u8> = vec![0; bytesize];
            LittleEndian::write_uint(&mut buf, packets[i].data, bytesize);
            params.extend(&buf);
        }

        Ok(Packet::ProtocolOne(protocol_one::Packet::new_raw(
            u8::from(self.get_id()),
            protocol_one::PacketType::Instruction(protocol_one::InstructionType::SyncWrite),
            params,
        )))
    }
    // fn bulk_read(&self) -> Result<Vec<Packet>, String>;
}
