use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use crate::inferior::Status;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    dwarf_data: DwarfData,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            dwarf_data: debug_data,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if let Some(inferior) = Inferior::new(&self.target, &args) {
                        // Check existed inferior and kill it
                        if self.inferior.is_some() {
                            self.inferior.as_mut().unwrap().kill();
                        }
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // get a mutable reference
                        let infer = self.inferior.as_mut().unwrap();
                        let status = infer.cont_exec().unwrap();
                        match status {
                            Status::Exited(exit_code) => println!("Child exited (status {})", exit_code),
                            Status::Stopped(signal, _) => {
                                println!("Child stopped (signal {})", signal);
                            },
                            Status::Signaled(_) => {
                                // nothing
                            },
                        };
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill();
                    }
                    return;
                }
                DebuggerCommand::Continue => {
                    // check valid inferior
                    if self.inferior.is_none() {
                        println!("The program is not being run.");
                        continue;
                    }
                    let infer = self.inferior.as_mut().unwrap();
                    let status = infer.cont_exec().unwrap();
                    match status {
                        Status::Exited(exit_code) => println!("Child exited (status {})", exit_code),
                        Status::Stopped(signal, _) => {
                            println!("Child stopped (signal {})", signal);
                        },
                        Status::Signaled(_) => {
                            // nothing
                        },
                    };
                }
                DebuggerCommand::Backtrace => {
                    // check valid inferior
                    if self.inferior.is_none() {
                        println!("The program is not being run.");
                        continue;
                    }
                    let infer = self.inferior.as_mut().unwrap();
                    infer.print_backtrace(&self.dwarf_data);
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
