/*
 Name        : BrainF Translator
 Author      : Siavash Katebzadeh
 Version     : 1
 Copyright   : GPL v2
 Description : Main File
*/
extern crate llvm_sys;
extern crate itertools;
extern crate quickcheck;
extern crate rand;
extern crate tempfile;
extern crate ansi_term;
extern crate getopts;
#[macro_use]
extern crate text_io;

use std::env;
#[macro_use]
opts.optflag("v", "version", "print bfc version");
extern crate matches;
use std::path::Path;
use getopts::Options;

pub mod interpreter;
pub mod compiler;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print help");
    opts.optflag("i", "interpret", "interpret input");
    opts.optflag("c", "compile", "compile input");
    opts.optopt("O", "opt", "optimization level (0-2)", "LEVEL");
    opts.optflag("v", "version", "print brainf version");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("ERROR");
            std::process::exit(1);
        }
    };
    if matches.free.len() == 0 {
        println!("Please specify a brainf file.");
        std::process::exit(2);
    }
    if !matches.opt_present("i") && !matches.opt_present("c") {
        println! ("Please select either interpret-mode or compile mode.");
        std::process::exit(3);
    }
    // let ref path =  matches.free[0];
    let path = Path::new(&matches.free[0]);
    if matches.opt_present("i") {
        runner::interpret(&path);
    }
}
