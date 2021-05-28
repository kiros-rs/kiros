//! # Dynamixel Protocol v1.0
//! This file contains a collection of abstract representations used to
//! communicate with Robotis 'Dynamixel' servos via their
//! [Protocol 1.0](https://emanual.robotis.com/docs/en/dxl/protocol1/)

use super::{DynamixelInformation, PacketManipulation};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Read, Write};
use num_traits::Num;

/// The types of instructions that can be sent to a Dynamixel.
#[derive(Copy, Clone, Debug)]
pub enum InstructionType {
    Ping,
    Read,
    Write,
    RegWrite,
    Action,
    Reset,
    Reboot,
    SyncWrite,
    BulkRead,
}

impl From<InstructionType> for u8 {
    fn from(instruction: InstructionType) -> u8 {
        match instruction {
            InstructionType::Ping => 1,
            InstructionType::Read => 2,
            InstructionType::Write => 3,
            InstructionType::RegWrite => 4,
            InstructionType::Action => 5,
            InstructionType::Reset => 6,
            InstructionType::Reboot => 7,
            InstructionType::SyncWrite => 131,
            InstructionType::BulkRead => 146,
        }
    }
}

impl TryFrom<u8> for InstructionType {
    type Error = &'static str;
    fn try_from(instruction: u8) -> Result<InstructionType, Self::Error> {
        match instruction {
            1 => Ok(InstructionType::Ping),
            2 => Ok(InstructionType::Read),
            3 => Ok(InstructionType::Write),
            4 => Ok(InstructionType::RegWrite),
            5 => Ok(InstructionType::Action),
            6 => Ok(InstructionType::Reset),
            7 => Ok(InstructionType::Reboot),
            131 => Ok(InstructionType::SyncWrite),
            146 => Ok(InstructionType::BulkRead),
            _ => Err("Unable to match out-of-range instruction!"),
        }
    }
}

/// The types of statuses that can be returned by a Dynamixel, as stored
/// with each bit representing a different error. For more info, see
/// <https://emanual.robotis.com/docs/en/dxl/protocol1/#status-packetreturn-packet>
#[derive(Clone, Copy, Debug)]
pub enum StatusType {
    // This needs work - maybe a Result to represent either success or failure?
    // It should not be possible to have Success and Overload at the same time.
    Success,
    Instruction,
    Overload,
    Checksum,
    Range,
    Overheating,
    AngleLimit,
    InputVoltage,
}

impl StatusType {
    /// Gets the numeric representation of an error code given the list of
    /// errors
    pub fn get_error_code(errors: &Vec<StatusType>) -> u8 {
        let mut error_code = 0u8;

        for err in errors {
            let index = match err {
                StatusType::Success => return 0,
                StatusType::Instruction => 1,
                StatusType::Overload => 2,
                StatusType::Checksum => 3,
                StatusType::Range => 4,
                StatusType::Overheating => 5,
                StatusType::AngleLimit => 6,
                StatusType::InputVoltage => 7,
            };

            if index != 0 {
                error_code |= 1 << index;
            } else {
                return 0;
            }
        }

        error_code
    }

    /// Gets the list of error types given a numeric representation of the error
    pub fn get_error_types(error: &u8) -> Vec<StatusType> {
        let mut errors: Vec<StatusType> = vec![];

        for i in 0..8 {
            if error & (1 << i) != 0 {
                let error_type: Option<StatusType> = match i {
                    1 => Some(StatusType::Instruction),
                    2 => Some(StatusType::Overload),
                    3 => Some(StatusType::Checksum),
                    4 => Some(StatusType::Range),
                    5 => Some(StatusType::Overheating),
                    6 => Some(StatusType::AngleLimit),
                    7 => Some(StatusType::InputVoltage),
                    _ => None,
                };

                if let Some(e) = error_type {
                    errors.push(e);
                }
            }
        }

        errors
    }
}

/// The different kinds of values that can be stored in the packet's
/// error/instruction column.
#[derive(Clone, Debug)]
pub enum PacketType {
    Instruction(InstructionType),
    Status(Vec<StatusType>),
}

/// An abstraction of incoming/outgoing packets
#[derive(Clone, Debug)]
pub struct Packet {
    pub id: u8,
    length: u8,
    pub packet_type: PacketType,
    pub parameters: Vec<u8>,
    checksum: u8,
}

