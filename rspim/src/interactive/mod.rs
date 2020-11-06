mod helper;
use std::fmt::Display;

use helper::MyHelper;

use rustyline::{
    Editor,
    KeyPress,
    Cmd,
    Movement,
    At,
    Word,
    error::ReadlineError,
};
use colored::*;
use rspim_lib::{
    Binary, 
    InstSet, 
    Runtime,
    decompile::decompile_inst_into_parts,
};

struct State {
    iset: InstSet,
    binary:  Option<Binary>,
    runtime: Option<Runtime>,
    prev_command: Option<String>,
    confirm_exit: bool,
}

impl State {
    fn new() -> Self {
        Self {
            iset: rspim_lib::inst_set().unwrap(),
            binary:  None,
            runtime: None,
            prev_command: None,
            confirm_exit: false,
        }
    }

    fn prompt(&self) -> &str {
        if self.confirm_exit {
            ""
        } else {
            "[rspim] "
        }
    }

    fn int(&mut self) {
        if self.confirm_exit {
            std::process::exit(0);
        } else {
            println!("press again to confirm exit...");
            self.confirm_exit = true;
        }
    }

    fn cleanup_cmd(&mut self, cmd: String) {
        self.confirm_exit = false;
        self.prev_command = Some(cmd);
    }

    fn do_exec(&mut self, line: &String) {
        let parts = match shlex::split(line) {
            Some(parts) => parts,
            None => return,
        };

        let command = match parts.first() {
            Some(command) => command,
            None => return,
        };

        let command_lower = command.to_ascii_lowercase();

        match &*command_lower {
            "load" => {
                let path = match parts.get(1) {
                    Some(path) => path,
                    None => {
                        Self::error(format!("missing parameter {}", "<file>".magenta()));
                        Self::tip_nl(format!("try `{}`", "help load".bold()));
                        return;
                    }
                };

                let program = match std::fs::read_to_string(path) {
                    Ok(program) => program,
                    Err(err) => {
                        Self::error_nl(err);
                        return;
                    }
                };
                
                let binary = match rspim_lib::compile(&self.iset, &program) {
                    Ok(binary) => binary,
                    Err(_err) => {
                        Self::error_nl("Error: Failed to compile");
                        return;
                    }
                };

                let runtime = match rspim_lib::run(&binary) {
                    Ok(runtime) => runtime,
                    Err(_err) => {
                        Self::error_nl("Error: Failed to compile");
                        return;
                    }
                };

                self.binary  = Some(binary);
                self.runtime = Some(runtime);
                Self::success_nl("file loaded");
            }
            "step" | "back" | "run" => {
                if self.binary.is_none() || self.runtime.is_none() {
                    Self::error_nl("you have to load a file first");
                    return;
                }

                let binary  = self.binary.as_ref().unwrap();
                let runtime = self.runtime.as_ref().unwrap();

                let result = match &*command_lower {
                    "step" => {
                        if let Ok(inst) = runtime.next_inst() {
                            self.print_inst(binary, inst, runtime.state().get_pc());
                        }

                        self.runtime.as_mut().unwrap().step()
                    },
                    "back" => {
                        if !self.runtime.as_mut().unwrap().back() {
                            Self::error_nl("can't step any further back");
                            return;
                        }

                        Ok(())
                    },
                    "run" => {
                        let runtime = self.runtime.as_mut().unwrap();

                        loop {
                            match runtime.step() {
                                Ok(_) => {}
                                Err(err) => break Err(err),
                            }
                        }
                    }
                    _ => unreachable!(),
                };

                match result {
                    Ok(_) => {},
                    Err(_err) => {
                        Self::error_nl("runtime error");
                    }
                }
            }
            "exit" => {
                std::process::exit(0);
            }
            "help" => {
                match parts.get(1) {
                    Some(arg) => {
                        match &**arg {
                            "load" => {

                            }
                            "step" => {

                            }
                            "back" => {

                            }
                            "run" => {

                            }
                            "help" => {
                                println!("...is that even legal?\n");
                            }
                            _ => {
                                Self::unknown_command(arg);
                            }
                        }
                    }
                    None => {
                        println!(
                            "\n{}\n{}\n{}\n{}\n{}\n{}\n",
                            "COMMANDS:".green().bold(),
                            format!("  {} {:<10}{}", "load".yellow().bold(), "<file>".magenta(), "- load a MIPS file to run".white()),
                            format!("  {:<15}{}",    "step".yellow().bold(), "- step forwards one instruction".white()),
                            format!("  {:<15}{}",    "back".yellow().bold(), "- step backwards one instruction".white()),
                            format!("  {:<15}{}",    "exit".yellow().bold(), "- exit rspim".white()),
                            format!("  {} {:<10}{}", "help".yellow().bold(), "[command]".magenta(), "- print this help text, or specific help for a command".white()),
                        );
                    }
                }
            }
            _  => {
                Self::unknown_command(command);
            }
        }

    }

