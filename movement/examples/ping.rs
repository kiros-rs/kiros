// PLEASE NOTE: this example is using a very unstable API, mainly to be used
// a reference for future use.

use connection::{wired::TTYPort, Connect, ConnectionSettings};
use movement::dynamixel::{protocol_one, DynamixelError, PacketOperation};
use std::io::{Read, Write};

fn main() -> Result<(), DynamixelError> {
    // Enable the LED on a connected Dynamixel (ID: 1, Model: AX-12A)
    let mut port = TTYPort::connect(&ConnectionSettings::default());
    port.write(&protocol_one::ping(0xFE).generate());

    let mut buf: Vec<u8> = vec![];
    port.read_to_end(&mut buf);
    println!("{:?}", buf);
    // let mut buf = [0u8; 1];
    // port.read_exact(&mut buf).unwrap();
    // println!("{:?}", dynamixel::protocol_one::write(1, 25, dynamixel::Parameter::unsigned(1)).generate());
    // let mut dxl: Dynamixel<TTYPort, u64> = Dynamixel::from_template("ax-12a", port, Protocol::ProtocolOne).unwrap();

    // dxl.ping()?;

    Ok(())
}