impl PacketManipulation for Packet {
    /// Calculates the checksum for the packet
    fn checksum(id: &u8, length: &u8, parameters: &Vec<u8>, opcode: &u8) -> u8 {
        let mut sum: usize = *id as usize + *length as usize;
        sum += parameters.iter().map(|i| *i as usize).sum::<usize>();
        sum += *opcode as usize;

        let chk: u8 = if sum > 255 {
            (sum as u8) & 0xFF
        } else {
            sum as u8
        };

        !chk
    }

    /// Provides packet-crafting functionality for servo communication. If you want
    /// to actually write to the servo, see the ConnectionHandler trait (TODO: LINK).
    fn generate(&self) -> Result<Vec<u8>, String> {
        if let PacketType::Instruction(instruction) = self.packet_type {
            let mut packet = vec![255, 255, self.id, self.length, instruction.into()];
            packet.extend(&self.parameters);
            packet.push(self.checksum);

            Ok(packet)
        } else {
            Err("You cannot write a status packet to a servo!".to_string())
        }
    }
}

#[derive(Debug)]
pub enum PacketReadError {
    InvalidLength,
    InvalidHeader,
    InvalidChecksum,
    InvalidInstruction,
}

impl Packet {
    pub fn from_vec(
        vec: Vec<u8>,
        op: InstructionType,
        length: Option<usize>, // This paramater could probably be improved
    ) -> Result<Packet, PacketReadError> {
        // Run any instruction-spectific checks
        match op {
            InstructionType::Ping => {
                if vec.len() != 6 {
                    return Err(PacketReadError::InvalidLength);
                }
            }
            InstructionType::Read => {
                if vec.len()
                    != 6 + length
                        .expect("Must pass length parameter for all applicable instructions!")
                {
                    return Err(PacketReadError::InvalidLength);
                }
            }
            InstructionType::Write => {}
            InstructionType::RegWrite => {}
            InstructionType::Action => {}
            InstructionType::Reset => {}
            InstructionType::Reboot => {}
            InstructionType::SyncWrite => {}
            InstructionType::BulkRead => {}
        };

        // Validate header
        if vec[0..2] != [0xFF, 0xFF] {
            return Err(PacketReadError::InvalidHeader);
        }

        // Extract packet data
        let (id, length, error) = (vec[2], vec[3], vec[4]);
        let params: Vec<u8> = vec[5..vec.len() - 1].to_vec();
        let chk = vec.last().unwrap();

        // Validate checksum
        if *chk != Packet::checksum(&id, &length, &params, &error) {
            return Err(PacketReadError::InvalidChecksum);
        }

        Ok(Packet::new_raw(
            id,
            PacketType::Status(StatusType::get_error_types(&error)),
            params,
        ))
    }

    pub fn new_raw(id: u8, packet_type: PacketType, parameters: Vec<u8>) -> Packet {
        // This should be changed to a universal trait to improve ergonomics
        let opcode = match packet_type {
            PacketType::Instruction(inst) => u8::from(inst),
            PacketType::Status(ref status) => StatusType::get_error_code(&status),
        };
        let checksum = Packet::checksum(&id, &(parameters.len() as u8 + 2u8), &parameters, &opcode);

        let packet = Packet {
            id,
            length: parameters.len() as u8 + 2u8,
            packet_type,
            parameters,
            checksum,
        };

        packet
    }

    /// Creates a new protocol 1 packet
    ///
    /// ```
    /// use movement::dynamixel::PacketManipulation;
    /// use movement::dynamixel::protocol_one::{Packet, PacketType, InstructionType};
    ///
    /// fn main() {
    ///     let pck = Packet::new(1, PacketType::Instruction(InstructionType::Write), vec![25, 1]);
    ///     assert_eq!(pck.generate().unwrap(), [255, 255, 1, 4, 3, 25, 1, 221]);
    /// }
    /// ```
    pub fn new(id: u8, packet_type: PacketType, parameters: Vec<u64>) -> Packet {
        // Convert all given parameters into little-endian format
        // Also determines the minimum amount of bytes needed to represent data
        // Apparently some of the data is signed? need to investigate...
        // TODO: Add test to make sure the min-bytes functionality works
        let mut new_params: Vec<u8> = vec![];

        for i in parameters.iter() {
            let mut write_buf: Vec<u8> = vec![];

            write_buf.write_u64::<LittleEndian>(*i).unwrap();
            for x in 1..write_buf.len() {
                if LittleEndian::read_uint(&write_buf[0..x], x) == *i {
                    new_params.extend(&write_buf[0..x]);
                    break;
                }
            }
        }

        // This should be changed to a universal trait to improve ergonomics
        let opcode = match packet_type {
            PacketType::Instruction(inst) => u8::from(inst),
            PacketType::Status(ref status) => StatusType::get_error_code(&status),
        };
        let checksum = Packet::checksum(&id, &(parameters.len() as u8 + 2u8), &new_params, &opcode);

        let packet = Packet {
            id,
            length: new_params.len() as u8 + 2u8,
            packet_type,
            parameters: new_params,
            checksum,
        };

        packet
    }
}

