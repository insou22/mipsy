use std::str::FromStr;
use super::{prompt};
use colored::*;
use mipsy_lib::*;
use std::io::Write;

pub(crate) struct Handler {
    pub(crate) verbose: bool,
    pub(crate) exit_status: Option<i32>,
    pub(crate) breakpoint:  bool,
}

impl Handler {
    pub fn make(verbose: bool) -> Handler {
        Self {
            verbose,
            exit_status: None,
            breakpoint: false,
        }
    }
}

fn get_input<T: FromStr>(name: &str, verbose: bool) -> T {
    let prompt: Box<dyn Fn()> = 
        if verbose {
            Box::new(|| prompt::error_nonl(format!("bad input (expected {}), try again: ", name)))
        } else {
            Box::new(|| print!("[mipsy] bad input (expected {}), try again: ", name))
        };

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<T>() {
            Ok(n) => return n,
            Err(_) => {
                (prompt)();
                std::io::stdout().flush().unwrap();
                continue;
            }
        };
    }
}

impl<'a> RuntimeHandler for Handler {
    fn sys1_print_int(&mut self, val: i32) {
        if self.verbose {
            prompt::syscall_nl(1, format!("print_int: {}", val.to_string().green()));
        } else {
            print!("{}", val);
        }
    }

    fn sys2_print_float(&mut self, val: f32) {
        if self.verbose {
            prompt::syscall_nl(2, format!("print_float: {}", val.to_string().green()));
        } else {
            print!("{}", val);
        }
    }

    fn sys3_print_double(&mut self, val: f64) {
        if self.verbose {
            prompt::syscall_nl(3, format!("print_double: {}", val.to_string().green()));
        } else {
            print!("{}", val);
        }
    }

    fn sys4_print_string(&mut self, val: String) {
        if self.verbose {
            prompt::syscall_nl(
                4,
                format!(
                    "print_string: \"{}\"",
                    val.escape_default().to_string().green()
                ),
            );
        } else {
            print!("{}", val);
        }
    }

    fn sys5_read_int(&mut self) -> i32 {
        if self.verbose {
            prompt::syscall(5, "read_int: ");
            std::io::stdout().flush().unwrap();
        }

        get_input("int", self.verbose)
    }

    fn sys6_read_float(&mut self) -> f32 {
        todo!()
    }

    fn sys7_read_double(&mut self) -> f64 {
        todo!()
    }

    fn sys8_read_string(&mut self, max_len: u32) -> String {
        if self.verbose {
            prompt::syscall(5, format!("read_string [size={}]: ", max_len));
            std::io::stdout().flush().unwrap();
        }

        loop {
            let input: String = get_input("string", self.verbose);

            if input.len() > max_len as usize {
                prompt::error(format!("bad input (max string length specified as {}, given string is {} bytes), try again: ", max_len, input.len()));
                prompt::error_nonl("please try again: ");
                std::io::stdout().flush().unwrap();
                continue;
            }

            return input;
        }
    }

    fn sys9_sbrk(&mut self, _val: i32) {
        todo!()
    }

    fn sys10_exit(&mut self) {
        if self.verbose {
            prompt::syscall_nl(10, "exit");
        }

        self.exit_status = Some(0);
    }

    fn sys11_print_char(&mut self, val: char) {
        if self.verbose {
            prompt::syscall_nl(
                11,
                format!("print_char: '{}'", val.escape_default().to_string().green()),
            );
        } else {
            print!("{}", val);
        }
    }

    fn sys12_read_char(&mut self) -> char {
        if self.verbose {
            prompt::syscall(5, "read_character: ");
            std::io::stdout().flush().unwrap();
        }

        get_input("character", self.verbose)
    }

    fn sys13_open(&mut self, _path: String, _flags: flags, _mode: mode) -> fd {
        todo!()
    }

    fn sys14_read(&mut self, _fd: fd, _buffer: void_ptr, _len: len) -> n_bytes {
        todo!()
    }

    fn sys15_write(&mut self, _fd: fd, _buffer: void_ptr, _len: len) -> n_bytes {
        todo!()
    }

    fn sys16_close(&mut self, _fd: fd) {
        todo!()
    }

    fn sys17_exit_status(&mut self, val: i32) {
        if self.verbose {
            prompt::syscall_nl(
                17,
                format!(
                    "exit_status: {}",
                    if val == 0 {
                        val.to_string().green()
                    } else {
                        val.to_string().red()
                    }
                ),
            );
        }

        self.exit_status = Some(val);
    }

    fn breakpoint(&mut self) {
        self.breakpoint = true;
    }
}
