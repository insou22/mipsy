use std::{fmt::{Debug, Display}, fs, process, rc::Rc, str::FromStr};
use std::io::Write;

use colored::Colorize;
use mipsy_codegen::instruction_set;
use mipsy_lib::{Binary, InstSet, MipsyError, MipsyResult, Runtime, Safe, error::runtime::ErrorContext};
use mipsy_interactive::prompt;
use clap::{Clap, AppSettings};
use mipsy_parser::TaggedFile;
use mipsy_utils::{MipsyConfig, MipsyConfigError, config_path, read_config};
use text_io::try_read;

#[derive(Clap, Debug)]
#[clap(version = VERSION, author = "Zac K. <zac.kologlu@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(long, about("Just output compilation errors, if any"))]
    check: bool,
    #[clap(long, about("Just compile program instead of executing"))]
    compile: bool,
    #[clap(long, about("Just compile program and output hexcodes"))]
    hex: bool,
    #[clap(long, about("Implies --hex: pad to 8 hex digits with zeroes"))]
    hex_pad_zero: bool,
    #[clap(long, about("Enable some SPIM compatibility options"))]
    spim: bool,
    #[clap(long, short('v'))]
    version: bool,
    files: Vec<String>,
    #[clap(last = true)]
    args:  Vec<String>,
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

fn get_input_eof<T>(name: &str) -> Option<T>
where
    T: FromStr + Display,
    <T as FromStr>::Err: Debug,
{
    loop {
        let result: Result<T, _> = try_read!();

        match result {
            Ok(n) => return Some(n),
            Err(text_io::Error::Parse(leftover, _)) => {
                if leftover == "" {
                    return None;
                }

                print!("[mipsy] bad input (expected {}), try again: ", name);
                std::io::stdout().flush().unwrap();
                continue;
            }
            Err(_) => {
                print!("[mipsy] bad input (expected {}), try again: ", name);
                std::io::stdout().flush().unwrap();
                continue;
            },
        };
    }
}

