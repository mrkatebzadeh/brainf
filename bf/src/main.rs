use std::{
    io::{self, Stdout},
    path::Path,
};
mod args;
use interpreter::{interpret, read_program};

fn main() -> anyhow::Result<()> {
    let args = args::parse();
    // let default_triple_cstring = compiler::llvm::wrapper::get_default_target_triple();
    // let default_triple = default_triple_cstring.to_str().unwrap();

    if args.file.is_empty() {
        println!("Please specify a BF file.");
        std::process::exit(2);
    }
    if !args.interpret && !args.compile {
        println!("Please select either interpret mode or compile mode.");
        std::process::exit(3);
    }

    let path = Path::new(&args.file);
    if args.interpret {
        let content = read_program(path)?;

        let mut stdout = io::stdout().lock();
        interpret(content, &mut stdout);
    } else if args.compile {
        // compiler::runner::compile(&args);
    }
    Ok(())
}
