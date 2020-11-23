// PLEASE NOTE: this example is using a very unstable API, mainly to be used
// a reference for future use.

// TODO: Update this to work with newer APIs & broader range of hardware
use connection::usb;
use movement::dynamixel::{self, protocol_one::ProtocolOne};

fn main() {
    // Enable the LED on a connected Dynamixel (ID: 1, Model: AX-12A)
    let mut port = usb::connect_usb("/dev/ttyACM0", 1_000_000);
    let dxl = dynamixel::Dynamixel {
        id: dynamixel::DynamixelID::ID(1),
    };
    let dynamixel::Packet::ProtocolOne(pck) = dxl.write(25, 1);

    dynamixel::connection::write_packet(&mut port, pck);
    std::thread::sleep(std::time::Duration::from_millis(50));
    println!(
        "Servo response: {:?}",
        dynamixel::connection::read_exact_packet(&mut port, 6)
    );
}
