//! `cargo install sno` installs this launcher. It runs the official `sno` in `~/.local/bin`, and installs it
//! first with the official installer when it is not there yet. The installer checks what it downloads.
use std::env;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, exit};

const INSTALLER: &str = "https://sno.ai/install/crate";

fn main() {
    let Some(home) = env::var_os("HOME") else {
        eprintln!("sno launcher: HOME is not set, so ~/.local/bin/sno cannot be found");
        exit(1);
    };
    let sno = PathBuf::from(home).join(".local/bin/sno");
    if !is_executable(&sno) {
        eprintln!("sno launcher: installing the official sno from {INSTALLER}");
        let script = format!("body=$(curl -fsSL {INSTALLER}) && printf '%s\\n' \"$body\" | sh");
        match Command::new("sh").args(["-c", &script]).status() {
            Ok(status) if status.success() && is_executable(&sno) => {}
            Ok(status) => {
                eprintln!(
                    "sno launcher: the installer ended with {status} and {} is missing; run: curl -fsSL {INSTALLER} | sh",
                    sno.display()
                );
                exit(status.code().unwrap_or(1));
            }
            Err(error) => {
                eprintln!("sno launcher: cannot start sh to run the installer: {error}");
                exit(1);
            }
        }
    }
    let error = Command::new(&sno).args(env::args_os().skip(1)).exec();
    eprintln!("sno launcher: cannot run {}: {error}", sno.display());
    exit(1);
}

fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
