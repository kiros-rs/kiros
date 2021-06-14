use crate::*;
pub use serialport::TTYPort;
use std::convert::TryInto;

// NOTE: This function currently assumes that all configuration is valid
// When adding errors to the library, make sure to provide detailed info when the configuration is invalid
// TODO: Make this less platform-specific by calling open() from builder directly
impl Connect for TTYPort {
    fn connect(settings: &ConnectionSettings) -> Self {
        let builder = serialport::new(
            settings.path.as_ref().unwrap(),
            settings.baudrate.unwrap().try_into().unwrap(),
        )
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(settings.timeout.unwrap());

        TTYPort::open(&builder).unwrap()
    }
}
