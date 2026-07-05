use std::io::Write;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

mod rsp;

#[derive(Debug, PartialEq)]
/// see: https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Result-Records.html#GDB_002fMI-Result-Records
pub enum ResultRecord {
    Done,
    Running,
    Connected,
    Error,
    Exit,
}

/// Interfaces with a GDB process over it's machine interface.
pub struct GDBInterface {
    child: std::process::Child,
}

impl GDBInterface {
    pub fn open() -> Self {
        let mut child = Command::new("arm-none-eabi-gdb")
            .arg("--interpreter=mi2") // enable machine interface mode
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("to create a command child");

        // Read the initial GDB output.
        if let Some(stdout) = &mut child.stdout {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.expect("line");

                println!("GDB responded: `{}`", line);

                if line.starts_with("(gdb)") {
                    println!("gdb ready");
                    break;
                }
            }
        } else {
            panic!("failed to take stdout")
        }

        // Always wait for the process to clean up
        // child.wait()?;
        Self { child }
    }

    /// Connects the GDB instance to a remote GDB server at the specified address.
    pub fn target_extended_remote(
        &mut self,
        target: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Take ownership of stdin and write the command.
        if let Some(stdin) = &mut self.child.stdin {
            writeln!(stdin, "-target-select extended-remote {}", target)?;
        } else {
            panic!("failed to take stdin")
        }

        // Take ownership of the stdout and wait for a result.
        if let Some(stdout) = &mut self.child.stdout {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.expect("line");

                if line.starts_with("^connected") {
                    println!("connected to target!");
                } else if line.starts_with("^error") {
                    println!("failed to connect to target!");
                } else if line.starts_with("(gdb)") {
                    println!("(gdb)");
                    break;
                } else {
                    println!("unexpected output: {}", line);
                }
            }
        } else {
            panic!("failed to take stdout")
        }

        Ok(())
    }

    /// Run monitor commands on GBD.
    pub fn monitor(&mut self, args: Option<Vec<String>>) -> Result<(), Box<dyn std::error::Error>> {
        // Take ownership of stdin and write the command.
        if let Some(stdin) = &mut self.child.stdin {
            write!(stdin, "monitor")?;

            // Write in args if supplied.
            if let Some(args) = args {
                for arg in args {
                    write!(stdin, " {}", arg)?;
                }
            }

            // Write end of line.
            write!(stdin, "\n")?;
        } else {
            panic!("failed to take stdin")
        }

        // Take ownership of the stdout and wait for a result.
        if let Some(stdout) = &mut self.child.stdout {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.expect("line");

                if line.starts_with("(gdb)") {
                    println!("(gdb)");
                    break;
                } else if line.starts_with("@") {
                    println!("{}", line); // TODO: strip starting and ending "
                } else {
                    println!("debug: {}", line);
                }
            }
        } else {
            panic!("failed to take stdout")
        }

        Ok(())
    }
}
