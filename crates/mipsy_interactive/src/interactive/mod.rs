pub mod prompt;
pub(crate) mod commands;
mod helper;
mod error;
mod runtime_handler;

use std::collections::HashMap;
use std::{ops::Deref, rc::Rc};

use mipsy_lib::{MipsyError, ParserError, error::parser, runtime::{SteppedRuntime, state::TIMELINE_MAX_LEN}};
use mipsy_lib::runtime::{SYS13_OPEN, SYS14_READ, SYS15_WRITE, SYS16_CLOSE};
use mipsy_lib::error::runtime::{Error, RuntimeError, ErrorContext, InvalidSyscallReason};
use helper::MyHelper;

use rustyline::{
    At,
    Cmd,
    Editor,
    KeyCode,
    KeyEvent,
    Modifiers,
    Movement,
    Word,
    error::ReadlineError,
};
use colored::*;
use mipsy_lib::{
    Binary, 
    InstSet, 
    Runtime,
};
use commands::{
    Command,
    Arguments,
};

use mipsy_utils::MipsyConfig;

use self::error::{CommandError, CommandResult};

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone, Default)]
pub(crate) struct Breakpoint {
    id: u32,
    enabled: bool,
}

impl Breakpoint {
    fn new(id: u32) -> Self {
        Self {
            id,
            enabled: true,
        }
    }
}

pub(crate) struct State {
    pub(crate) config: MipsyConfig,
    pub(crate) iset: InstSet,
    pub(crate) commands: Vec<Command>,
    pub(crate) program: Option<Vec<(String, String)>>,
    pub(crate) binary:  Option<Binary>,
    pub(crate) breakpoints: HashMap<u32, Breakpoint>,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) exited: bool,
    pub(crate) prev_command: Option<String>,
    pub(crate) confirm_exit: bool,
}