    fn print_inst(&self, binary: &Binary, inst: u32, addr: u32) {
        let parts = decompile_inst_into_parts(binary, &self.iset, inst, addr);

        if parts.inst_name.is_none() {
            return;
        }

        let name = parts.inst_name.unwrap();

        if !parts.labels.is_empty() {
            println!();
        }

        for label in parts.labels.iter() {
            Self::banner_nl(label.yellow().bold());
        }

        let args = parts.arguments
            .iter()
            .map(|arg| {
                if arg.starts_with('$') {
                    format!("{}{}", "$".yellow(), arg[1..].bold())
                } else if arg.chars().next().unwrap().is_alphabetic() {
                    arg.yellow().bold().to_string()
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        println!(
            "{} [{}]    {:6} {}",
            format!("0x{:08x}", addr).bright_black(),
            format!("0x{:08x}", parts.opcode).green(),
            name.yellow().bold(),
            args,
        );
    }

    fn unknown_command<D: Display>(command: D) {
        Self::error(format!("{} `{}`", "unknown command", command));
        Self::tip_nl(format!("{}{}{}", "use `", "help".bold(), "` for a list of commands"));
    }

    fn banner<D: Display>(text: D) {
        print!("{}{}", text, ": ".bold());
    }

    fn banner_nl<D: Display>(text: D) {
        Self::banner(text);
        println!();
    }

    fn ebanner<D: Display>(text: D) {
        eprint!("{}{}", text, ": ".bold());
    }

    fn ebanner_nl<D: Display>(text: D) {
        Self::ebanner(text);
        println!();
    }

    fn tip<D: Display>(text: D) {
        Self::banner("tip".yellow().bold());
        println!("{}", text);
    }

    fn tip_nl<D: Display>(text: D) {
        Self::tip(text);
        println!();
    }

    fn success<D: Display>(text: D) {
        Self::banner("success".green().bold());
        println!("{}", text);
    }

    fn success_nl<D: Display>(text: D) {
        Self::success(text);
        println!();
    }

    fn error<D: Display>(text: D) {
        Self::ebanner("error".red());
        eprintln!("{}", text);
    }

    fn error_nl<D: Display>(text: D) {
        Self::error(text);
        println!();
    }

    fn exec_command(&mut self, line: String) {
        self.do_exec(&line);
        self.cleanup_cmd(line);
    }

    fn exec_prev(&mut self) {
        if let Some(cmd) = self.prev_command.take() {
            self.exec_command(cmd);
        }
    }
}

fn editor() -> Editor<MyHelper> {
    let mut rl = Editor::new();

    let helper = MyHelper::new();
    rl.set_helper(Some(helper));

    rl.bind_sequence(KeyPress::ControlLeft,  Cmd::Move(Movement::BackwardWord(1, Word::Emacs)));
    rl.bind_sequence(KeyPress::ControlRight, Cmd::Move(Movement::ForwardWord (1, At::BeforeEnd, Word::Emacs)));

    rl
}

pub(crate) fn launch() -> ! {
    let mut rl = editor();
    let mut state = State::new();

    loop {
        let readline = rl.readline(state.prompt());

        match readline {
            Ok(line) => {
                if line.is_empty() {
                    if !state.confirm_exit {
                        state.exec_prev();
                    }

                    state.confirm_exit = false;
                    continue;
                }

                rl.add_history_entry(&line);
                state.exec_command(line);
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                state.int();
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    std::process::exit(0)
}