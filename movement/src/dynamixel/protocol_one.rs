#[derive(Debug)]
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

impl InstructionType {
    pub fn get_instruction_code(instruction: &InstructionType) -> u8 {
        match *instruction {
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

    pub fn get_instruction_type(instruction: &u8) -> Result<InstructionType, String> {
        match *instruction {
            1 => Ok(InstructionType::Ping),
            2 => Ok(InstructionType::Read),
            3 => Ok(InstructionType::Write),
            4 => Ok(InstructionType::RegWrite),
            5 => Ok(InstructionType::Action),
            6 => Ok(InstructionType::Reset),
            7 => Ok(InstructionType::Reboot),
            8 => Ok(InstructionType::SyncWrite),
            9 => Ok(InstructionType::BulkRead),
            _ => Err("Instruction out of range!".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum StatusType {
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

#[derive(Debug)]
pub enum PacketType {
    Instruction(InstructionType),
    Status(Vec<StatusType>),
}

#[derive(Debug)]
pub struct Packet {
    pub id: u8,
    length: u8,
    pub packet_type: PacketType,
    pub parameters: Vec<u8>,
    checksum: u8,
}

impl Packet {
    fn calc_checksum(&mut self) {
        let mut sum = self.id as usize + self.length as usize;
        sum += self.parameters.iter().map(|i| *i as usize).sum::<usize>();
        sum += match &self.packet_type {
            PacketType::Instruction(instruction) => {
                InstructionType::get_instruction_code(instruction)
            }
            PacketType::Status(statuses) => StatusType::get_error_code(statuses),
        } as usize;

        let chk: u8 = if sum > 255 {
            (sum as u8) & 0xFF
        } else {
            sum as u8
        };

        self.checksum = !chk;
    }

    // Should this become a builder pattern?
    pub fn new(id: u8, packet_type: PacketType, parameters: Vec<u8>) -> Packet {
        let mut packet = Packet {
            id,
            length: parameters.len() as u8 + 2u8,
            packet_type,
            parameters,
            checksum: 0,
        };

        packet.calc_checksum();
        packet
    }
}

// THE REST OF THIS SECTION NEEDS TO BE CAREFULLY DESIGNED

// Abstract way of representing a link between computer & U2D2
// Default implementation should be using USB, but hopefully we could get some
// other interfaces working at some point (eg Arduino)
pub trait PortHandler {
    fn send();
    fn recv();
}

// Can the interface be simplified?
pub trait PacketHandler {
    fn send_packet(packet: Packet);
    fn receive_packet() -> Option<Packet>;
}

pub trait ProtocolHandler {
    fn ping() -> Packet;
    fn read() -> Packet;
    fn write() -> Packet;
    fn reg_write() -> Packet;
    fn action() -> Packet;
    fn reset() -> Packet;
    fn reboot() -> Packet;
    fn sync_write() -> Packet;
    fn bulk_read() -> Packet;
}
