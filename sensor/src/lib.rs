pub mod virtual_sensor;

pub enum SensorType {
    Measurement,
    DataStream,
}

// This will need to change later to accommodate data streams
pub struct SensorValue {
    pub value_type: SensorType,
    pub value: usize,
}

pub struct Sensor {
    pub sensor_type: SensorType,
    pub model_name: String,
    pub last_value: SensorValue,
}

pub trait DataSensor {
    fn get_data(&mut self) -> SensorValue;
    // fn get_historical_data() -> Vec<SensorValue>;
}