/// This trait exposes all functionality possessed by Protocol One servos. For
/// more information, please refer to <https://emanual.robotis.com/docs/en/dxl/protocol1/#instruction-details>
// TODO: Refactor doctests using fully-implemented API
// TODO: Fix number sizes
pub trait ProtocolOne {
    /// Creates a packet to ping the dynamixel, returning the crafted packet
    ///
    /// This function implements section [4.1](https://emanual.robotis.com/docs/en/dxl/protocol1/#ping)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, DynamixelID, protocol_one::ProtocolOne};
    ///
    /// fn main() {
    ///     let dxl = Dynamixel {id: DynamixelID::ID(1)};
    ///     let Packet::ProtocolOne(packet) = dxl.ping();
    ///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x02, 0x01, 0xFB]);
    /// }
    ///
    /// ```
    fn ping(&mut self) -> Packet;

    /// Creates a packet to read from an address on the dynamixel, returning the crafted packet
    ///
    /// This function implements section [4.2](https://emanual.robotis.com/docs/en/dxl/protocol1/#read)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, DynamixelID, protocol_one::ProtocolOne};
    ///
    /// fn main() {
    ///     let dxl = Dynamixel {id: DynamixelID::ID(1)};
    ///     let Packet::ProtocolOne(packet) = dxl.read(43, 1);
    ///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x04, 0x02, 0x2B, 0x01, 0xCC]);
    /// }
    ///
    /// ```
    fn read(&mut self, address: u8, length: u64) -> Packet;

    /// Creates a packet to write a value to the dynamixel at a given address,
    /// returning the crafted packet
    ///
    /// This function implements section [4.3](https://emanual.robotis.com/docs/en/dxl/protocol1/#write)
    // TODO: Create doctest using working id() function
    fn write(&mut self, address: u8, value: u64);

    /// Creates a packet to register a value to write to the dynamixel at a
    /// given address, returning the crafted packet
    ///
    /// This function implements section [4.4](https://emanual.robotis.com/docs/en/dxl/protocol1/#reg-write)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, DynamixelID, protocol_one::ProtocolOne};
    ///
    /// fn main() {
    ///     let dxl = Dynamixel {id: DynamixelID::ID(1)};
    ///     let Packet::ProtocolOne(packet) = dxl.register_write(30, 500);
    ///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x05, 0x04, 0x1E, 0xF4, 0x01, 0xE2]);
    /// }
    ///
    /// ```
    fn register_write(&self, address: u8, value: u64) -> super::Packet;

    /// Creates a packet to action the registered value change, returning the
    /// crafted packet
    ///
    /// This function implements sction [4.5](https://emanual.robotis.com/docs/en/dxl/protocol1/#action)
    // TODO: Create doctest using working id() function
    fn action(&self) -> super::Packet;

    /// Creates a packet to reset the servo, returning the crafted packet
    ///
    /// This function implements section [4.6](https://emanual.robotis.com/docs/en/dxl/protocol1/#reset)
    // TODO: Create doctest using working id() function
    fn reset(&self) -> super::Packet;

    /// Creates a packet to reboot the servo, returning the crafted packet
    ///
    /// This function implements section [4.7](https://emanual.robotis.com/docs/en/dxl/protocol1/#reboot)
    fn reboot(&self) -> super::Packet;

    // fn bulk_read(&self) -> Result<Vec<Packet>, String>;
}

