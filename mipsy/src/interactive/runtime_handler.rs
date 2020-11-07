use super::{prompt, State};
use colored::*;
use mipsy_lib::*;
use std::io::Write;

pub(crate) struct Handler {
    pub(crate) verbose: bool,
    pub(crate) exit_status: Option<i32>,
}

impl Handler {
    pub fn make(verbose: bool) -> Handler {
        Self {
            verbose,
            exit_status: None,
        }
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
            loop {
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                match input.trim().parse::<i32>() {
                    Ok(n) => return n,
                    Err(_) => {
                        prompt::error_nonl("bad input, try again: ");
                        continue;
                    }
                };
            }
        } else {
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                match input.trim().parse::<i32>() {
                    Ok(n) => return n,
                    Err(_) => {
                        print!("[mipsy] bad input, try again: ");
                        std::io::stdout().flush().unwrap();

                        continue;
                    }
                };
            }
        }
    }

    fn sys6_read_float(&mut self) -> f32 {
        todo!()
    }

    fn sys7_read_double(&mut self) -> f64 {
        todo!()
    }

    fn sys8_read_string(&mut self) -> String {
        todo!()
    }

    fn sys9_sbrk(&mut self, val: i32) {
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
        todo!()
    }

    fn sys13_open(&mut self, path: String, flags: flags, mode: mode) -> fd {
        todo!()
    }

    fn sys14_read(&mut self, fd: fd, buffer: void_ptr, len: len) -> n_bytes {
        todo!()
    }

    fn sys15_write(&mut self, fd: fd, buffer: void_ptr, len: len) -> n_bytes {
        todo!()
    }

    fn sys16_close(&mut self, fd: fd) {
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
}
