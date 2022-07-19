use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::process::Child;
use std::process::Command;
use std::os::unix::process::CommandExt;
use std::mem::size_of;

use crate::dwarf_data::{DwarfData, Error as DwarfError};

/// for writing breakpionts
fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}

#[derive(Clone)]
pub struct Breakpoint {
    addr: usize,
    orig_byte: u8,
}

impl Breakpoint {
    pub fn new(_addr: usize, _orig_byte: u8) -> Option<Breakpoint> {
        Some(Breakpoint{addr: _addr, orig_byte: _orig_byte})
    }
}

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &Vec<usize>) -> Option<Inferior> {
        let mut command = Command::new(target);
        command.args(args);
        unsafe {
            command.pre_exec(child_traceme);
        }
        let child = command.spawn().expect("Failed to spawn a subprocess");
        let pid = nix::unistd::Pid::from_raw(child.id() as i32);
        let mut infer = Inferior{child};
        // install breakpoints
        for i in 0..breakpoints.len()-1 {
            let addr = &breakpoints[i];
            match infer.write_byte(*addr, 0xcc as u8) {
                Ok(_) => {}
                Err(e) => {
                    println!("Fail to install Breakpoint {} at {:#x}: {}", i + 1, addr, e);
                }
            }
        }
        // check SIGTRAP
        match waitpid(pid, None).ok()? {
            WaitStatus::Stopped(_, _) => {
                // println!("#{} got {}", _pid, signal); // it works
            },
            other => {
                println!("NO SIGTRAP. Got {:?}", other);
                return None
            },
        }
        Some(infer)
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    /// Restart the program after being stopped.
    pub fn cont_exec(&self) -> Result<Status, nix::Error> {
        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    /// Kill the existed process
    /// I decide to ignore the error in it
    pub fn kill(&mut self) {
        match self.child.kill() {
            Ok(()) => {
                println!("Killing running inferior (pid {})", self.pid());
                // reap
                let rst = self.wait(None).unwrap();
                match rst {
                    Status::Signaled(_) => { // SIGKILL
                        // nothing
                    }
                    _ => {
                        println!("Error in killing.")
                    }
                }
            }
            Err(_) => {
                // It is always "No such process"
                // println!("{}", e);
            }
        }
    }

    /// Printing a backtrace
    pub fn print_backtrace(&self, dwarf_data: &DwarfData) -> Result<(), nix::Error> {
        let regs = ptrace::getregs(self.pid())?;
        // println!("%rip register: {:#x}", regs.rip);
        let mut instruction_ptr = regs.rip as usize;
        let mut base_ptr = regs.rbp as usize;
        loop {
            let line = dwarf_data.get_line_from_addr(instruction_ptr).unwrap();
            let func = dwarf_data.get_function_from_addr(instruction_ptr).unwrap();
            println!("{} ({}:{})", func, line.file, line.number);
            if func == "main" {
                break;
            }
            instruction_ptr = ptrace::read(self.pid(), (base_ptr+8) as ptrace::AddressType)? as usize;
            base_ptr = ptrace::read(self.pid(), base_ptr as ptrace::AddressType)? as usize;
        } 
        Ok(())
    }

    /// for writing breakpoints
    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }
}
