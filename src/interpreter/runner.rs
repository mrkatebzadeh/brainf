extern crate core;

use std::path::Path;
use interpreter::simple::tape;
use interpreter::simple::program;

pub fn interpret(path : &Path) {
    let mut program = program::Program::new(&path);
    let mut tape = tape::Tape::new();

    while program.pointer != program.len() as usize {
        match program.command() {
            '+' => tape.inc(),
            '-' => tape.dec(),
            '>' => tape.next(),
            '<' => tape.prev(),
            '.' => print!("{}", tape.value() as char),
            ',' => tape.read_value(),
            '[' if tape.zero() => program.fast_forward(1),
            ']' if tape.not_zero() => program.rewind(1),
            _ => ()
        }

        program.forward();
    }
}
