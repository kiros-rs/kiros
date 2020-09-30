use super::{DataSensor, DataValue};

pub struct NumericDataValue {
    pub model_name: String,
    pub last_value: DataValue<isize>,
    pub historical_values: Vec<DataValue<isize>>,
    pub stores_historical: bool,
}

pub trait NumericDataSensor: DataSensor<isize> {
    fn get_data(&self) -> DataValue<isize>;
}