fn get_input_int(name: &str) -> Option<i32> {
    loop {
        let result: Result<i64, _> = try_read!();

        match result {
            Ok(n) => return Some(n as i32),
            Err(text_io::Error::Parse(leftover, _)) => {
                if leftover == "" {
                    return None;
                }

                print!("[mipsy] bad input (expected {}), try again: ", name);
                std::io::stdout().flush().unwrap();
                continue;
            }
            Err(_) => {
                print!("[mipsy] bad input (expected {}), try again: ", name);
                std::io::stdout().flush().unwrap();
                continue;
            },
        };
    }
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut config = match read_config() {
        Ok(config) => config,
        Err(MipsyConfigError::InvalidConfig(to_path, config)) => {
            let config_path = match config_path() {
                Some(path) => path.to_string_lossy().to_string(),
                None => String::from("mipsy config"),
            };

            let warning = format!(
                "your {} file failed to parse. it has been moved to {}, and you have been generated a new config",
                config_path,
                to_path.to_string_lossy()
            );

            prompt::warning_nl(warning);
            
            config
        }
    };

    if opts.spim {
        config.spim = true;
    }

    if opts.files.is_empty() {
        // launch() returns !
        mipsy_interactive::launch(config);
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
            .collect::<Vec<_>>();
    
    let args = opts.args.iter()
            .map(|arg| &**arg)
            .collect::<Vec<_>>();

    let (iset, binary, mut runtime) = match compile(&config, &files, &args) {
        Ok((iset, binary, runtime)) => (iset, binary, runtime),

        Err(MipsyError::Parser(error)) => {
            prompt::error(format!("failed to parse `{}`", error.file_tag()));

            let file_tag = error.file_tag();

            let file = files
                .iter()
                .filter(|(tag, _)| &**tag == &*file_tag)
                .next()
                .map(|(_, str)| Rc::from(&**str))
                .expect("for file to throw a parser error, it should probably exist");

            error.show_error(&config, file);

            process::exit(1);
        }

        Err(MipsyError::Compiler(error)) => {
            let compile_tag = if error.file_tag().is_empty() {
                String::new()
            } else {
                format!(" `{}`", error.file_tag())
            };

            prompt::error(format!("failed to compile{}", compile_tag));

            let file_tag = error.file_tag();

            let file = files
                .iter()
                .filter(|(tag, _)| &**tag == &*file_tag)
                .next()
                .map(|(_, str)| Rc::from(&**str))
                .unwrap_or_else(|| Rc::from(""));

            error.show_error(&config, file);

            process::exit(1);
        }

        // unreachable: a bit tricky to get a runtime error at compile-time
        Err(MipsyError::Runtime(_)) => unreachable!(),
    };

    if opts.check {
        return;
    }

    if opts.compile {
        let decompiled = mipsy_lib::decompile(&iset, &binary);
        println!("Compiled program:\n{}\n", decompiled);

        return;
    }

    if opts.hex || opts.hex_pad_zero {
        for opcode in binary.text_words() {
            if let Safe::Valid(opcode) = opcode {
                if opts.hex_pad_zero {
                    println!("{:08x}", opcode);
                } else {
                    println!("{:x}", opcode);
                }
            } else {
                println!("uninitialized");
            }
        }

        return;
    }

    loop {
        match runtime.step() {
            Ok(stepped_runtime) => {
                match stepped_runtime {
                    Ok(new_runtime) => {
                        runtime = new_runtime;
                    },
                    Err(runtime_guard) => {
                        use mipsy_lib::runtime::RuntimeSyscallGuard::*;

                        match runtime_guard {
                            PrintInt(args, new_runtime) => {
                                print!("{}", args.value);
                                std::io::stdout().flush().unwrap();

                                runtime = new_runtime;
                            }
                            PrintFloat(args, new_runtime) => {
                                print!("{}", args.value);
                                std::io::stdout().flush().unwrap();
                                
                                runtime = new_runtime;
                            }
                            PrintDouble(args, new_runtime) => {
                                print!("{}", args.value);
                                std::io::stdout().flush().unwrap();
                                
                                runtime = new_runtime;
                            }
                            PrintString(args, new_runtime) => {
                                print!("{}", String::from_utf8_lossy(&args.value));
                                std::io::stdout().flush().unwrap();
                                
                                runtime = new_runtime;
                            }
                            ReadInt(guard) => {
                                let number = get_input_int("int").unwrap_or(0);
                                runtime = guard(number);
                            }
                            ReadFloat(guard) => {
                                let number = get_input_eof("float").unwrap_or(0.0);
                                runtime = guard(number);
                            }
                            ReadDouble(guard) => {
                                let number = get_input_eof("double").unwrap_or(0.0);
                                runtime = guard(number);
                            }
                            ReadString(args, guard) => {
                                let string = read_string(args.max_len);
                                runtime = guard(string.into_bytes());
                            }
                            Sbrk(_args, new_runtime) => {
                                runtime = new_runtime;
                            }
                            Exit(_new_runtime) => {
                                std::process::exit(0);
                            }
                            PrintChar(args, new_runtime) => {
                                print!("{}", args.value as char);
                                std::io::stdout().flush().unwrap();
                                
                                runtime = new_runtime;
                            }
                            ReadChar(guard) => {
                                let number: char = get_input_eof("character").unwrap_or('\0');
                                runtime = guard(number as u8);
                            }
                            Open(_, _) => todo!(),
                            Read(_, _) => todo!(),
                            Write(_, _) => todo!(),
                            Close(_, _) => todo!(),
                            ExitStatus(args, _new_runtime) => {
                                std::process::exit(args.exit_code);
                            }
                            Breakpoint(new_runtime) => {
                                runtime = new_runtime;
                            }
                            Trap(new_runtime) => {
                                // TODO(zkol): What do we want to do with a trap here
                                runtime = new_runtime;
                            }
                            UnknownSyscall(args, new_runtime) => {
                                runtime = new_runtime;
                                prompt::error(format!("unknown syscall: {}", args.syscall_number));
                            }
                        }
                    }
                }
            }
            Err((old_runtime, MipsyError::Runtime(err))) => {
                runtime = old_runtime;

                println!();
                err.show_error(
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
            Err((_, MipsyError::Parser(_) | MipsyError::Compiler(_))) => {
                unreachable!("the only possible error at runtime is a MipsyError::Runtime");
            }
        }
    }
}

fn read_string(_max_len: u32) -> String {
    loop {
        let input: String = get_input("string", true);

        // if input.len() > max_len as usize {
        //     println!("[mipsy] bad input (max string length specified as {}, given string is {} bytes)", max_len, input.len());
        //     print!  ("[mipsy] please try again: ");
        //     std::io::stdout().flush().unwrap();

        //     continue;
        // }

        // if input.len() == max_len as usize {
        //     println!("[mipsy] bad input (max string length specified as {}, given string is {} bytes -- must be at least one byte fewer, for NULL character), try again: ", max_len, input.len());
        //     print!  ("[mipsy] please try again: ");
        //     std::io::stdout().flush().unwrap();

        //     continue;
        // }

        return input;
    }
}

fn compile(config: &MipsyConfig, files: &[(String, String)], args: &[&str]) -> MipsyResult<(InstSet, Binary, Runtime)> {
    let files = files.iter()
        .map(|(k, v)| TaggedFile::new(Some(k), v))
        .collect::<Vec<_>>();

    let iset    = mipsy_instructions::inst_set();
    let binary  = mipsy_lib::compile(&iset, files, &config)?;
    let runtime = mipsy_lib::runtime(&binary, args);

    Ok((iset, binary, runtime))
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));
