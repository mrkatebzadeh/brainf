use std::{io, path::Path};
mod args;
use interpreter::{interpret, read_program};

fn main() -> anyhow::Result<()> {
    let args = args::parse();

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
        todo!()
    }
    Ok(())
}
