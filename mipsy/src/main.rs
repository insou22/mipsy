use std::{collections::HashMap, fmt::{Debug, Display}, fs, process, rc::Rc, str::FromStr};
use std::io::Write;

use colored::Colorize;
use mipsy_lib::{Binary, InstSet, MipsyError, MipsyResult, Runtime, RuntimeHandler, error::runtime::ErrorContext, fd, flags, len, mode, n_bytes, void_ptr};
use mipsy_interactive::{
    prompt,
};
use clap::Clap;
use text_io::try_read;

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
    files: Vec<String>,
}

fn get_input<T>(name: &str, line: bool) -> T
where
    T: FromStr + Display,
    <T as FromStr>::Err: Debug,
{
    loop {
        let result: Result<T, _> = if line {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            
            input.parse()
                .map_err(|_| ())
        } else {
            try_read!()
                .map_err(|_| ())
        };

        match result {
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
        get_input("int", false)
    }

    fn sys6_read_float(&mut self) -> f32 {
        get_input("float", false)
    }

    fn sys7_read_double(&mut self) -> f64 {
        get_input("double", false)
    }

    fn sys8_read_string(&mut self, max_len: u32) -> String {
        loop {
            let input: String = get_input("string", true);

            if input.len() > max_len as usize {
                println!("[mipsy] bad input (max string length specified as {}, given string is {} bytes)", max_len, input.len());
                print!  ("[mipsy] please try again: ");
                std::io::stdout().flush().unwrap();

                continue;
            }

            if input.len() == max_len as usize {
                println!("[mipsy] bad input (max string length specified as {}, given string is {} bytes -- must be at least one byte fewer, for NULL character), try again: ", max_len, input.len());
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
        get_input("character", false)
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

    if opts.files.is_empty() {
        // launch() returns !
        mipsy_interactive::launch();
    }

    let files = opts.files.into_iter()
            .map(|name| {
                let file_contents = match fs::read_to_string(&name) {
                    Ok(contents) => contents,
                    Err(err) => {
                        prompt::error_nl(format!("failed to read file `{}`: {}", name.bold(), err.to_string().bright_red()));
            
                        process::exit(1);
                    },
                };

                (name, file_contents)
            })
            .collect::<HashMap<_, _>>();

    let (iset, binary, mut runtime) = match compile(&files) {
        Ok((iset, binary, runtime)) => (iset, binary, runtime),

        Err(MipsyError::Parser(error)) => {
            prompt::error(format!("failed to parse `{}`", error.file_tag()));

            let file_tag = error.file_tag();

            let file = files
                .get(&*file_tag)
                .map(|str| Rc::from(&**str))
                .expect("for file to throw a parser error, it should probably exist");

            error.show_error(file);

            process::exit(1);
        }

        Err(MipsyError::Compiler(error)) => {
            prompt::error(format!("failed to compile `{}`", error.file_tag()));

            let file_tag = error.file_tag();

            let file = files
                .get(&*file_tag)
                .map(|str| Rc::from(&**str))
                .expect("for file to throw a compiler error, it should probably exist");

            error.show_error(file);

            process::exit(1);
        }

        // unreachable: a bit tricky to get a runtime error at compile-time
        Err(MipsyError::Runtime(_)) => unreachable!(),
    };

    if opts.compile {
        let decompiled = mipsy_lib::decompile(&iset, &binary);
        println!("Compiled program:\n{}\n", decompiled);

        return;
    }

    if opts.hex {
        for &opcode in binary.text.iter() {
            if opts.hex_pad_zero {
                println!("{:08x}", opcode);
            } else {
                println!("{:x}", opcode);
            }
        }

        return;
    }

    loop {
        let mut handler = Handler;

        match runtime.step(&mut handler) {
            Ok(_) => {}
            
            Err(MipsyError::Runtime(error)) => {
                error.show_error(
                    ErrorContext::Binary,
                    files.iter()
                        .map(|(tag, content)| (Rc::from(&**tag), Rc::from(&**content)))
                        .collect(),
                    &iset,
                    &binary,
                    &runtime
                );

                process::exit(1);
            }

            // unreachable: the only possible error at runtime is a MipsyError::Runtime
            _ => unreachable!(),
        }
    }
}

fn compile(files: &HashMap<String, String>) -> MipsyResult<(InstSet, Binary, Runtime)> {
    let files = files.iter()
            .map(|(k, v)| (Some(&**k), &**v))
            .collect::<Vec<_>>();

    let iset    = mipsy_lib::inst_set();
    let binary   = mipsy_lib::compile(&iset, files)?;
    let runtime = mipsy_lib::runtime(&binary);

    Ok((iset, binary, runtime))
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));
