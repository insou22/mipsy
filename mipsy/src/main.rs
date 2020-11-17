use std::str::FromStr;
use std::io::Write;

use mipsy_lib::*;
use interactive::prompt;
use clap::Clap;

mod interactive;
mod error;

#[derive(Clap, Debug)]
#[clap(version = VERSION, author = "Zac K. <zac.kologlu@gmail.com>")]
struct Opts {
    #[clap(long, about("Just compile program instead of executing"))]
    compile: bool,
    #[clap(long, about("Just compile program and output hexcodes"))]
    hex: bool,
    #[clap(long, about("With --hex: pad to 8 hex digits with zeroes"))]
    hex_pad_zero: bool,
    #[clap(long, short('v'))]
    version: bool,
    file: Option<String>,
}

fn get_input<T: FromStr>(name: &str) -> T {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<T>() {
            Ok(n) => return n,
            Err(_) => {
                print!("[mipsy] bad input (expected {}), try again: ", name);
                std::io::stdout().flush().unwrap();
                continue;
            },
        };
    }
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
        get_input("int")
    }

    fn sys6_read_float(&mut self) -> f32 {
        get_input("float")
    }

    fn sys7_read_double(&mut self) -> f64 {
        get_input("double")
    }

    fn sys8_read_string(&mut self, max_len: u32) -> String {
        loop {
            let input: String = get_input("string");

            if input.len() > max_len as usize {
                println!("[mipsy] bad input (max string length specified as {}, given string is {} bytes)", max_len, input.len());
                print!  ("[mipsy] please try again: ");
                std::io::stdout().flush().unwrap();
                continue;
            }

            return input;
        }
    }

    fn sys9_sbrk(&mut self, _val: i32) {
        // no-op
    }

    fn sys10_exit(&mut self) {
        std::process::exit(0);
    }

    fn sys11_print_char(&mut self, val: char) {
        print!("{}", val);
    }

    fn sys12_read_char(&mut self) -> char {
        get_input("character")
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

    fn breakpoint(&mut self) {
        // no-op
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    if let None = opts.file {
        interactive::launch();
    }

    let file = opts.file.as_ref().unwrap();

    let file_contents = std::fs::read_to_string(file).expect("Could not read file {}");

    match run(&opts, &file_contents) {
        Ok(_) => {}
        Err(MipsyError::Compile(error)) => {
            prompt::error(format!("failed to compile `{}`", file));
            error::compile_error::handle(error, &file_contents, None, None, None);
        }
        Err(MipsyError::CompileLoc { line, col, col_end, error }) => {
            prompt::error(format!("failed to compile `{}`", file));
            error::compile_error::handle(error, &file_contents, line, col, col_end);
        }
        Err(MipsyError::Runtime(error)) => {
            println!("runtime error: {:?}", error);
        }
    }
}

fn run(opts: &Opts, file: &str) -> MipsyResult<()> {
    let iset       = mipsy_lib::inst_set()?;
    let binary     = mipsy_lib::compile(&iset, file)?;

    if opts.compile {
        let decompiled = mipsy_lib::decompile(&iset, &binary);
        println!("Compiled program:\n{}\n", decompiled);
        return Ok(())
    }

    if opts.hex {
        for &opcode in binary.text.iter() {
            if opts.hex_pad_zero {
                println!("{:08x}", opcode);
            } else {
                println!("{:x}", opcode);
            }
        }

        return Ok(());
    }

    let mut runtime = mipsy_lib::run(&binary)?;
    loop {
        let mut handler = Handler;

        runtime.step(&mut handler)?;
    }
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));
