use super::{DataSensor, DataValue};

/// An abstract representation of a signed integer collected by the sensor
pub struct NumericDataValue {
    pub model_name: String,
    pub last_value: DataValue<isize>,
    pub historical_values: Vec<DataValue<isize>>,
    pub stores_historical: bool,
}

/// An API to extend DataSensor meant for sensors returning integer readings
pub trait NumericDataSensor: DataSensor<isize> {
    fn get_data(&self) -> DataValue<isize>;
}
