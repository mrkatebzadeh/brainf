use std::process::Command;

fn shell_command(command: &str, args: &[&str]) -> Result<String, String> {
    let mut c = Command::new(command);
    for arg in args {
        c.arg(arg);
    }

    match c.output() {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                Ok((*stdout).to_owned())
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Err((*stderr).to_owned())
            }
        }
        Err(_) => Err(format!("Could not execute '{}'. Is it on $PATH?", command)),
    }
}

pub fn run_shell_command(command: &str, args: &[&str]) -> Result<(), String> {
    match shell_command(command, args) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
