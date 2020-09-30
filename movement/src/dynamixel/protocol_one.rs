//! # Protocol 1
//! This file contains a collection of abstract representations used to
//! communicate with Robotis 'Dynamixel' servos via their
//! [Protocol 1.0](https://emanual.robotis.com/docs/en/dxl/protocol1/)

use std::convert::TryFrom;

/// Represents the types of instructions that can be sent to a Dynamixel.
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
            InstructionType::SyncWrite => 8,
            InstructionType::BulkRead => 9,
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
            8 => Ok(InstructionType::SyncWrite),
            9 => Ok(InstructionType::BulkRead),
            _ => Err("Unable to match out-of-range instruction!"),
        }
    }
}

/// Represents the types of statuses that can be returned by a Dynamixel,
/// as stored using each bit to represent a different error. For more info, see
/// https://emanual.robotis.com/docs/en/dxl/protocol1/#status-packetreturn-packet
#[derive(Clone, Debug)]
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
        let mut error_code = vec![0; 8];

        for err in errors.iter() {
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

            error_code[index] = 1;
        }

        let bin_err = error_code
            .into_iter()
            .map(|i| i.to_string())
            .collect::<String>();
        u8::from_str_radix(&bin_err, 2).unwrap()
    }

    /// Gets the list of error types given a numeric representation of the
    /// error
    pub fn get_error_types(error: &u8) -> Vec<StatusType> {
        let mut errors: Vec<StatusType> = vec![];

        for (i, c) in format!("{:0>8b}", error).chars().enumerate() {
            if c == '1' {
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

                if error_type.is_some() {
                    errors.push(error_type.unwrap());
                }
            }
        }

        errors
    }
}

/// Represents the different kinds of values that can be stored in the packet's
/// error/instruction column.
#[derive(Clone, Debug)]
pub enum PacketType {
    Instruction(InstructionType),
    Status(Vec<StatusType>),
}

/// Reresents either an outgoing or incoming packet.
#[derive(Clone, Debug)]
pub struct Packet {
    pub id: u8,
    length: u8,
    pub packet_type: PacketType,
    pub parameters: Vec<u8>,
    checksum: u8,
}

impl Packet {
    /// Calculates the checksum for the packet
    pub fn checksum(&self) -> u8 {
        let mut sum = self.id as usize + self.length as usize;
        sum += self.parameters.iter().map(|i| *i as usize).sum::<usize>();
        sum += match &self.packet_type {
            PacketType::Instruction(instruction) => u8::from(*instruction),
            PacketType::Status(statuses) => StatusType::get_error_code(statuses),
        } as usize;

        let chk: u8 = if sum > 255 {
            (sum as u8) & 0xFF
        } else {
            sum as u8
        };

        !chk
    }

    /// Provides packet-crafting functionality for servo communication. If you want
    /// to actually write to the servo, see the ConnectionHandler trait (TODO: LINK).
    pub fn generate(&self) -> Result<Vec<u8>, String> {
        if let PacketType::Instruction(instruction) = self.packet_type {
            let mut packet = vec![255, 255, self.id, self.length, u8::from(instruction)];
            packet.extend(&self.parameters);
            packet.push(self.checksum);

            Ok(packet)
        } else {
            Err("You cannot write a status packet to a servo!".to_string())
        }
    }

    /// Creates a new protocol 1 packet
    pub fn new(id: u8, packet_type: PacketType, parameters: Vec<u8>) -> Packet {
        let mut packet = Packet {
            id,
            length: parameters.len() as u8 + 2u8,
            packet_type,
            parameters,
            checksum: 0,
        };

        packet.checksum = packet.checksum();
        packet
    }
}
