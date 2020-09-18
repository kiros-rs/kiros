pub mod dynamixel;

pub trait Servo {
    fn goto(pos: usize, speed: usize) -> Result<(), String>;
}

pub trait Motor {
    fn set_speed(speed: usize) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
