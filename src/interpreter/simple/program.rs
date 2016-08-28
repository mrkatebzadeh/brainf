use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct Program {
    content: Vec<char>,
    pub pointer: usize
}

impl Program {
    pub fn new(file_path: &Path) -> Program {
        let content = Program::read_program(file_path);
        Program {
            content: content,
            pointer: 0
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn forward(&mut self) {
        self.pointer += 1;
    }

    pub fn back(&mut self) {
        self.pointer -= 1;
    }

    pub fn command(&self) -> char {
        self.content[self.pointer]
    }

    pub fn fast_forward(&mut self, count: usize) {
        if count != 0 {
            self.forward();

            match self.command() {
                ']' => self.fast_forward(count - 1),
                '[' => self.fast_forward(count + 1),
                 _  => self.fast_forward(count)
            }
        }
    }

    pub fn rewind(&mut self, count: usize) {
        if count != 0 {
            self.back();

            match self.command() {
                '[' => self.rewind(count - 1),
                ']' => self.rewind(count + 1),
                 _  => self.rewind(count)
            }
        }
    }

    fn read_program(path: &Path) -> Vec<char> {
        
        let mut buffer = String::new();
        
        let mut file = File::open(path).expect("Could not open the file!");
        file.read_to_string(&mut buffer);
        // let mut content: Vec<char> = Vec::new();
        buffer.chars().collect()
        

    }
}
