//! # Dynamixel Protocol v1.0
//! This file contains a collection of abstract representations used to
//! communicate with Robotis 'Dynamixel' servos via their
//! [Protocol 1.0](https://emanual.robotis.com/docs/en/dxl/protocol1/)

use super::{DynamixelInformation, PacketManipulation, Parameter};
use num_traits::Num;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Read, Write};

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
    fn from(instruction: InstructionType) -> Self {
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
    fn try_from(instruction: u8) -> Result<Self, Self::Error> {
        match instruction {
            1 => Ok(Self::Ping),
            2 => Ok(Self::Read),
            3 => Ok(Self::Write),
            4 => Ok(Self::RegWrite),
            5 => Ok(Self::Action),
            6 => Ok(Self::Reset),
            7 => Ok(Self::Reboot),
            131 => Ok(Self::SyncWrite),
            146 => Ok(Self::BulkRead),
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
    pub fn get_error_code(errors: &[Self]) -> u8 {
        let mut error_code = 0u8;

        for err in errors {
            let index = match err {
                Self::Success => return 0,
                Self::Instruction => 1,
                Self::Overload => 2,
                Self::Checksum => 3,
                Self::Range => 4,
                Self::Overheating => 5,
                Self::AngleLimit => 6,
                Self::InputVoltage => 7,
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
    pub fn get_error_types(error: u8) -> Vec<Self> {
        let mut errors: Vec<Self> = vec![];

        for i in 0..8 {
            if error & (1 << i) != 0 {
                let error_type: Option<Self> = match i {
                    1 => Some(Self::Instruction),
                    2 => Some(Self::Overload),
                    3 => Some(Self::Checksum),
                    4 => Some(Self::Range),
                    5 => Some(Self::Overheating),
                    6 => Some(Self::AngleLimit),
                    7 => Some(Self::InputVoltage),
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
    pub bytes: Vec<u8>,
    pub parameters: Vec<Parameter>,
    checksum: u8,
}

impl PacketManipulation for Packet {
    /// Calculates the checksum for the packet
    fn checksum(id: u8, length: u8, parameters: &[u8], opcode: u8) -> u8 {
        let mut sum: usize = id as usize + length as usize;
        sum += parameters.iter().map(|i| *i as usize).sum::<usize>();
        sum += opcode as usize;

        !sum.to_le_bytes()[0]
    }

    /// Provides packet-crafting functionality for servo communication. If you want
    /// to actually write to the servo, see the `ConnectionHandler` trait.
    fn generate(&self) -> Result<Vec<u8>, String> {
        if let PacketType::Instruction(instruction) = self.packet_type {
            let mut packet = vec![255, 255, self.id, self.length, instruction.into()];
            packet.extend(&self.bytes);
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
    pub fn from_buf(&self, buf: &[u8]) -> Result<Self, PacketReadError> {
        // Run any instruction-spectific checks
        let params: Vec<Parameter> = match self.packet_type {
            PacketType::Instruction(op) => match op {
                InstructionType::Ping => {
                    if buf.len() != 6 {
                        return Err(PacketReadError::InvalidLength);
                    }

                    vec![]
                }
                InstructionType::Read => {
                    // The second parameter is guaranteed to be an unsigned u8
                    let data_len = self.parameters[1].as_bytes()[0] as usize;
                    if buf.len() != 6 + data_len {
                        return Err(PacketReadError::InvalidLength);
                    }

                    // Need to use the stored range to figure out if this is a signed or unsigned value
                    let mut bytes = [0u8; 8];
                    bytes[..data_len].clone_from_slice(&buf[5..(data_len + 5)]);
                    vec![Parameter::unsigned(u64::from_le_bytes(bytes), data_len)]
                }
                _ => buf[5..buf.len() - 1]
                    .iter()
                    .map(|i| Parameter::unsigned(*i as u64, 1))
                    .collect(),
            },
            PacketType::Status(_) => todo!(),
        };

        // Validate header
        if buf[0..2] != [0xFF, 0xFF] {
            return Err(PacketReadError::InvalidHeader);
        }

        // Extract packet data
        let (id, len, error) = (buf[2], buf[3], buf[4]);
        let chk = buf.last().unwrap();

        // Validate checksum
        if *chk != Self::checksum(id, len, &Parameter::from_slice(&params), error) {
            return Err(PacketReadError::InvalidChecksum);
        }

        Ok(Self::new(
            id,
            PacketType::Status(StatusType::get_error_types(error)),
            &params, // TODO: Fix this
        ))
    }

    /// Creates a new protocol 1 packet
    ///
    /// ```
    /// use movement::dynamixel::PacketManipulation;
    /// use movement::dynamixel::protocol_one::{Packet, PacketType, InstructionType};
    ///
    /// let pck = Packet::new(1, PacketType::Instruction(InstructionType::Write), vec![25, 1]);
    /// assert_eq!(pck.generate().unwrap(), [255, 255, 1, 4, 3, 25, 1, 221]);
    /// ```
    pub fn new(id: u8, packet_type: PacketType, parameters: &[Parameter]) -> Self {
        let param_bytes: Vec<u8> = Parameter::from_slice(parameters);

        // This should be changed to a universal trait to improve ergonomics
        let opcode = match packet_type {
            PacketType::Instruction(inst) => u8::from(inst),
            PacketType::Status(ref status) => StatusType::get_error_code(status),
        };
        let checksum = Self::checksum(id, parameters.len() as u8 + 2u8, &param_bytes, opcode);

        Self {
            id,
            length: param_bytes.len() as u8 + 2u8,
            packet_type,
            bytes: param_bytes,
            parameters: parameters.to_vec(),
            checksum,
        }
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
    /// let dxl = Dynamixel {id: DynamixelID::ID(1)};
    /// let Packet::ProtocolOne(packet) = dxl.ping();
    /// assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x02, 0x01, 0xFB]);
    /// ```
    fn ping(&mut self) -> Packet;

    /// Creates a packet to read from an address on the dynamixel, returning the crafted packet
    ///
    /// This function implements section [4.2](https://emanual.robotis.com/docs/en/dxl/protocol1/#read)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, DynamixelID, protocol_one::ProtocolOne};
    ///
    /// let dxl = Dynamixel {id: DynamixelID::ID(1)};
    /// let Packet::ProtocolOne(packet) = dxl.read(43, 1);
    /// assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x04, 0x02, 0x2B, 0x01, 0xCC]);
    /// ```
    // There should be a way to just get the returned value as a Parameter
    fn read(&mut self, address: u8, length: u8) -> Packet;

    /// Creates a packet to write a value to the dynamixel at a given address,
    /// returning the crafted packet
    ///
    /// This function implements section [4.3](https://emanual.robotis.com/docs/en/dxl/protocol1/#write)
    // TODO: Create doctest using working id() function
    fn write(&mut self, address: u8, value: Parameter);

    /// Creates a packet to register a value to write to the dynamixel at a
    /// given address, returning the crafted packet
    ///
    /// This function implements section [4.4](https://emanual.robotis.com/docs/en/dxl/protocol1/#reg-write)
    /// ```
    /// use movement::dynamixel::{Dynamixel, Packet, DynamixelID, protocol_one::ProtocolOne};
    ///
    /// let dxl = Dynamixel {id: DynamixelID::ID(1)};
    /// let Packet::ProtocolOne(packet) = dxl.register_write(30, 500);
    /// assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0x01, 0x05, 0x04, 0x1E, 0xF4, 0x01, 0xE2]);
    /// ```
    fn register_write(&self, address: u8, value: Parameter) -> super::Packet;

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
        let packet = Packet::new(dxl_id, PacketType::Instruction(InstructionType::Ping), &[]);

        super::servo_connection::write_packet(self.connection_handler.as_mut(), &packet);
        let raw_packet =
            super::servo_connection::read_exact_packet(self.connection_handler.as_mut(), 6);

        Packet::from_buf(&packet, &raw_packet).unwrap()
    }

    fn read(&mut self, address: u8, length: u8) -> Packet {
        let packet = Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Read),
            &[
                Parameter::unsigned(address.into(), 1),
                Parameter::unsigned(length.into(), 1),
            ],
        );

        super::servo_connection::write_packet(self.connection_handler.as_mut(), &packet);
        let raw_packet = super::servo_connection::read_exact_packet(
            self.connection_handler.as_mut(),
            6 + length as usize,
        );

        Packet::from_buf(&packet, &raw_packet).unwrap()
    }

    fn write(&mut self, address: u8, value: Parameter) {
        let packet = Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Write),
            &[Parameter::unsigned(address.into(), 1), value],
        );

        super::servo_connection::write_packet(self.connection_handler.as_mut(), &packet);
    }

    fn register_write(&self, address: u8, value: Parameter) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::RegWrite),
            &[Parameter::unsigned(address.into(), 1), value],
        ))
    }

    fn action(&self) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Action),
            &[],
        ))
    }

    fn reset(&self) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Reset),
            &[],
        ))
    }

    fn reboot(&self) -> super::Packet {
        super::Packet::ProtocolOne(Packet::new(
            self.get_id().into(),
            PacketType::Instruction(InstructionType::Reboot),
            &[],
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
///
/// let dxl = Dynamixel {
///     id: DynamixelID::Broadcast,
/// };
/// let packets: Vec<SyncPacket> = vec![
///     SyncPacket {
///         id: 0,
///         data: 0x010,
///         address: 0x1E,
///     },
///     SyncPacket {
///         id: 0,
///         data: 0x150,
///         address: 0x20,
///     },
///     SyncPacket {
///         id: 1,
///         data: 0x220,
///         address: 0x1E,
///     },
///     SyncPacket {
///         id: 1,
///         data: 0x360,
///         address: 0x20,
///     },
/// ];
/// let Packet::ProtocolOne(packet) = sync_write(packets, 2).unwrap();
/// assert_eq!(
///     packet.generate().unwrap(),
///     vec![
///         0xFF, 0xFF, 0xFE, 0x0E, 0x83, 0x1E, 0x04, 0x00, 0x10, 0x00, 0x50, 0x01, 0x01, 0x20,
///         0x02, 0x60, 0x03, 0x67
///     ]
/// );
/// ```
pub fn sync_write(mut packets: Vec<super::SyncPacket>) -> Result<super::Packet, String> {
    // The sync_write method has more conditions that must be satisfied
    // The servo must support sync_write
    // There must be at least 1 Dynamixel
    if packets.is_empty() {
        return Err(String::from("Must have at least 1 Dynamixel!"));
    }

    let bytesize = packets[0].data.len;
    if !packets.iter().all(|i| i.data.len != bytesize) {
        return Err("Inconsistent byte size!".into());
    }

    // There must be the same amount of data for every servo
    let mut instruction_count: HashMap<u8, usize> = HashMap::new();
    for pck in &packets {
        let entry = instruction_count.entry(pck.id).or_insert(0);
        *entry += 1;
    }

    // Since all lengths must be equal, we can use length in all future calculations
    let length = instruction_count.values().next().unwrap();
    if !instruction_count.values().all(|len| len == length) {
        return Err("Inconsistent data length!".into());
    }
    // Each servo must receive the same set of addresses
    // The addresses must be consecutive

    // Sort the packets by ID then by address
    packets.sort_by(|a, b| a.id.cmp(&b.id).cmp(&b.address.cmp(&a.address)));

    let mut params: Vec<Parameter> = vec![
        Parameter::unsigned(packets[0].address.into(), 1),
        Parameter::unsigned((*length * bytesize) as u64, 1),
    ];

    for (i, pck) in packets.iter().enumerate() {
        // Check if this packet is the first one for the servo
        if i % *length as usize == 0 {
            params.push(Parameter::unsigned(pck.id.into(), 1));
        }

        params.push(pck.data);
    }

    Ok(super::Packet::ProtocolOne(Packet::new(
        super::DynamixelID::Broadcast.into(),
        PacketType::Instruction(InstructionType::SyncWrite),
        &params,
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
///
/// let dxl = Dynamixel {
///     id: DynamixelID::Broadcast,
/// };
///
/// let packets: Vec<BulkReadPacket> = vec![BulkReadPacket{id: 1, length: 2, address: 30}, BulkReadPacket{id: 2, length: 2, address: 36}];
/// let Packet::ProtocolOne(packet) = bulk_read(packets).unwrap();
/// assert_eq!(packet.generate().unwrap(), vec![0xFF, 0xFF, 0xFE, 0x09, 0x92, 0x00, 0x02, 0x01, 0x1E, 0x02, 0x02, 0x24, 0x1D]);
/// ```
pub fn bulk_read(packets: &[super::BulkReadPacket]) -> Result<super::Packet, String> {
    let mut known_ids: Vec<u8> = vec![];
    for i in packets {
        if known_ids.contains(&i.id) {
            return Err(String::from("Cannot address the same ID more than once!"));
        }

        known_ids.push(i.id);
    }

    let mut params: Vec<Parameter> = vec![Parameter::unsigned(0, 1)];
    for i in packets {
        params.push(Parameter::unsigned(i.length.into(), 1));
        params.push(Parameter::unsigned(i.id.into(), 1));
        params.push(Parameter::unsigned(i.address.into(), 1));
    }

    Ok(super::Packet::ProtocolOne(Packet::new(
        super::DynamixelID::Broadcast.into(),
        PacketType::Instruction(InstructionType::BulkRead),
        &params,
    )))
}
