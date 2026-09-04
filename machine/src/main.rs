//! Local stdio machine host. One process, one Application catalog.

use std::io::{self, BufRead, Write};

use rigforge_machine::{handle_line, MachineSession};

fn main() {
    let mut session = MachineSession::production();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                let _ = writeln!(io::stderr(), "machine stdin: {err}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let mut emit = |message: rigforge_machine::MachineMessage| match message.to_line() {
            Ok(encoded) => {
                let _ = writeln!(stdout, "{encoded}");
                let _ = stdout.flush();
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "machine encode: {err}");
            }
        };
        match handle_line(&mut session, &line, &mut emit) {
            Ok(true) => break,
            Ok(false) => {}
            Err(err) => {
                emit(rigforge_machine::MachineMessage::err("invalid", err));
            }
        }
    }
}