impl State {
    fn new(config: MipsyConfig) -> Self {
        Self {
            config,
            iset: mipsy_instructions::inst_set(),
            commands: vec![],
            program: None,
            binary:  None,
            breakpoints: HashMap::new(),
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

    fn cleanup_cmd(&mut self, cmd: String) {
        self.confirm_exit = false;
        self.prev_command = Some(cmd);
    }

    fn find_command(&self, cmd: &str) -> Option<&Command> {
        self.commands.iter()
                .find(|command| {
                    command.name == cmd || 
                        command.aliases.iter()
                                .any(|alias| alias == cmd)
                })
    }

    fn do_exec(&mut self, line: &str) {
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
        let required = match &command.args {
            Arguments::Exactly { required, optional: _ } => required,
            Arguments::VarArgs { required, format:   _ } => required,
        };

        if (parts.len() - 1) < required.len() {
            self.handle_error(CommandError::WithTip {
                error: Box::new(CommandError::MissingArguments {
                    args: required.to_vec(),
                    instead: parts.to_vec(),
                }),
                tip: format!("try `{} {}`", "help".bold(), command_name.bold()),
            }, true);
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
            CommandError::MissingArguments { args, instead } => {
                let mut err_msg = String::from("missing required parameter");

                if args.len() - (instead.len().saturating_sub(1)) > 1 {
                    err_msg.push('s');
                }

                err_msg.push(' ');

                err_msg.push_str(
                    &args[(instead.len().saturating_sub(1))..(args.len())]
                        .iter()
                        .map(|s| format!("{}{}{}", "<".magenta(), s.magenta(), ">".magenta()))
                        .collect::<Vec<String>>()
                        .join(" ")
                );

                prompt::error(err_msg);
            }
            CommandError::BadArgument { arg, instead, } => {
                prompt::error(
                    format!("bad argument `{}` for {}", instead, arg)
                );
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
            CommandError::InvalidBpId { arg } => {
                prompt::error(format!("breakpoint with id {} does not exist", arg.blue()));
            },
            CommandError::HelpUnknownCommand { command, } => {
                prompt::error(format!("unknown command `{}`", command));
            }
            CommandError::CannotReadFile { path, os_error, } => {
                prompt::error(format!("failed to read file `{}`: {}", path, os_error));
            }
            CommandError::CannotCompile  { mipsy_error } => {
                let file_tag = match mipsy_error {
                    MipsyError::Parser(ref error) => error.file_tag(),
                    MipsyError::Compiler(ref error) => error.file_tag(),
                    // unreachable: can't have a runtime error at compile time (hopefully)
                    MipsyError::Runtime(_) => unreachable!(),
                };

                let file_prompt = {
                    if file_tag.is_empty() {
                        String::new()
                    } else {
                        format!("`{}`", file_tag)
                    }
                };

                prompt::error(format!("failed to compile {}", file_prompt));
                self.mipsy_error(mipsy_error, ErrorContext::Interactive, None);
            }
            CommandError::CannotParseLine { line, error } => {
                prompt::error("failed to parse");

                self.mipsy_error(
                    MipsyError::Parser(
                        ParserError::new(
                            parser::Error::ParseFailure,
                            Rc::from(""),
                            error.line,
                            error.col as u32,
                        )
                    ),
                    ErrorContext::Repl,
                    Some(line),
                );
            }
            CommandError::CannotCompileLine { line, error } => {
                prompt::error("failed to compile instruction");
                self.mipsy_error(error, ErrorContext::Repl, Some(line));
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
            CommandError::RanOutOfHistory => {
                prompt::error(format!("ran out of history (max {} steps) -- try using `{}`", TIMELINE_MAX_LEN, "reset".bold()))
            }
            CommandError::RuntimeError { mipsy_error, } => {
                self.mipsy_error(mipsy_error, ErrorContext::Interactive, None);
            }
            CommandError::ReplRuntimeError { mipsy_error, line, } => {
                self.mipsy_error(mipsy_error, ErrorContext::Repl, Some(line));
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

    pub(crate) fn mipsy_error(&self, error: MipsyError, context: ErrorContext, repl_line: Option<String>) {
        let config = &self.config;

        match error {
            MipsyError::Parser(error) => {
                if let Some(line) = repl_line {
                    error.show_error(config, Rc::from(&*line));
                } else {
                    let file_tag = error.file_tag();

                    let file = self.program.as_ref()
                        .expect("cannot get parser error without a file to compile")
                        .iter()
                        .find(|(tag, _)| tag.as_str() == file_tag.deref())
                        .map(|(_, str)| Rc::from(&**str))
                        .expect("for file to throw a parser error, it should probably exist");

                    error.show_error(config, file);
                }
            }
            MipsyError::Compiler(error) => {
                if let Some(line) = repl_line {
                    error.show_error(config, Rc::from(&*line));
                } else {
                    let file_tag = error.file_tag();
    
                    let file = self.program.as_ref()
                        .expect("cannot get compiler error without a file to compile")
                        .iter()
                        .find(|(tag, _)| tag.as_str() == file_tag.deref())
                        .map(|(_, str)| Rc::from(&**str))
                        .unwrap_or_else(|| Rc::from(""));
    
                    error.show_error(config, file);
                }
            }
            MipsyError::Runtime(error) => {
                error.show_error(
                    context,
                    if let Some(line) = repl_line {
                        vec![(Rc::from(""), Rc::from(&*line))]
                    } else {
                        self.program.as_ref().unwrap().iter()
                            .map(|(tag, content)| (Rc::from(&**tag), Rc::from(&**content)))
                            .collect()
                    },
                    &self.iset,
                    self.binary.as_ref().unwrap(),
                    self.runtime.as_ref().unwrap(),
                )
            }
        }
    }

    pub(crate) fn eval_stepped_runtime(&mut self, verbose: bool, result: Result<SteppedRuntime, (Runtime, MipsyError)>) -> CommandResult<bool> {
        let mut breakpoint = false;
        let mut trapped = false;

        match result {
            Ok(Ok(new_runtime)) => {
                self.runtime = Some(new_runtime);
            }
            Ok(Err(guard)) => {
                // Ok(true) on exit or breakpoint, see self::exec_status
                use mipsy_lib::runtime::RuntimeSyscallGuard::*;

                match guard {
                    PrintInt(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::sys1_print_int(verbose, args.value);
                    }
                    PrintFloat(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::sys2_print_float(verbose, args.value);
                    }
                    PrintDouble(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::sys3_print_double(verbose, args.value);
                    }
                    PrintString(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::sys4_print_string(verbose, &args.value);
                    }
                    ReadInt(guard) => {
                        let value = runtime_handler::sys5_read_int(verbose);
                        self.runtime = Some(guard(value));
                    }
                    ReadFloat(guard) => {
                        let value = runtime_handler::sys6_read_float(verbose);
                        self.runtime = Some(guard(value));
                    }
                    ReadDouble(guard) => {
                        let value = runtime_handler::sys7_read_double(verbose);
                        self.runtime = Some(guard(value));
                    }
                    ReadString(args, guard) => {
                        let value = runtime_handler::sys8_read_string(verbose, args.max_len);
                        self.runtime = Some(guard(value));
                    }
                    Sbrk(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::sys9_sbrk(verbose, args.bytes);
                    }
                    Exit(new_runtime) => {
                        self.runtime = Some(new_runtime);
                        self.exited = true;
                        
                        runtime_handler::sys10_exit(verbose);
                    }
                    PrintChar(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::sys11_print_char(verbose, args.value);
                    }
                    ReadChar(guard) => {
                        let value = runtime_handler::sys12_read_char(verbose);
                        self.runtime = Some(guard(value));
                    }
                    Open(_args, guard) => {
                        // TODO: implement file open for mipsy interactive frontend

                        let mut new_runtime = guard(-1);
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = Some(new_runtime);
                        return Err(CommandError::RuntimeError { mipsy_error: MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall: SYS13_OPEN, reason: InvalidSyscallReason::Unimplemented }))});

                        // let value = runtime_handler::sys13_open(verbose, args);
                        // self.runtime = Some(guard(value));
                    }
                    Read(_args, guard) => {
                        // TODO: implement file read for mipsy interactive frontend

                        let mut new_runtime = guard((-1, Vec::new()));
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = Some(new_runtime);
                        return Err(CommandError::RuntimeError { mipsy_error: MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall: SYS14_READ, reason: InvalidSyscallReason::Unimplemented }))});

                        // let value = runtime_handler::sys14_read(verbose, args);
                        // self.runtime = Some(guard(value));
                    }
                    Write(_args, guard) => {
                        // TODO: implement file write for mipsy interactive frontend

                        let mut new_runtime = guard(-1);
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = Some(new_runtime);
                        return Err(CommandError::RuntimeError { mipsy_error: MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall: SYS15_WRITE, reason: InvalidSyscallReason::Unimplemented }))});

                        // let value = runtime_handler::sys15_write(verbose, args);
                        // self.runtime = Some(guard(value));
                    }
                    Close(_args, guard) => {
                        // TODO: implement file close for mipsy interactive frontend

                        let mut new_runtime = guard(-1);
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = Some(new_runtime);
                        return Err(CommandError::RuntimeError { mipsy_error: MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall: SYS16_CLOSE, reason: InvalidSyscallReason::Unimplemented }))});

