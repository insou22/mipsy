use std::{io::Write, fmt::{Debug, Display}, fs, process, rc::Rc, str::FromStr};

use colored::Colorize;
use mipsy_lib::{Binary, InstSet, MipsyError, MipsyResult, MpProgram, Runtime, Safe, compile::{get_kernel, CompilerOptions}};
use mipsy_lib::runtime::{SYS13_OPEN, SYS14_READ, SYS15_WRITE, SYS16_CLOSE};
use mipsy_lib::error::runtime::{Error, RuntimeError, ErrorContext, InvalidSyscallReason};
use mipsy_interactive::prompt;
use clap::Parser;
use mipsy_parser::TaggedFile;
use mipsy_utils::{MipsyConfig, MipsyConfigError, config_path, read_config};
use text_io::try_read;

#[derive(Parser, Debug)]
#[clap(version = VERSION, author = "Zac K. <zac.kologlu@gmail.com>")]
struct Opts {
    /// Just output compilation errors, if any
    #[clap(long)]
    check: bool,

    /// Implies --check: Ignore missing main label
    #[clap(long)]
    check_no_main: bool,

    /// Just compile program instead of executing
    #[clap(long)]
    compile: bool,

    /// Just compile program and output hexcodes
    #[clap(long)]
    hex: bool,

    /// Implies --hex: pad to 8 hex digits with zeroes
    #[clap(long)]
    hex_pad_zero: bool,

    /// Enable some SPIM compatibility options
    #[clap(long)]
    spim: bool,

    /// Move a label to point to a different label
    #[clap(long)]
    move_label: Vec<String>,

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
        let result: Result<i128, _> = try_read!();

        match result {
            Ok(n) => {
                match i32::try_from(n) {
                    Ok(n) => return Some(n),
                    Err(_) => {
                        println!("[mipsy] bad input (too big to fit in 32 bits)");
                        println!("[mipsy] if you want the value to be truncated to 32 bits, try {}", n as i32);
                        print!(  "[mipsy] try again: ");
                        std::io::stdout().flush().unwrap();
                        continue;
                    }
                }
            },
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

    let moves = {
        opts.move_label.into_iter()
            .map(|s| {
                let (old, new) = match s.split_once('=') {
                    Some(parts) => parts,
                    None => {
                        eprintln!("Invalid move label: {s}");
                        eprintln!("Must be in format: --move-label old1=new1 --move-label old2=new2 ...");
                        std::process::exit(1);
                    },
                };

                (old.to_string(), new.to_string())
            })
            .collect::<Vec<_>>()
        };

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
            .map(|mut name| {
                #[cfg(unix)]
                if name == "-" {
                    name = String::from("/dev/stdin");
                }

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

    let compiler_options = CompilerOptions::new(moves);

    let compiled = if opts.check_no_main {
        compile_with_kernel(&compiler_options, &config, &files, &args, &mut MpProgram::new(vec![], vec![]))
    } else {
        compile(&compiler_options, &config, &files, &args)
    };

    let (iset, binary, mut runtime) = match compiled {
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

    if opts.check || opts.check_no_main {
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
                                let character: char = get_input_eof("character").unwrap_or('\0');
                                runtime = guard(character as u8);
                            }
                            Open(_args, guard) => {
                                // TODO: implement file open for mipsy cli frontend
                                runtime = guard(-1);
                                runtime.timeline_mut().pop_last_state();
                                println!();
                                RuntimeError::new(Error::InvalidSyscall { syscall: SYS13_OPEN, reason: InvalidSyscallReason::Unimplemented }).show_error(
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
                            Read(_args, guard) => {
                                // TODO: implement file read for mipsy cli frontend
                                runtime = guard((-1, Vec::new()));
                                runtime.timeline_mut().pop_last_state();
                                println!();
                                RuntimeError::new(Error::InvalidSyscall { syscall: SYS14_READ, reason: InvalidSyscallReason::Unimplemented }).show_error(
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
                            Write(_args, guard) => {
                                // TODO: implement file write for mipsy cli frontend
                                runtime = guard(-1);
                                runtime.timeline_mut().pop_last_state();
                                println!();
                                RuntimeError::new(Error::InvalidSyscall { syscall: SYS15_WRITE, reason: InvalidSyscallReason::Unimplemented }).show_error(
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
                            Close(_args, guard) => {
                                // TODO: implement file close for mipsy cli frontend
                                runtime = guard(-1);
                                runtime.timeline_mut().pop_last_state();
                                println!();
                                RuntimeError::new(Error::InvalidSyscall { syscall: SYS16_CLOSE, reason: InvalidSyscallReason::Unimplemented }).show_error(
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

fn compile(options: &CompilerOptions, config: &MipsyConfig, files: &[(String, String)], args: &[&str]) -> MipsyResult<(InstSet, Binary, Runtime)> {
    compile_with_kernel(options, config, files, args, &mut get_kernel())
}

fn compile_with_kernel(options: &CompilerOptions, config: &MipsyConfig, files: &[(String, String)], args: &[&str], kernel: &mut MpProgram) -> MipsyResult<(InstSet, Binary, Runtime)> {
    let files = files.iter()
        .map(|(k, v)| TaggedFile::new(Some(k), v))
        .collect::<Vec<_>>();

    let iset    = mipsy_instructions::inst_set();
    let binary  = mipsy_lib::compile_with_kernel(&iset, files, kernel, options, &config)?;
    let runtime = mipsy_lib::runtime(&binary, args);

    Ok((iset, binary, runtime))
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));
