pub mod dynamixel;

/// Generic functionality that should be exposed by any connected servo
pub trait Servo {
    fn set_pos(&mut self, pos: usize) -> Result<(), String>;
}

/// Generic functionality that should be exposed by any connected motor
pub trait Motor {
    fn set_speed(&mut self, speed: usize) -> Result<(), String>;
    fn get_speed(&self) -> usize;
    fn get_max_speed(&self) -> usize;
}
