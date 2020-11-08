mod helper;
mod commands;
mod prompt;
mod error;
mod runtime_handler;

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
use mipsy_lib::{
    Binary, 
    InstSet, 
    Runtime,
};
use commands::Command;
use runtime_handler::Handler;

use self::error::{CommandError, CommandResult};

pub(crate) struct State {
    pub(crate) iset: InstSet,
    pub(crate) commands: Vec<Command>,
    pub(crate) binary:  Option<Binary>,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) exited: bool,
    pub(crate) prev_command: Option<String>,
    pub(crate) confirm_exit: bool,
}

impl State {
    fn new() -> Self {
        Self {
            iset: mipsy_lib::inst_set().unwrap(),
            commands: vec![],
            binary:  None,
            runtime: None,
            exited: false,
            prev_command: None,
            confirm_exit: false,
        }
    }

    fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    fn prompt(&self) -> &str {
        if self.confirm_exit {
            ""
        } else {
            "[mipsy] "
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

    fn find_command(&self, cmd: &str) -> Option<&Command> {
        self.commands.iter()
                .find(|command| {
                    command.name == cmd || command.aliases.iter().find(|&alias| alias == cmd).is_some()
                })
    }

    fn do_exec(&mut self, line: &String) {
        let parts = match shlex::split(line) {
            Some(parts) => parts,
            None => return,
        };

        let command_name = match parts.first() {
            Some(command_name) => command_name,
            None => return,
        };

        let command = self.find_command(&command_name.to_ascii_lowercase());

        if command.is_none() {
            prompt::unknown_command(command_name);
            return;
        }

        let command = command.unwrap();
        if (parts.len() - 1) < command.required_args.len() {
            let mut err_msg = String::from("missing required parameter");

            if command.required_args.len() - (parts.len() - 1) > 1 {
                err_msg.push('s');
            }

            err_msg.push(' ');

            err_msg.push_str(
                &command.required_args[(parts.len() - 1)..(command.required_args.len())]
                    .iter()
                    .map(|s| format!("{}{}{}", "<".magenta(), s.magenta(), ">".magenta()))
                    .collect::<Vec<String>>()
                    .join(" ")
            );

            prompt::error(err_msg);
            prompt::tip_nl(format!("try `{} {}`", "help".bold(), command_name.bold()));
            return;
        }

        let result = (command.exec)(self, command_name, &parts[1..]);
        match result {
            Ok(_)    => {}
            Err(err) => self.handle_error(err, true),
        };
    }

    fn handle_error(&self, err: CommandError, nl: bool) {
        match err {
            CommandError::BadArgument { arg, instead, } => {
                prompt::error(
                    format!("bad argument `{}` for {}", instead, arg)
                )
            }
            CommandError::ArgExpectedI32 { arg, instead, } => {
                prompt::error(
                    format!("parameter {} expected integer, got `{}` instead", arg, instead)
                );
            }
            CommandError::ArgExpectedU32 { arg, instead, } => {
                prompt::error(
                    format!("parameter {} expected positive integer, got `{}` instead", arg, instead)
                );
            }
            CommandError::HelpUnknownCommand { command, } => {
                prompt::error(format!("unknown command `{}`", command));
            }
            CommandError::CannotReadFile { path, os_error, } => {
                prompt::error(format!("failed to read file `{}`: {}", path, os_error));
            }
            CommandError::CannotCompile  { path, program: _, mipsy_error, } => {
                prompt::error(format!("failed to compile `{}` -- {:?}", path, mipsy_error));
            }
            CommandError::UnknownRegister { register } => {
                prompt::error(format!("unknown register: {}{}", "$".yellow(), register.bold()));
            }
            CommandError::MustLoadFile => {
                prompt::error("you have to load a file first");
            }
            CommandError::ProgramExited => {
                prompt::error("program has exited");
                prompt::tip(format!("try using `{}` or `{}`", "back".bold(), "reset".bold()));
            }
            CommandError::CannotStepFurtherBack => {
                prompt::error("can't step any further back")
            }
            CommandError::RuntimeError { mipsy_error, } => {
                prompt::error(format!("runtime error -- {:?}", mipsy_error));
            }
            CommandError::WithTip { error, tip, } => {
                self.handle_error(*error, false);
                prompt::tip(tip);
            }
            CommandError::UnknownLabel { label } => {
                prompt::error(format!("unknown label: \"{}\"", label));
            }
            CommandError::UninitialisedPrint { addr } => {
                prompt::error(format!("memory at address 0x{:08x} is uninitialized", addr));
            }
            CommandError::UnterminatedString { good_parts } => {
                prompt::error(format!("unterminated string: \"{}\"", good_parts.red()));
                prompt::tip(format!("make sure your strings are null terminated - use {} instead of {}", ".asciiz".green(), ".ascii".red()));
            }
        }

        if nl {
            println!();
        }
    }

    pub(crate) fn step(&mut self, verbose: bool) -> CommandResult<bool> {
        let runtime = self.runtime.as_mut().ok_or(CommandError::MustLoadFile)?;

        let mut handler = Handler::make(verbose);
        runtime.step(&mut handler).map_err(|err| CommandError::RuntimeError { mipsy_error: err })?;

        Ok(
            if handler.exit_status.is_some() {
                self.exited = true;
                true
            } else {
                let pc = runtime.state().get_pc();

                let binary = self.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

                if handler.breakpoint || binary.breakpoints.contains(&pc) {
                    let label = binary.labels.iter()
                            .find(|(_, &addr)| addr == pc)
                            .map(|(name, _)| name.yellow().bold().to_string());

                    println!(
                        "{}{}{}\n", 
                        "\n[BREAKPOINT ".cyan().bold(), 
                        label.unwrap_or(format!("{}{:08x}", "0x".yellow(), pc)), 
                        "]".cyan().bold()
                    );
                    true
                } else {
                    false
                }
            }
        )
    }

    pub(crate) fn run(&mut self) -> CommandResult<()> {
        if self.exited {
            return Err(CommandError::ProgramExited);
        }

        loop {
            if self.step(false)? {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn reset(&mut self) -> CommandResult<()> {
        let runtime = self.runtime.as_mut().ok_or(CommandError::MustLoadFile)?;
        runtime.reset();
        self.exited = false;

        Ok(())
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

fn state() -> State {
    let mut state = State::new();

    state.add_command(commands::load_command());
    state.add_command(commands::run_command());
    state.add_command(commands::step_command());
    state.add_command(commands::back_command());
    state.add_command(commands::step2syscall_command());
    state.add_command(commands::step2input_command());
    state.add_command(commands::reset_command());
    state.add_command(commands::breakpoint_command());
    state.add_command(commands::breakpoints_command());
    state.add_command(commands::decompile_command());
    state.add_command(commands::label_command());
    state.add_command(commands::labels_command());
    state.add_command(commands::print_command());
    state.add_command(commands::help_command());
    state.add_command(commands::exit_command());

    state
}

pub(crate) fn launch() -> ! {
    let mut rl = editor();
    let mut state = state();

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