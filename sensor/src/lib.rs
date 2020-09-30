pub mod numeric_sensor;

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

pub struct DataValue<T> {
    pub unit: DataUnit,
    pub power: isize,
    pub value: T,
}

pub struct Sensor<T> {
    pub model_name: String,
    pub last_value: DataValue<T>,
    pub historical_values: Vec<DataValue<T>>,
    pub stores_historical: bool,
}

pub trait DataSensor<T> {
    fn get_data(&self) -> DataValue<T>;
    // fn get_historical_data() -> Vec<SensorValue>;
}
