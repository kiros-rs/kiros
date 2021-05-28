pub mod numeric_sensor;

use serde::{Serialize, Deserialize};

/// A representation of all common units of data that may be processed
#[derive(Serialize, Deserialize, Debug)]
pub enum DataUnit {
    Second,
    Pulse,
    RevolutionsPerMinute,
    DegreesCelcius,
    Volts,
    Percentage,
    Amps,
    Other,
}

/// An abstract representation of data collected by the sensor
pub struct DataValue<T> {
    pub unit: DataUnit,
    pub power: isize,
    pub value: T,
}

/// An abstract representation of a sensor on the robot
pub struct Sensor<T> {
    pub model_name: String,
    pub last_value: DataValue<T>,
    pub historical_values: Vec<DataValue<T>>,
    pub stores_historical: bool,
}

/// The most generic API a sensor can possess
pub trait DataSensor<T> {
    fn get_data(&self) -> DataValue<T>;
    // fn get_historical_data() -> Vec<SensorValue>;
}
