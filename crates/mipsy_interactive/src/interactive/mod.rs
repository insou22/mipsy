pub(crate) mod commands;
mod error;
mod helper;
pub mod prompt;
mod runtime_handler;

use std::{
    mem::take,
    ops::Deref,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use helper::MyHelper;
use mipsy_lib::error::runtime::{Error, ErrorContext, InvalidSyscallReason, RuntimeError};
use mipsy_lib::runtime::{SYS13_OPEN, SYS14_READ, SYS15_WRITE, SYS16_CLOSE};
use mipsy_lib::{
    compile::breakpoints::{get_affected_registers, TargetAction, TargetWatch},
    error::parser,
    runtime::{state::TIMELINE_MAX_LEN, SteppedRuntime},
    Binary, InstSet, MipsyError, ParserError, Runtime,
};

use colored::*;
use commands::{Arguments, Command};
use rustyline::{
    config::Configurer, error::ReadlineError, At, Cmd, Editor, KeyCode, KeyEvent, Modifiers,
    Movement, Word,
};

use mipsy_utils::MipsyConfig;

use self::error::{CommandError, CommandResult};

pub(crate) struct State {
    pub(crate) config: MipsyConfig,
    pub(crate) iset: InstSet,
    pub(crate) commands: Vec<Command>,
    pub(crate) program: Option<Vec<(String, String)>>,
    pub(crate) binary: Option<Binary>,
    pub(crate) runtime: Runtime,
    pub(crate) exited: bool,
    pub(crate) prev_command: Option<String>,
    pub(crate) confirm_exit: bool,
    pub(crate) interrupted: Arc<AtomicBool>,
}

impl State {
    fn new(config: MipsyConfig) -> Self {
        Self {
            config,
            iset: mipsy_instructions::inst_set(),
            commands: vec![],
            program: None,
            binary: None,
            runtime: Runtime::new_without_binary(),
            exited: false,
            prev_command: None,
            confirm_exit: false,
            interrupted: Arc::new(AtomicBool::new(false)),
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

    fn find_command(&self, cmd: &str) -> Option<Command> {
        self.commands
            .iter()
            .find(|command| command.name == cmd || command.aliases.iter().any(|alias| alias == cmd))
            .cloned()
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
            Arguments::Exactly {
                required,
                optional: _,
            } => required,
            Arguments::VarArgs {
                required,
                format: _,
            } => required,
        };

        if (parts.len() - 1) < required.len() {
            self.handle_error(
                CommandError::WithTip {
                    error: Box::new(CommandError::MissingArguments {
                        args: required.to_vec(),
                        instead: parts.to_vec(),
                    }),
                    tip: format!("try `{} {}`", "help".bold(), command_name.bold()),
                },
                true,
            );
            return;
        }

        let result = command.exec(self, command_name, &parts[1..]);
        match result {
            Ok(_) => {}
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
                        .join(" "),
                );

                prompt::error(err_msg);
            }
            CommandError::BadArgument { arg, instead } => {
                prompt::error(format!("bad argument `{}` for {}", instead, arg));
            }
            CommandError::ArgExpectedI32 { arg, instead } => {
                prompt::error(format!(
                    "parameter {} expected integer, got `{}` instead",
                    arg, instead
                ));
            }
            CommandError::ArgExpectedU32 { arg, instead } => {
                prompt::error(format!(
                    "parameter {} expected positive integer, got `{}` instead",
                    arg, instead
                ));
            }
            CommandError::InvalidBpId { arg } => {
                prompt::error(format!("breakpoint with id {} does not exist", arg.blue()));
            }
            CommandError::HelpUnknownCommand { command } => {
                prompt::error(format!("unknown command `{}`", command));
            }
            CommandError::CannotReadFile { path, os_error } => {
                prompt::error(format!("failed to read file `{}`: {}", path, os_error));
            }
            CommandError::CannotCompile { mipsy_error } => {
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
                    MipsyError::Parser(ParserError::new(
                        parser::Error::ParseFailure,
                        Rc::from(""),
                        error.line,
                        error.col as u32,
                    )),
                    ErrorContext::Repl,
                    Some(line),
                );
            }
            CommandError::CannotCompileLine { line, error } => {
                prompt::error("failed to compile instruction");
                self.mipsy_error(error, ErrorContext::Repl, Some(line));
            }
            CommandError::LineDoesNotExist { line_number } => {
                prompt::error(format!(
                    "line :{line_number} does not exist in this program"
                ));
            }
            CommandError::UnknownRegister { register } => {
                prompt::error(format!(
                    "unknown register: {}{}",
                    "$".yellow(),
                    register.bold()
                ));
            }
            CommandError::MustLoadFile => {
                prompt::error("you have to load a file first");
            }
            CommandError::MustSpecifyFile => {
                prompt::error(
                    "there are multiple files loaded, you must specify which file to use",
                );
            }
            CommandError::ProgramExited => {
                prompt::error("program has exited");
                prompt::tip(format!(
                    "try using `{}` or `{}`",
                    "back".bold(),
                    "reset".bold()
                ));
            }
            CommandError::CannotStepFurtherBack => prompt::error("can't step any further back"),
            CommandError::RanOutOfHistory => prompt::error(format!(
                "ran out of history (max {} steps) -- try using `{}`",
                TIMELINE_MAX_LEN,
                "reset".bold()
            )),
            CommandError::RuntimeError { mipsy_error } => {
                self.mipsy_error(mipsy_error, ErrorContext::Interactive, None);
            }
            CommandError::ReplRuntimeError { mipsy_error, line } => {
                self.mipsy_error(mipsy_error, ErrorContext::Repl, Some(line));
            }
            CommandError::WithTip { error, tip } => {
                self.handle_error(*error, false);
                prompt::tip(tip);
            }
            CommandError::UnknownLabel { label } => {
                prompt::error(format!("unknown label: \"{}\"", label));
            }
            CommandError::UninitialisedRegister { register } => {
                prompt::error(format!("register {register} is uninitialized"));
            }
            CommandError::UninitialisedPrint { addr } => {
                prompt::error(format!("memory at address 0x{:08x} is uninitialized", addr));
            }
            CommandError::UnterminatedString { good_parts } => {
                prompt::error(format!("unterminated string: \"{}\"", good_parts.red()));
                prompt::tip(format!(
                    "make sure your strings are null terminated - use {} instead of {}",
                    ".asciiz".green(),
                    ".ascii".red()
                ));
            }
        }

        if nl {
            println!();
        }
    }

    pub(crate) fn mipsy_error(
        &self,
        error: MipsyError,
        context: ErrorContext,
        repl_line: Option<String>,
    ) {
        let config = &self.config;

        match error {
            MipsyError::Parser(error) => {
                if let Some(line) = repl_line {
                    error.show_error(config, Rc::from(&*line));
                } else {
                    let file_tag = error.file_tag();

                    let file = self
                        .program
                        .as_ref()
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

                    let file = self
                        .program
                        .as_ref()
                        .expect("cannot get compiler error without a file to compile")
                        .iter()
                        .find(|(tag, _)| tag.as_str() == file_tag.deref())
                        .map(|(_, str)| Rc::from(&**str))
                        .unwrap_or_else(|| Rc::from(""));

                    error.show_error(config, file);
                }
            }
            MipsyError::Runtime(error) => error.show_error(
                context,
                if let Some(line) = repl_line {
                    vec![(Rc::from(""), Rc::from(&*line))]
                } else {
                    self.program
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|(tag, content)| (Rc::from(&**tag), Rc::from(&**content)))
                        .collect()
                },
                &self.iset,
                self.binary.as_ref().unwrap(),
                &self.runtime,
            ),
        }
    }

    pub(crate) fn eval_stepped_runtime(
        &mut self,
        verbose: bool,
        result: Result<SteppedRuntime, (Runtime, MipsyError)>,
        inst: u32,
        original_pc: u32,
    ) -> CommandResult<bool> {
        let mut breakpoint = false;
        let mut trapped = false;

        match result {
            Ok(Ok(new_runtime)) => {
                self.runtime = new_runtime;
            }
            Ok(Err(guard)) => {
                // Ok(true) on exit or breakpoint, see self::exec_status
                use mipsy_lib::runtime::RuntimeSyscallGuard::*;

                match guard {
                    PrintInt(args, new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::sys1_print_int(verbose, args.value);
                    }
                    PrintFloat(args, new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::sys2_print_float(verbose, args.value);
                    }
                    PrintDouble(args, new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::sys3_print_double(verbose, args.value);
                    }
                    PrintString(args, new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::sys4_print_string(verbose, &args.value);
                    }
                    ReadInt(guard) => {
                        let value = runtime_handler::sys5_read_int(verbose);
                        self.runtime = guard(value);
                    }
                    ReadFloat(guard) => {
                        let value = runtime_handler::sys6_read_float(verbose);
                        self.runtime = guard(value);
                    }
                    ReadDouble(guard) => {
                        let value = runtime_handler::sys7_read_double(verbose);
                        self.runtime = guard(value);
                    }
                    ReadString(args, guard) => {
                        let value = runtime_handler::sys8_read_string(verbose, args.max_len);
                        self.runtime = guard(value);
                    }
                    Sbrk(args, new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::sys9_sbrk(verbose, args.bytes);
                    }
                    Exit(new_runtime) => {
                        self.runtime = new_runtime;
                        self.exited = true;

                        runtime_handler::sys10_exit(verbose);
                    }
                    PrintChar(args, new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::sys11_print_char(verbose, args.value);
                    }
                    ReadChar(guard) => {
                        let value = runtime_handler::sys12_read_char(verbose);
                        self.runtime = guard(value);
                    }
                    #[cfg(feature = "raw_io")]
                    Open(args, guard) => {
                        let value = runtime_handler::sys13_open(verbose, args);
                        self.runtime = guard(value);
                    }
                    #[cfg(feature = "raw_io")]
                    Read(args, guard) => {
                        let value = runtime_handler::sys14_read(verbose, args);
                        self.runtime = guard(value);
                    }
                    #[cfg(feature = "raw_io")]
                    Write(args, guard) => {
                        let value = runtime_handler::sys15_write(verbose, args);
                        self.runtime = guard(value);
                    }
                    #[cfg(feature = "raw_io")]
                    Close(args, guard) => {
                        let value = runtime_handler::sys16_close(verbose, args);
                        self.runtime = guard(value);
                    }
                    #[allow(unreachable_patterns)] // fall-through
                    Open(_args, guard) => {
                        let mut new_runtime = guard(-1);
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = new_runtime;

                        return Err(CommandError::RuntimeError {
                            mipsy_error: MipsyError::Runtime(RuntimeError::new(
                                Error::InvalidSyscall {
                                    syscall: SYS13_OPEN,
                                    reason: InvalidSyscallReason::Disabled,
                                },
                            )),
                        });
                    }
                    #[allow(unreachable_patterns)] // fall-through
                    Read(_args, guard) => {
                        let mut new_runtime = guard((-1, vec![]));
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = new_runtime;

                        return Err(CommandError::RuntimeError {
                            mipsy_error: MipsyError::Runtime(RuntimeError::new(
                                Error::InvalidSyscall {
                                    syscall: SYS14_READ,
                                    reason: InvalidSyscallReason::Disabled,
                                },
                            )),
                        });
                    }
                    #[allow(unreachable_patterns)] // fall-through
                    Write(_args, guard) => {
                        let mut new_runtime = guard(-1);
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = new_runtime;

                        return Err(CommandError::RuntimeError {
                            mipsy_error: MipsyError::Runtime(RuntimeError::new(
                                Error::InvalidSyscall {
                                    syscall: SYS15_WRITE,
                                    reason: InvalidSyscallReason::Disabled,
                                },
                            )),
                        });
                    }
                    #[allow(unreachable_patterns)] // fall-through
                    Close(_args, guard) => {
                        let mut new_runtime = guard(-1);
                        new_runtime.timeline_mut().pop_last_state();
                        self.runtime = new_runtime;

                        return Err(CommandError::RuntimeError {
                            mipsy_error: MipsyError::Runtime(RuntimeError::new(
                                Error::InvalidSyscall {
                                    syscall: SYS16_CLOSE,
                                    reason: InvalidSyscallReason::Disabled,
                                },
                            )),
                        });
                    }
                    ExitStatus(args, new_runtime) => {
                        self.runtime = new_runtime;
                        self.exited = true;

                        runtime_handler::sys17_exit_status(verbose, args.exit_code);
                    }
                    Breakpoint(new_runtime) => {
                        self.runtime = new_runtime;
                        breakpoint = true;
                    }
                    Trap(new_runtime) => {
                        self.runtime = new_runtime;
                        runtime_handler::trap(verbose);
                        trapped = true;
                    }
                }
            }
            Err((new_runtime, err)) => {
                self.runtime = new_runtime;

                return Err(CommandError::RuntimeError { mipsy_error: err });
            }
        };

        let mut empty_binary = Binary::default();
        let binary = self.binary.as_mut().unwrap_or(&mut empty_binary);
        let affected_registers = get_affected_registers(&self.runtime, inst);
        // TODO(joshh): move this into else if once let-chains are stabilised (1.64 baited me smh)
        let watchpoints = affected_registers
            .iter()
            .filter(|&wp| {
                binary.watchpoints.get(&wp.target).map_or(false, |watch| {
                    watch.action.fits(&wp.action) && watch.enabled
                })
            })
            .collect::<Vec<_>>();

        Ok(if self.exited {
            true
        } else {
            let pc = self.runtime.timeline().state().pc();
            let bp = binary.breakpoints.get_mut(&pc);

            if breakpoint || (bp.is_some() && bp.as_ref().unwrap().enabled) {
                if bp.is_some() && bp.as_ref().unwrap().ignore_count > 0 {
                    bp.unwrap().ignore_count -= 1;
                    trapped
                } else {
                    let label = binary
                        .labels
                        .iter()
                        .find(|(_, &addr)| addr == pc)
                        .map(|(name, _)| name.yellow().bold().to_string());

                    runtime_handler::breakpoint(label.as_deref(), pc, &binary.line_numbers);
                    if let Some(bp) = bp {
                        bp.commands.clone().iter().for_each(|command| {
                            self.exec_command(command.to_owned());
                        });
                    }

                    true
                }
            } else if !watchpoints.is_empty() {
                let mut all_ignored = true;
                let mut to_exec = Vec::new();
                for watchpoint in watchpoints {
                    let wp = binary
                        .watchpoints
                        .get_mut(&watchpoint.target)
                        .expect("I got the condition wrong");
                    if wp.ignore_count > 0 {
                        wp.ignore_count -= 1;
                    } else {
                        runtime_handler::watchpoint(watchpoint, original_pc, &binary.line_numbers);
                        to_exec.extend(wp.commands.clone().into_iter());
                        all_ignored = false;
                    }
                }

                // TODO(joshh): would be nice to have the watchpoint notification in between
                // the actions for each watchpoint
                to_exec.into_iter().for_each(|command| {
                    self.exec_command(command);
                });

                if all_ignored {
                    trapped
                } else {
                    true
                }
            } else {
                trapped
            }
        })
    }

    pub(crate) fn step(&mut self, verbose: bool) -> CommandResult<bool> {
        let runtime = take(&mut self.runtime);
        let original_pc = runtime.timeline().state().pc();
        let inst = runtime.current_inst();
        self.eval_stepped_runtime(verbose, runtime.step(), inst, original_pc)
    }

    pub(crate) fn exec_inst(&mut self, opcode: u32, verbose: bool) -> CommandResult<bool> {
        let runtime = take(&mut self.runtime);
        let original_pc = runtime.timeline().state().pc();
        self.eval_stepped_runtime(verbose, runtime.exec_inst(opcode), opcode, original_pc)
    }

    pub(crate) fn run(&mut self) -> CommandResult<String> {
        if self.exited {
            return Err(CommandError::ProgramExited);
        }

        self.interrupted.store(false, Ordering::SeqCst);
        while !self.interrupted.load(Ordering::SeqCst) {
            if self.step(false)? {
                break;
            }
        }

        Ok("".into())
    }

    pub(crate) fn reset(&mut self) -> CommandResult<()> {
        self.runtime.timeline_mut().reset();
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

pub(crate) fn editor() -> Editor<MyHelper> {
    let mut rl = Editor::new().unwrap();

    rl.set_check_cursor_position(true);

    let helper = MyHelper::new();
    rl.set_helper(Some(helper));

    rl.bind_sequence(
        KeyEvent(KeyCode::Left, Modifiers::CTRL),
        Cmd::Move(Movement::BackwardWord(1, Word::Emacs)),
    );
    rl.bind_sequence(
        KeyEvent(KeyCode::Right, Modifiers::CTRL),
        Cmd::Move(Movement::ForwardWord(1, At::BeforeEnd, Word::Emacs)),
    );

    rl
}

fn state(config: MipsyConfig) -> State {
    let mut state = State::new(config);

    state.add_command(commands::load_command());
    state.add_command(commands::run_command());
    state.add_command(commands::step_command());
    state.add_command(commands::reset_command());
    state.add_command(commands::watchpoint_command());
    state.add_command(commands::breakpoint_command());
    state.add_command(commands::disassemble_command());
    state.add_command(commands::context_command());
    state.add_command(commands::label_command());
    state.add_command(commands::labels_command());
    state.add_command(commands::examine_command());
    state.add_command(commands::print_command());
    state.add_command(commands::dot_command());
    state.add_command(commands::help_command());
    state.add_command(commands::exit_command());

    state
}

pub fn launch(config: MipsyConfig) -> ! {
    let mut rl = editor();
    let mut state = state(config);
    let interrupted = state.interrupted.clone();
    ctrlc::set_handler(move || interrupted.store(true, Ordering::SeqCst))
        .expect("Failed to set signal handler!");

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
