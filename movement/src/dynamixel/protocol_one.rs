//! # Dynamixel Protocol v1.0
//! This file contains a collection of abstract representations used to
//! communicate with Robotis 'Dynamixel' servos via their
//! [Protocol 1.0](https://emanual.robotis.com/docs/en/dxl/protocol1/)

use super::{PacketManipulation, Parameter};
use std::collections::HashMap;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolOneError {
    #[error("Dynamixel returned error code {0:#04x?} {:?}", StatusType::get_error_types(*.0))]
    DynamixelError(u8),
    #[error("Value {0} is invalid!")]
    InvalidValue(u8),
    #[error("Length of {0} is invalid!")]
    InvalidLength(usize),
    #[error("The header {0:?} is invalid!")]
    InvalidHeader(Vec<u8>),
    #[error("The checksum of {0} is invalid!")]
    InvalidChecksum(u8),
    #[error("The instruction {0} is invalid!")]
    InvalidInstruction(u8),
    #[error("Error while processing sync write: {0}")]
    SyncWrite(SyncWriteError),
    #[error("Error while processing bulk read: {0}")]
    BulkRead(BulkReadError)
}

#[derive(Debug, Error)]
pub enum SyncWriteError {
    #[error("Must have at least 1 Dynamixel!")]
    NoDynamixels,
    #[error("Inconsistent byte size!")]
    InconsistentSize,
    #[error("Inconsistent data length!")]
    InconsistentLength,
}

#[derive(Debug, Error)]
pub enum BulkReadError {
    #[error("Cannot address the same ID more than once!")]
    SameID(u8),
}

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
    type Error = ProtocolOneError;
    fn try_from(instruction: u8) -> Result<Self, ProtocolOneError> {
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
            val => Err(ProtocolOneError::InvalidValue(val)),
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
    fn generate(&self) -> Vec<u8> {
        let opcode = match &self.packet_type {
            PacketType::Instruction(inst) => *inst as u8,
            PacketType::Status(errs) => StatusType::get_error_code(&errs),
        };

        let mut packet = vec![255, 255, self.id, self.length, opcode];
        packet.extend(&self.bytes);
        packet.push(self.checksum);

        packet
    }
}

impl Packet {
    pub fn from_buf(&self, buf: &[u8]) -> Result<Self, ProtocolOneError> {
        // Run any instruction-spectific checks
        let params: Vec<Parameter> = match self.packet_type {
            PacketType::Instruction(op) => match op {
                InstructionType::Ping => {
                    if buf.len() != 6 {
                        return Err(ProtocolOneError::InvalidLength(buf.len()));
                    }

                    vec![]
                }
                InstructionType::Read => {
                    // The second parameter is guaranteed to be an unsigned u8
                    let data_len = self.parameters[1].as_bytes()[0] as usize;
                    if buf.len() != 6 + data_len {
                        return Err(ProtocolOneError::InvalidLength(buf.len()));
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
            return Err(ProtocolOneError::InvalidHeader(buf[0..2].to_vec()));
        }

        // Extract packet data
        let (id, len, error) = (buf[2], buf[3], buf[4]);
        let chk = buf.last().unwrap();

        // Validate checksum
        if *chk != Self::checksum(id, len, &Parameter::from_slice(&params), error) {
            return Err(ProtocolOneError::InvalidChecksum(*chk));
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

pub fn ping(id: u8) -> Packet {
    Packet::new(id, PacketType::Instruction(InstructionType::Ping), &[])
}

pub fn read(id: u8, address: u8, length: u8) -> Packet {
    Packet::new(
        id,
        PacketType::Instruction(InstructionType::Read),
        &[
            Parameter::unsigned(address.into(), 1),
            Parameter::unsigned(length.into(), 1),
        ],
    )
}

pub fn write(id: u8, address: u8, value: Parameter) -> Packet {
    Packet::new(
        id,
        PacketType::Instruction(InstructionType::Write),
        &[Parameter::unsigned(address.into(), 1), value],
    )
}

pub fn register_write(id: u8, address: u8, value: Parameter) -> Packet {
    Packet::new(
        id,
        PacketType::Instruction(InstructionType::RegWrite),
        &[Parameter::unsigned(address.into(), 1), value],
    )
}

pub fn action(id: u8) -> Packet {
    Packet::new(
        id,
        PacketType::Instruction(InstructionType::Action),
        &[],
    )
}

pub fn reset(id: u8) -> Packet {
    Packet::new(
        id,
        PacketType::Instruction(InstructionType::Reset),
        &[],
    )
}

pub fn reboot(id: u8) -> Packet {
    Packet::new(
        id,
        PacketType::Instruction(InstructionType::Reboot),
        &[],
    )
}

/// A packet used to address the same instruction to a group of servos
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyncPacket {
    pub id: u8, // Cannot be DynamixelID enum as only non-broadcast IDs allowed
    pub data: Parameter,
    pub address: u8,
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
pub fn sync_write(mut packets: Vec<SyncPacket>) -> Result<Packet, ProtocolOneError> {
    // The sync_write method has more conditions that must be satisfied
    // The servo must support sync_write
    // There must be at least 1 Dynamixel
    if packets.is_empty() {
        return Err(ProtocolOneError::SyncWrite(SyncWriteError::NoDynamixels));
    }

    let bytesize = packets[0].data.len;
    if !packets.iter().all(|i| i.data.len != bytesize) {
        return Err(ProtocolOneError::SyncWrite(SyncWriteError::InconsistentSize));
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
        return Err(ProtocolOneError::SyncWrite(SyncWriteError::InconsistentLength));
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

    Ok(Packet::new(
        super::DynamixelID::Broadcast.into(),
        PacketType::Instruction(InstructionType::SyncWrite),
        &params,
    ))
}

/// A packet used to read from multiple servos at the same time (MX series only)
pub struct BulkReadPacket {
    pub id: u8,
    pub length: u8,
    pub address: u8,
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
pub fn bulk_read(packets: &[BulkReadPacket]) -> Result<Packet, ProtocolOneError> {
    let mut known_ids: Vec<u8> = vec![];
    for i in packets {
        if known_ids.contains(&i.id) {
            return Err(ProtocolOneError::BulkRead(BulkReadError::SameID(i.id)));
        }

        known_ids.push(i.id);
    }

    let mut params: Vec<Parameter> = vec![Parameter::unsigned(0, 1)];
    for i in packets {
        params.push(Parameter::unsigned(i.length.into(), 1));
        params.push(Parameter::unsigned(i.id.into(), 1));
        params.push(Parameter::unsigned(i.address.into(), 1));
    }

    Ok(Packet::new(
        super::DynamixelID::Broadcast.into(),
        PacketType::Instruction(InstructionType::BulkRead),
        &params,
    ))
}
