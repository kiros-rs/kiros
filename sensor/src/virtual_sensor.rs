use crate::{DataSensor, SensorType, SensorValue};
pub enum VirtualDataType {
    Random,
    Dataset, // Should wrap data in enum here later....
    Constant(usize),
}

pub struct VirtualSensor {
    pub datatype: VirtualDataType,
}

impl VirtualSensor {
    pub fn new(datatype: VirtualDataType) -> VirtualSensor {
        VirtualSensor { datatype }
    }
}

impl DataSensor for VirtualSensor {
    fn get_data(&mut self) -> SensorValue {
        match self.datatype {
            VirtualDataType::Constant(constant) => SensorValue {
                value_type: SensorType::Measurement,
                value: constant,
            },
            _ => unimplemented!(),
        }
    }
}
