pub struct Program {
    content: Vec<char>,
    pub pointer: usize,
}

impl Program {
    pub fn new(content: Vec<char>) -> Program {
        Program {
            content,
            pointer: 0,
        }
    }

    pub fn finished(&self) -> bool {
        self.pointer == self.content.len() as usize
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
                _ => self.fast_forward(count),
            }
        }
    }

    pub fn rewind(&mut self, count: usize) {
        if count != 0 {
            self.back();

            match self.command() {
                '[' => self.rewind(count - 1),
                ']' => self.rewind(count + 1),
                _ => self.rewind(count),
            }
        }
    }
}
