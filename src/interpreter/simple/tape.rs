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
        let prev: u8 = *self.content.get_mut(self.pointer).unwrap();
        let res;
        match prev.checked_add(1){
            Some(r) => {res = r;}
            None => {res = 0;}
        }
        *self.content.get_mut(self.pointer).unwrap() = res;
    }

    pub fn dec(&mut self) {
        let prev: u8 = *self.content.get_mut(self.pointer).unwrap();
        let res;
        match prev.checked_sub(1){
            Some(r) => {res = r;}
            None => {res = u8::max_value();}
        }
        *self.content.get_mut(self.pointer).unwrap() = res;
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
        let value: String;
        value = read!("{}\n");
        let res ;
        if value.len() == 0 {
            res = self.value();
        } else {
            res = value.into_bytes()[0];
        }
        self.set_value(res)
    }

    fn set_value(&mut self, value: u8) {
        *self.content.get_mut(self.pointer).unwrap() = value;
    }
}
