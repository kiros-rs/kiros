use sensor::{
    virtual_sensor::{VirtualDataType, VirtualSensor},
    DataSensor,
};
use std::thread;
use std::time::Duration;

fn main() {
    let mut sensor = VirtualSensor::new(VirtualDataType::Constant(1));

    loop {
        println!("Got data from sensor: {}", sensor.get_data().value);
        thread::sleep(Duration::from_secs(1));
    }
}
