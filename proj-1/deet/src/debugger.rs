
use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use crate::inferior::Status;
use crate::inferior::Breakpoint;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    dwarf_data: DwarfData,
    breakpoints: Vec<usize>,
    breakpoints_map: HashMap<usize, Breakpoint>,
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

        // debug
        debug_data.print();

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            dwarf_data: debug_data,
            breakpoints: Vec::new(),
            breakpoints_map: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
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
                            Status::Stopped(signal, rip) => {
                                println!("Child stopped (signal {})", signal);
                                match self.dwarf_data.get_line_from_addr(rip) {
                                    Some(line) => {
                                        println!("Stopped at {}:{}", line.file, line.number);
                                    },
                                    None => {}
                                } 
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
                        Status::Stopped(signal, rip) => {
                            println!("Child stopped (signal {})", signal);
                            match self.dwarf_data.get_line_from_addr(rip) {
                                Some(line) => {
                                    println!("Stopped at {}:{}", line.file, line.number);
                                },
                                None => {}
                            }              
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
                    match infer.print_backtrace(&self.dwarf_data) {
                        Ok(()) => {}
                        Err(e) => println!("{}", e),
                    }
                }
                DebuggerCommand::Break(args) => {
                    if args.len() != 1 {
                        println!("invalid break targets");
                        continue;
                    }
                    let token = &args[0];
                    if token.starts_with("*") { // address mode
                        let addr_str = &token[1..];
                        // println!("{}", addr_str);
                        let addr_without_0x = if addr_str.to_lowercase().starts_with("0x") {
                            &addr_str[2..]
                        } else {
                            &addr_str
                        };
                        let decoded = usize::from_str_radix(addr_without_0x, 16);
                        match decoded {
                            Ok(addr) => {
                                self.breakpoints.push(addr);
                                self.breakpoints_map.insert(addr, Breakpoint::new(addr, 0).unwrap());
                                println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), addr);
                            }
                            Err(e) => {
                                println!("Given address error: {}", e);
                            }
                        }
                    }
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
