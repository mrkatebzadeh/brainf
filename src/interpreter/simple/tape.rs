use std::iter;
extern crate text_io;
pub struct Tape {
    content: Vec<u8>,
    pointer: usize
}

impl Tape {
    pub fn new() -> Tape {
        Tape {
            content: iter::repeat(0u8).take(30000).collect(),
            pointer: 0,
        }
    }

    pub fn inc(&mut self) {
        *self.content.get_mut(self.pointer).unwrap() += 1u8;
    }

    pub fn dec(&mut self) {
        *self.content.get_mut(self.pointer).unwrap() -= 1u8;
    }

    pub fn next(&mut self) {
        self.pointer += 1;
    }

    pub fn prev(&mut self) {
        self.pointer -= 1;
    }

    pub fn value(&self) -> u8 {
        self.content[self.pointer]
    }

    pub fn zero(&self) -> bool {
        self.value() == 0
    }

    pub fn not_zero(&self) -> bool {
        !self.zero()
    }

    pub fn read_value(&mut self) {
        let value: u8;
        value = read!("{}\n");
        // scan!("{}\n", value);
        self.set_value(value)
    }

    fn set_value(&mut self, value: u8) {
        *self.content.get_mut(self.pointer).unwrap() = value;
    }
}
