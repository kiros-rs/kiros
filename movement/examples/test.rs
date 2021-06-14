use connection;
use movement::dynamixel::Dynamixel;

fn main() {
    let dxl: Dynamixel<_, u8> = Dynamixel::from_template("ax-12a", connection::virt::create_virtual(&connection::ConnectionSettings::default(), connection::virt::VirtualMode::Empty));
    println!("{:?}", dxl.control_table);
}
