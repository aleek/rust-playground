// Copyright 2024 Aleksander Dutkowski

#[derive(Debug)]
pub struct Calc {
    first_arg: u32,
    second_arg: u32
}

pub fn new_calc() -> Calc {
    return Calc{
        first_arg: 0,
        second_arg: 0,
    };
}

impl Calc {
    pub fn set_first_argument(&mut self, arg: u32) -> bool {
        self.first_arg = arg;

        return true;
    }

    pub fn set_second_argument(&mut self, arg: u32) -> bool {
        self.second_arg = arg;

        return true;
    }

    pub fn sum(&self) -> u32 {
        return self.first_arg + self.second_arg;
    }
}