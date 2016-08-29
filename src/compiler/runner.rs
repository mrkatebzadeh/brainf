#![warn(trivial_numeric_casts)]

use std::fs::File;
use std::io::prelude::Read;
use std::io::Error;
use std::path::Path;
use std::process;
use getopts::{Options, Matches};
use tempfile::NamedTempFile;
use compiler::diagnostics::{Info, Level};
use compiler::bfir;
use compiler::execution;
use compiler::llvm;
use compiler::peephole;
use compiler::shell;

fn slurp(path: &str) -> Result<String, Info> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(message) => {
            return Err(Info {
                level: Level::Error,
                filename: path.to_owned(),
                message: format!("{}", message),
                position: None,
                source: None,
            })
        }
    };

    let mut contents = String::new();

    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(message) => {
            Err(Info {
                level: Level::Error,
                filename: path.to_owned(),
                message: format!("{}", message),
                position: None,
                source: None,
            })
        }
    }
}

fn executable_name(bf_path: &str) -> String {
    let bf_file_name = Path::new(bf_path).file_name().unwrap().to_str().unwrap();

    let mut name_parts: Vec<_> = bf_file_name.split('.').collect();
    let parts_len = name_parts.len();
    if parts_len > 1 {
        name_parts.pop();
    }

    name_parts.join(".")
}

fn print_usage(bin_name: &str, opts: Options) {
    let brief = format!("Usage: {} SOURCE_FILE [options]", bin_name);
    print!("{}", opts.usage(&brief));
}

fn convert_io_error<T>(result: Result<T, Error>) -> Result<T, String> {
    match result {
        Ok(value) => Ok(value),
        Err(e) => Err(format!("{}", e)),
    }
}

fn compile_file(matches: &Matches) -> Result<(), String> {
    let path = &matches.free[0];

    let src = match slurp(path) {
        Ok(src) => src,
        Err(info) => {
            return Err(format!("{}", info));
        }
    };

    let mut instrs = match bfir::parse(&src) {
        Ok(instrs) => instrs,
        Err(parse_error) => {
            let info = Info {
                level: Level::Error,
                filename: path.to_owned(),
                message: parse_error.message,
                position: Some(parse_error.position),
                source: Some(src),
            };
            return Err(format!("{}", info));
        }
    };

    let opt_level = matches.opt_str("opt").unwrap_or_else(|| String::from("2"));
    if opt_level != "0" {
        let pass_specification = matches.opt_str("passes");
        let (opt_instrs, warnings) = peephole::optimize(instrs, &pass_specification);
        instrs = opt_instrs;

        for warning in warnings {
            let info = Info {
                level: Level::Warning,
                filename: path.to_owned(),
                message: warning.message,
                position: warning.position,
                source: Some(src.clone()),
            };
            println!("{}", info);
        }
    }

    if matches.opt_present("dump-ir") {
        for instr in &instrs {
            println!("{}", instr);
        }
        return Ok(());
    }

    let (state, warning) = if opt_level == "2" {
        execution::execute(&instrs, execution::MAX_STEPS)
    } else {
        let mut init_state = execution::ExecutionState::initial(&instrs[..]);
        init_state.start_instr = Some(&instrs[0]);
        (init_state, None)
    };

    if let Some(warning) = warning {
        let info = Info {
            level: Level::Warning,
            filename: path.to_owned(),
            message: warning.message,
            position: warning.position,
            source: Some(src),
        };
        println!("{}", info);
    }

    let target_triple = matches.opt_str("target");
    let mut llvm_module = llvm::wrapper::compile_to_module(path, target_triple.clone(), &instrs, &state);

    if matches.opt_present("dump-llvm") {
        let llvm_ir_cstr = llvm_module.to_cstring();
        let llvm_ir = String::from_utf8_lossy(llvm_ir_cstr.as_bytes());
        println!("{}", llvm_ir);
        return Ok(());
    }

    let llvm_opt_raw = matches.opt_str("llvm-opt").unwrap_or("3".to_owned());
    let mut llvm_opt = llvm_opt_raw.parse::<i64>().unwrap_or(3);
    if llvm_opt < 0 || llvm_opt > 3 {
        llvm_opt = 3;
    }

    llvm::wrapper::optimise_ir(&mut llvm_module, llvm_opt);

    let object_file = try!(convert_io_error(NamedTempFile::new()));
    let obj_file_path = object_file.path().to_str().expect("path not valid utf-8");
    try!(llvm::wrapper::write_object_file(&mut llvm_module, &obj_file_path));

    let output_name = executable_name(path);
    try!(link_object_file(&obj_file_path, &output_name, target_triple));

    let strip_opt = matches.opt_str("strip").unwrap_or("no".to_owned());
    if strip_opt == "yes" {
        try!(strip_executable(&output_name))
    }

    Ok(())
}

fn link_object_file(object_file_path: &str,
                    executable_path: &str,
                    target_triple: Option<String>)
                    -> Result<(), String> {
    let clang_args = if let Some(ref target_triple) = target_triple {
        vec![object_file_path, "-target", &target_triple, "-o", &executable_path[..]]
    } else {
        vec![object_file_path, "-o", &executable_path[..]]
    };

    shell::run_shell_command("clang", &clang_args[..])
}

fn strip_executable(executable_path: &str) -> Result<(), String> {
    let strip_args = ["-s", &executable_path[..]];
    shell::run_shell_command("strip", &strip_args[..])
}

pub fn compile(matches: &Matches) {
    

    
    match compile_file(&matches) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
            process::exit(3);
        }
    }
}
