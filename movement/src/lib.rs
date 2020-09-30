pub mod dynamixel;

pub trait Servo {
    fn set_pos(&mut self, pos: usize) -> Result<(), String>;
}

pub trait Motor {
    fn set_speed(&mut self, speed: usize) -> Result<(), String>;
    fn get_speed(&self) -> usize;
    fn get_max_speed(&self) -> usize;
}
