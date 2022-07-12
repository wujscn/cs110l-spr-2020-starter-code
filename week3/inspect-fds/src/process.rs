use crate::open_file::OpenFile;
use std::fs;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Process {
    pub pid: usize,
    pub ppid: usize,
    pub command: String,
}

impl Process {
    pub fn new(pid: usize, ppid: usize, command: String) -> Process {
        Process { pid, ppid, command }
    }

    /// This function returns a list of file descriptor numbers for this Process, if that
    /// information is available (it will return None if the information is unavailable). The
    /// information will commonly be unavailable if the process has exited. (Zombie processes
    /// still have a pid, but their resources have already been freed, including the file
    /// descriptor table.)
    pub fn list_fds(&self) -> Option<Vec<usize>> {
        use std::fmt::Write;
        let mut path = String::new();
        write!(&mut path, "/proc/{}/fd", self.pid).ok()?;
        let read_rst = fs::read_dir(path).ok()?;
        let mut rst : Vec<usize> = Vec::new();
        for entry in read_rst {
            let entry = entry.ok()?;
            // println!("{} is found", entry.path().to_str().unwrap());
            if entry.path().is_dir() {
                continue;
            }
            let entry_name = entry.file_name();
            let fd_name = entry_name.to_str()?;
            let fd = fd_name.parse::<usize>().ok()?;
            rst.push(fd)
        }
        Some(rst)
    }

    /// This function returns a list of (fdnumber, OpenFile) tuples, if file descriptor
    /// information is available (it returns None otherwise). The information is commonly
    /// unavailable if the process has already exited.
    pub fn list_open_files(&self) -> Option<Vec<(usize, OpenFile)>> {
        let mut open_files = vec![];
        for fd in self.list_fds()? {
            open_files.push((fd, OpenFile::from_fd(self.pid, fd)?));
        }
        Some(open_files)
    }

    pub fn print(&self) {
        println!("========== \"{}\" (pid {}, ppid {}) ==========", self.command, self.pid, self.ppid);
        // print!("file descriptors: ");
        match self.list_open_files() {
            None => println!(
                "Warning: could not inspect file descriptors for this process! \
                    It might have exited just as we were about to look at its fd table, \
                    or it might have exited a while ago and is waiting for the parent \
                    to reap it."
            ),
            Some(open_files) => {
                for (fd, file) in open_files {
                    println!(
                        "{:<4} {:<15} cursor: {:<4} {}",
                        fd,
                        format!("({})", file.access_mode),
                        file.cursor,
                        file.colorized_name(),
                    );
                }
            }
        }
        println!();
    }
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "$p({})", self.pid)
    }
}

#[cfg(test)]
mod test {
    use nix::libc::sleep;

    use crate::{ps_utils, process::Process};
    use std::process::{Child, Command};

    fn start_c_program(program: &str) -> Child {
        Command::new(program)
            .spawn()
            .expect(&format!("Could not find {}. Have you run make?", program))
    }

    #[test]
    fn test_list_fds() {
        let mut test_subprocess = start_c_program("./multi_pipe_test");
        let process = ps_utils::get_target("multi_pipe_test").unwrap().unwrap();
        // ! I always got result like this : [0, 1, 2, 4, 5, 19, 20], maybe I did something wrong.
        // ! update: when I test more, I found that in the runtime, it produce 4 processes but only 2 
        // ! was expected to. It is also surprising that 2 of them had the correct fds, and other 2 
        // ! had the extra "19" and "20". Maybe it is related to the wsl technically, I would test in
        // ! real Linux envir soon.
        // process.print();
        // let pp = Process::new(process.ppid, 9, String::new());
        // pp.print();
        // use std::time::Duration;
        // use std::thread::sleep;
        // sleep(Duration::from_secs(20));
        assert_eq!(
            process
                .list_fds()
                .expect("Expected list_fds to find file descriptors, but it returned None"),
            vec![0, 1, 2, 4, 5],
            "Expected fds not correct"
        );
        let _ = test_subprocess.kill();
    }

    #[test]
    fn test_list_fds_zombie() {
        let mut test_subprocess = start_c_program("./nothing");
        let process = ps_utils::get_target("nothing").unwrap().unwrap();
        assert!(
            process.list_fds().is_none(),
            "Expected list_fds to return None for a zombie process"
        );
        let _ = test_subprocess.kill();
    }
}
