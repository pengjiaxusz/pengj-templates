//! The standalone `gate` binary.
//!
//! The same front end is also reachable as `pengj-templates-cli gate ...`,
//! so a project that already installs the template CLI does not need a second
//! binary just to run the commit gate.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match pengj_gate::cli::run(&args) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("[gate] {message}");
            ExitCode::from(2)
        }
    }
}
