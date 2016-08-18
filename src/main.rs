extern crate getopts;
use std::env;
use getopts::{Options, Matches};

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print help");
    opts.optflag("i", "interpret", "interpret input");
    opts.optflag("c", "compile", "compile input");
    opts.optopt("O", "opt", "optimization level (0-2)", "LEVEL");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("ERROR");
            std::process::exit(1);
        }
    };
}
