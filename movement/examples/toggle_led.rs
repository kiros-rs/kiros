// PLEASE NOTE: this example is using a very unstable API, mainly to be used
// a reference for future use.

// TODO: Update this to work with newer APIs & broader range of hardware
use connection::{wired, Connect, ConnectionSettings};
use movement::dynamixel::{protocol_one::ProtocolOne, Dynamixel, Parameter, ParameterType};

fn main() {
    // Enable the LED on a connected Dynamixel (ID: 1, Model: AX-12A)
    let mut port = wired::TTYPort::connect(&ConnectionSettings::default());
    let mut dxl: Dynamixel<_, u8> = Dynamixel::from_template("ax-12a", &mut port);
    let pck = dxl.read(25, 1);

    let led_state = match pck.parameters[0].param_type {
        ParameterType::Unsigned(val) => match val {
            0 => 1,
            1 => 0,
            _ => panic!("Invalid LED state!"),
        },
        ParameterType::Signed(_) => panic!("Unexpected unsigned value!"),
    };

    dxl.write(25, Parameter::unsigned(led_state, 1));
}