                        // let value = runtime_handler::sys16_close(verbose, args);
                        // self.runtime = Some(guard(value));
                    }
                    ExitStatus(args, new_runtime) => {
                        self.runtime = Some(new_runtime);
                        self.exited = true;

                        runtime_handler::sys17_exit_status(verbose, args.exit_code);
                    }
                    Breakpoint(new_runtime) => {
                        self.runtime = Some(new_runtime);
                        breakpoint = true;
                    }
                    Trap(new_runtime) => {
                        self.runtime = Some(new_runtime);
                        runtime_handler::trap(verbose);
                        trapped = true;
                    }
                }
            }
            Err((new_runtime, err)) => {
                self.runtime = Some(new_runtime);

                return Err(CommandError::RuntimeError { mipsy_error: err });
            }
        };

        Ok(
            if self.exited {
                true
            } else {
                let pc = self.runtime.as_ref().unwrap().timeline().state().pc();
                let binary = self.binary.as_ref().unwrap();

                // TODO(joshh): make this less ugly when
                // https://github.com/rust-lang/rust/pull/94927 is released
                // if let Some(bp) = self.breakpoints.get(&pc) || breakpoint {
                //     if !bp.enabled { trapped }

                let bp = self.breakpoints.get(&pc).cloned().unwrap_or_default();
                if breakpoint || bp.enabled {
                    let label = binary.labels.iter()
                            .find(|(_, &addr)| addr == pc)
                            .map(|(name, _)| name.yellow().bold().to_string());
                    
                    runtime_handler::breakpoint(label.as_deref(), pc);

                    true

                // todo(joshh): add watchpoints
                } else {
                    trapped
                }
            }
        )
    }

    pub(crate) fn step(&mut self, verbose: bool) -> CommandResult<bool> {
        let runtime = self.runtime.take().ok_or(CommandError::MustLoadFile)?;

        self.eval_stepped_runtime(verbose, runtime.step())
    }

    pub(crate) fn exec_inst(&mut self, opcode: u32, verbose: bool) -> CommandResult<bool> {
        let runtime = self.runtime.take().ok_or(CommandError::MustLoadFile)?;

        self.eval_stepped_runtime(verbose, runtime.exec_inst(opcode))
    }

    pub(crate) fn run(&mut self) -> CommandResult<String> {
        if self.exited {
            return Err(CommandError::ProgramExited);
        }

        loop {
            if self.step(false)? {
                break;
            }
        }

        Ok("".into())
    }

    pub(crate) fn reset(&mut self) -> CommandResult<()> {
        let runtime = self.runtime.as_mut().ok_or(CommandError::MustLoadFile)?;
        runtime.timeline_mut().reset();
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

    fn generate_breakpoint_id(&self) -> u32 {
        // TODO(joshh): reuses old breakpoint ids
        // this diverges from gdb behaviour but is it a problem?
        let mut id = self.breakpoints
                    .values()
                    .map(|bp| bp.id)
                    .fold(std::u32::MIN, |x, y| x.max(y))
                    .wrapping_add(1);

        if self.breakpoints.values().any(|bp| bp.id == id) {
            // find a free id to use
            // there's probably a neater way to do this,
            // but realistically if someone is using enough breakpoints
            // to fill a u32, they have bigger problems

            let mut ids = self.breakpoints
                    .values()
                    .map(|bp| bp.id)
                    .collect::<Vec<_>>();

            ids.sort_unstable();

            id = ids.into_iter()
                    .enumerate()
                    .find(|x| x.0 != x.1 as usize)
                    .expect("you've run out of breakpoints! why are you using so many")
                    .1;
        }

        id
    }
}

