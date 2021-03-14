// PLEASE NOTE: this example is using a very unstable API, mainly to be used
// a reference for future use.

// TODO: Update this to work with newer APIs & broader range of hardware
use connection::usb;
use movement::dynamixel::{self, protocol_one::{ProtocolOne, Packet, InstructionType}, Dynamixel};

fn main() {
    // Enable the LED on a connected Dynamixel (ID: 1, Model: AX-12A)
    let mut port = usb::connect_usb("/dev/ttyACM0", 1_000_000);
    let mut dxl = Dynamixel::new_empty(&mut port);

    dxl.ping();
    dxl.write(25, 1);
    println!(
        "Servo response: {:?}",
        dynamixel::servo_connection::read_exact_packet(&mut port, 6)
    );
}
