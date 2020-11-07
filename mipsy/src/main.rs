use std::io::Write;

use mipsy_lib::*;
use clap::Clap;

mod interactive;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Zac K. <zac.kologlu@gmail.com>")]
struct Opts {
    #[clap(long, about("Step-by-step execution"))]
    step: bool,
    #[clap(long, about("Just compile program instead of executing"))]
    compile: bool,
    file: Option<String>,
}

struct Handler;
impl RuntimeHandler for Handler {
    fn sys1_print_int(&mut self, val: i32) {
        print!("{}", val);
    }

    fn sys2_print_float(&mut self, val: f32) {
        print!("{}", val);
    }

    fn sys3_print_double(&mut self, val: f64) {
        print!("{}", val);
    }

    fn sys4_print_string(&mut self, val: String) {
        print!("{}", val);
    }

    fn sys5_read_int(&mut self) -> i32 {
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<i32>() {
                Ok(n) => return n,
                Err(_) => {
                    print!("[mipsy] bad input, try again: ");
                    std::io::stdout().flush().unwrap();
                    continue;
                },
            };
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

    fn sys9_sbrk(&mut self, _val: i32) {
        todo!()
    }

    fn sys10_exit(&mut self) {
        std::process::exit(0);
    }

    fn sys11_print_char(&mut self, val: char) {
        print!("{}", val);
    }

    fn sys12_read_char(&mut self) -> char {
        todo!()
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
        std::process::exit(val);
    }
}

fn main() -> MipsyResult<()> {
    let opts: Opts = Opts::parse();

    if let None = opts.file {
        interactive::launch();
    }

    let file_contents = std::fs::read_to_string(&opts.file.as_ref().unwrap()).expect("Could not read file {}");

    let iset       = mipsy_lib::inst_set()?;
    let binary     = mipsy_lib::compile(&iset, &file_contents)?;

    if opts.compile {
        let decompiled = mipsy_lib::decompile(&iset, &binary);
        println!("Compiled program:\n{}\n", decompiled);
        return Ok(())
    }

    let mut runtime = mipsy_lib::run(&binary)?;
    loop {
        runtime.step(&mut Handler)?;
    }
}