fn editor() -> Editor<MyHelper> {
    let mut rl = Editor::new();

    let helper = MyHelper::new();
    rl.set_helper(Some(helper));

    rl.bind_sequence(KeyEvent(KeyCode::Left, Modifiers::CTRL),  Cmd::Move(Movement::BackwardWord(1, Word::Emacs)));
    rl.bind_sequence(KeyEvent(KeyCode::Right, Modifiers::CTRL), Cmd::Move(Movement::ForwardWord (1, At::BeforeEnd, Word::Emacs)));

    rl
}

fn state(config: MipsyConfig) -> State {
    let mut state = State::new(config);

    state.add_command(commands::load_command());
    state.add_command(commands::run_command());
    state.add_command(commands::step_command());
    state.add_command(commands::back_command());
    state.add_command(commands::step2syscall_command());
    state.add_command(commands::step2input_command());
    state.add_command(commands::reset_command());
    state.add_command(commands::breakpoint_command());
    state.add_command(commands::disassemble_command());
    state.add_command(commands::context_command());
    state.add_command(commands::label_command());
    state.add_command(commands::labels_command());
    state.add_command(commands::print_command());
    state.add_command(commands::dot_command());
    state.add_command(commands::help_command());
    state.add_command(commands::exit_command());

    state
}

pub fn launch(config: MipsyConfig) -> ! {
    let mut rl = editor();
    let mut state = state(config);

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
            Err(ReadlineError::Interrupted) => {}
            Err(ReadlineError::Eof) => {
                std::process::exit(0);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    std::process::exit(0)
}
