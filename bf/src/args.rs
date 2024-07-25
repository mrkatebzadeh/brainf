use clap::{ArgAction, Parser};
use std::fmt;

#[derive(Parser)]
#[command(version, about, long_about = None, term_width = 80)]
pub(crate) struct Args {
    #[arg(short = 'i', long, help = "Interpret input")]
    #[clap(action=ArgAction::SetTrue, conflicts_with = "compile")]
    pub interpret: bool,

    #[arg(short = 'c', long, help = "Compile input")]
    #[clap(action=ArgAction::SetTrue, conflicts_with = "interpret")]
    pub compile: bool,

    #[arg(short = 'O', long, help = "Optimization level (0-2)")]
    #[clap(default_value_t = 0)]
    pub optimizatoin: u32,

    #[arg(long, help = "Print the generated LLVM IR")]
    #[clap(action=ArgAction::SetTrue)]
    pub dump_llvm: bool,

    #[arg(long, help = "Print the generated BF IR")]
    #[clap(action=ArgAction::SetTrue)]
    pub dump_ir: bool,

    #[arg(long, help = "LLVM Optimization level (0-3)")]
    #[clap(default_value_t = 0)]
    pub llvm_opt: u32,

    #[arg(short = 'g', long, help = "Include debugging information")]
    #[clap(action=ArgAction::SetTrue)]
    pub debug: bool,

    #[arg(short = 't', long, help = "LLVM target triple")]
    pub target: Option<String>,

    #[arg(short = 'f', long, help = "BF input file")]
    pub file: String,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = format!(
            "
        Interpret:     {}
        Compile:       {}
        Optimization:  {}
        Dump_LLVM:     {}
        DUMP_IR:       {}
        LLVM_Opt:      {}
        Debug:         {}
        Target:        {:?}
        File:          {}
",
            self.interpret,
            self.compile,
            self.optimizatoin,
            self.dump_llvm,
            self.dump_ir,
            self.llvm_opt,
            self.debug,
            self.target,
            self.file,
        );
        write!(f, "{}", string)
    }
}

pub(crate) fn parse() -> Args {
    Args::parse()
}