// is it possible to turn this pattern into a macro?
impl<C, T> ProtocolOne for super::Dynamixel<C, T>
where
    C: Read + Write,
    T: Num,
{
    fn ping(&mut self) -> Packet {
        let dxl_id = self.get_id().into();
        let packet = Packet::new(
            dxl_id,
            PacketType::Instruction(InstructionType::Ping),
            vec![],
        );

        super::servo_connection::write_packet(self.connection_handler.as_mut(), packet);
        let raw_packet =
            super::servo_connection::read_exact_packet(self.connection_handler.as_mut(), 6);

        Packet::from_vec(raw_packet, InstructionType::Ping, None).unwrap()
    }

    fn read(&mut self, address: u8, length: u64) -> Packet {
        let packet = Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Read),
            vec![address.into(), length],
        );

        super::servo_connection::write_packet(self.connection_handler.as_mut(), packet);
        let raw_packet = super::servo_connection::read_exact_packet(
            self.connection_handler.as_mut(),
            6 + length as usize,
        );
        Packet::from_vec(raw_packet, InstructionType::Read, Some(length as usize)).unwrap()
    }

    fn write(&mut self, address: u8, value: u64) {
        let packet = Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Write),
            vec![address.into(), value],
        );

        super::servo_connection::write_packet(self.connection_handler.as_mut(), packet);
    }

    fn register_write(&self, address: u8, value: u64) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::RegWrite),
            vec![address.into(), value],
        ))
    }

    fn action(&self) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Action),
            vec![],
        ))
    }

    fn reset(&self) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Reset),
            vec![],
        ))
    }

    fn reboot(&self) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Reboot),
            vec![],
        ))
    }
}

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
/// use movement::dynamixel::{Dynamixel, DynamixelID, Packet, SyncPacket};
/// use movement::dynamixel::protocol_one::{ProtocolOne, sync_write};
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
///     let Packet::ProtocolOne(packet) = sync_write(packets, 2).unwrap();
///     assert_eq!(
///         packet.generate().unwrap(),
///         vec![
///             0xFF, 0xFF, 0xFE, 0x0E, 0x83, 0x1E, 0x04, 0x00, 0x10, 0x00, 0x50, 0x01, 0x01, 0x20,
///             0x02, 0x60, 0x03, 0x67
///         ]
///     );
/// }
/// ```
pub fn sync_write(
    mut packets: Vec<super::SyncPacket>,
    bytesize: usize,
) -> Result<super::Packet, String> {
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

    Ok(super::Packet::ProtocolOne(Packet::new_raw(
        super::DynamixelID::Broadcast.into(),
        PacketType::Instruction(InstructionType::SyncWrite),
        params,
    )))
}

/// Creates a packet to read from multiple servos at once
/// This function will return an error if multiple items in the `packets`
/// vector contain the same ID.
///
/// This function implements section [4.9](https://emanual.robotis.com/docs/en/dxl/protocol1/#bulk-read)
/// ```
/// use movement::dynamixel::{Dynamixel, DynamixelID, Packet, BulkReadPacket};
/// use movement::dynamixel::protocol_one::{ProtocolOne, bulk_read};
/// fn main() {
///     let dxl = Dynamixel {
///         id: DynamixelID::Broadcast,
///     };
///
///     let packets: Vec<BulkReadPacket> = vec![BulkReadPacket{id: 1, length: 2, address: 30}, BulkReadPacket{id: 2, length: 2, address: 36}];
///     let Packet::ProtocolOne(packet) = bulk_read(packets).unwrap();
///     assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0xFE, 0x09, 0x92, 0x00, 0x02, 0x01, 0x1E, 0x02, 0x02, 0x24, 0x1D]);
/// }
pub fn bulk_read(packets: Vec<super::BulkReadPacket>) -> Result<super::Packet, String> {
    let mut known_ids: Vec<u8> = vec![];
    for i in packets.iter() {
        if known_ids.contains(&i.id) {
            return Err(String::from("Cannot address the same ID more than once!"));
        }

        known_ids.push(i.id);
    }

    let mut params: Vec<u64> = vec![0x00];
    for i in packets.iter() {
        params.push(i.length.into());
        params.push(i.id.into());
        params.push(i.address.into());
    }

    Ok(super::Packet::ProtocolOne(Packet::new(
        super::DynamixelID::Broadcast.into(),
        PacketType::Instruction(InstructionType::BulkRead),
        params,
    )))
}
