use crate::state::config::MipsyWebConfig;
use crate::{state::state::MipsState, utils::generate_highlighted_line};
use log::{error, info};
use mipsy_lib::compile::CompilerOptions;
use mipsy_lib::error::runtime::ErrorContext;
use mipsy_lib::{runtime::RuntimeSyscallGuard, Binary, InstSet, MipsyError, Runtime, Safe};
use mipsy_parser::TaggedFile;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew_agent::{Agent, AgentLink, HandlerId, Public};

//            Worker Overview
// ___________________________________________
// Main:   Please compile this file
// Worker: Here is the decompiled code_
// Main:   Run this Instr
// Worker: One instruction ran good
// Main:   Run this Instr
// Worker: One instruction ran good
// Main:   Run this Instr
// Worker: There's a syscall I need input for
// Main:   Here's the syscall value
// Worker: hey the instruction ran good
// Main:   Run this Instr
// Worker: There is breakpoint
// Main:   Run this Instr
// Worker: Hey the instruction
// Main:   Run this Instr
// Worker: The program exited
// Main:   Reset the runtime of the program
// Worker: Sure, here is a cleared set of registers
// ____________________________________________

pub struct Worker {
    // the link that allows to communicate to main thread
    link: AgentLink<Self>,
    inst_set: InstSet,
    // the runtime may not exist if no binary
    runtime: Option<RuntimeState>,
    // the binary will not exist if we have not been sent a file
    // realistically we should have a state that encapsulates binary and runtime
    // but that's a shift in the worker's behaviour
    // we can do that later
    binary: Option<Binary>,
    config: MipsyWebConfig,
}

type Guard<T> = Box<dyn FnOnce(T) -> Runtime>;

// for now, the below are not implemented
enum RuntimeState {
    Running(Runtime),
    WaitingInt(Guard<i32>),
    WaitingFloat(Guard<f32>),
    //WaitingDouble(Guard<f64>),
    WaitingString(Guard<Vec<u8>>),
    WaitingChar(Guard<u8>),
    //WaitingOpen(Guard<i32>),
    //WaitingRead(Guard<(i32, Vec<u8>)>),
    //WaitingWrite(Guard<i32>),
    //WaitingClose(Guard<i32>),
    //Stopped,
}

type NumSteps = i32;

#[derive(Serialize, Deserialize)]
pub enum ReadSyscallInputs {
    Int(i32),
    Float(f32),
    Double(f64),
    Char(u8),
    String(Vec<u8>),
    //Read((i32, Vec<u8>)),
}

#[derive(Serialize, Deserialize)]
pub enum WorkerRequest {
    // The struct that worker can obtain
    CompileCode(FileInformation),
    ResetRuntime(MipsState),
    UpdateConfig(MipsyWebConfig),
    // Toggle a breakpoiint at an address
    ToggleBreakpoint(u32),
    Run(MipsState, NumSteps, FileInformation),
    GiveSyscallValue(MipsState, ReadSyscallInputs),
}

#[derive(Serialize, Deserialize)]
pub struct DecompiledResponse {
    pub decompiled: String,
    pub file: Option<String>,
    pub binary: Binary,
}

#[derive(Serialize, Deserialize)]
pub struct FileInformation {
    pub file: String,
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    // error is RuntimeError, ParseError, or CompilerError
    pub error: MipsyError,
    // the file itself
    pub file: String,

    // error message
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct RuntimeErrorResponse {
    pub error: MipsyError,
    pub mips_state: MipsState,
}

#[derive(Serialize, Deserialize)]
pub enum WorkerResponse {
    DecompiledCode(DecompiledResponse),
    WorkerError(ErrorResponse),
    UpdateMipsState(MipsState),
    UpdateBinary(Option<Binary>),
    InstructionOk(MipsState),
    ProgramExited(MipsState),
    NeedInt(MipsState),
    NeedFloat(MipsState),
    NeedDouble(MipsState),
    NeedChar(MipsState),
    NeedString(MipsState),
    RuntimeError(RuntimeErrorResponse), //NeedRead((i32, Vec<u8>)),
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = WorkerRequest;
    type Output = WorkerResponse;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            inst_set: mipsy_instructions::inst_set(),
            runtime: None,
            binary: None,
            config: MipsyWebConfig::default(),
        }
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }

    fn update(&mut self, _msg: Self::Message) {
        // no messaging exists
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        let mut breakpoint = false;
        match msg {
            Self::Input::CompileCode(FileInformation { file, filename }) => {
                // TODO(shreys): this is a hack to get the file to compile
                let config = &self.config.mipsy_config;
                let compiled = mipsy_lib::compile(
                    &self.inst_set,
                    vec![TaggedFile::new(Some(&filename), file.as_str())],
                    &CompilerOptions::default(),
                    &config,
                );

                match compiled {
                    Ok(binary) => {
                        let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                        let response = Self::Output::DecompiledCode(DecompiledResponse {
                            decompiled,
                            file: Some(file),
                            binary: binary.to_owned(),
                        });
                        let runtime = mipsy_lib::runtime(&binary, &[]);
                        self.binary = Some(binary);
                        self.runtime = Some(RuntimeState::Running(runtime));

                        self.link.respond(id, response)
                    }

                    Err(error) => {
                        self.binary = None;
                        self.runtime = None;
                        let error_msg = match error {
                            MipsyError::Compiler(ref compiler_err) => {
                                format!(
                                    "{}\n{}\n{}",
                                    generate_highlighted_line(file.clone(), compiler_err),
                                    compiler_err.error().message(),
                                    compiler_err.error().tips().join("\n")
                                )
                            }
                            MipsyError::Parser(_) => String::from("failed to parse"),
                            MipsyError::Runtime(_) => {
                                unreachable!(
                                    "runtime error should not be possible at compile time"
                                );
                            }
                        };
                        self.link.respond(
                            id,
                            Self::Output::WorkerError(ErrorResponse {
                                error,
                                file,
                                message: error_msg,
                            }),
                        )
                    }
                }
            }

            Self::Input::ResetRuntime(mut mips_state) => {
                if let Some(runtime_state) = &mut self.runtime {
                    match runtime_state {
                        RuntimeState::Running(runtime) => {
                            runtime.timeline_mut().reset();
                            mips_state.stdout.drain(..);
                            mips_state.mipsy_stdout.drain(..);
                            mips_state.exit_status = None;
                            mips_state.current_instr = None;
                            mips_state.register_values = vec![Safe::Uninitialised; 32];
                            self.link
                                .respond(id, WorkerResponse::UpdateMipsState(mips_state));
                        }
                        _ => {
                            // if we are not running, then just recompile the file (since we have
                            // lost the old runtime lawl)
                            if let Some(binary) = &self.binary {
                                let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                                let response = Self::Output::DecompiledCode(DecompiledResponse {
                                    decompiled,
                                    file: None,
                                    binary: binary.to_owned(),
                                });
                                let runtime = mipsy_lib::runtime(&binary, &[]);
                                self.runtime = Some(RuntimeState::Running(runtime));
                                self.link.respond(id, response)
                            }
                        }
                    }
                } else {
                    if let Some(binary) = &self.binary {
                        let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                        let response = Self::Output::DecompiledCode(DecompiledResponse {
                            decompiled,
                            file: None,
                            binary: binary.to_owned(),
                        });
                        let runtime = mipsy_lib::runtime(&binary, &[]);
                        self.runtime = Some(RuntimeState::Running(runtime));
                        self.link.respond(id, response)
                    }
                }
            }

            // I wonder if there's a nicer way to do this with generics..?
            Self::Input::GiveSyscallValue(mips_state, val) => {
                match self.runtime.take() {
                    Some(runtime_state) => {
                        match runtime_state {
                            RuntimeState::WaitingInt(guard) => {
                                if let ReadSyscallInputs::Int(int) = val {
                                    Self::upload_syscall_value(
                                        self,
                                        mips_state,
                                        guard,
                                        int,
                                        id,
                                        format!("{}\n", int),
                                    );
                                } else {
                                    panic!("Error: please report this to developers, with steps to reproduce")
                                }
                            }

                            RuntimeState::WaitingFloat(guard) => {
                                if let ReadSyscallInputs::Float(float) = val {
                                    Self::upload_syscall_value(
                                        self,
                                        mips_state,
                                        guard,
                                        float,
                                        id,
                                        format!("{}\n", float),
                                    );
                                } else {
                                    error!("Error: please report this to developers, with steps to reproduce")
                                }
                            }
                            RuntimeState::WaitingChar(guard) => {
                                if let ReadSyscallInputs::Char(char) = val {
                                    Self::upload_syscall_value(
                                        self,
                                        mips_state,
                                        guard,
                                        char,
                                        id,
                                        format!("{}\n", char as char),
                                    );
                                } else {
                                    error!("Error: please report this to developers, with steps to reproduce")
                                }
                            }
                            RuntimeState::WaitingString(guard) => {
                                if let ReadSyscallInputs::String(string) = val {
                                    let display = String::from_utf8_lossy(&string).into_owned();
                                    Self::upload_syscall_value(
                                        self,
                                        mips_state,
                                        guard,
                                        string,
                                        id,
                                        format!("{}\n", display),
                                    );
                                } else {
                                    error!("Error: please report this to developers, with steps to reproduce")
                                }
                            }

                            _ => {
                                error!("Error: please report this to developers, with steps to reproduce")
                            }
                        }
                    }
                    None => {
                        error!("Error: please report this to developers, with steps to reproduce. Runtime is None when Giving Syscall Val")
                    }
                }
            }

            /* Toggles a breakpoint at an address
             */
            Self::Input::ToggleBreakpoint(addr) => {
                let binary = self.binary.as_mut();
                if let Some(binary) = binary {
                    if binary.breakpoints.iter().any(|&x| x == addr) {
                        binary.breakpoints.retain(|&x| addr != x);
                    } else {
                        binary.breakpoints.push(addr);
                    }
                }

                let response = Self::Output::UpdateBinary(self.binary.clone());
                self.link.respond(id, response)
            }

            Self::Input::UpdateConfig(config) => self.config = config,

            Self::Input::Run(mut mips_state, step_size, FileInformation { file, filename }) => {
                let binary = self.binary.as_ref().unwrap();
                if let Some(runtime_state) = self.runtime.take() {
                    if let RuntimeState::Running(mut runtime) = runtime_state {
                        // fast forward the kernel segment
                        // or, fast rewind the kernel segment if we are rewinding
                        if runtime.timeline().state().pc() >= mipsy_lib::KTEXT_BOT {
                            while runtime.timeline().state().pc() >= mipsy_lib::KTEXT_BOT {
                                // info!("stepping ktext: {:08x}", runtime.timeline().state().pc());
                                if step_size == -1 {
                                    info!("stepping back: {:08x}", runtime.timeline().state().pc());
                                    runtime.timeline_mut().pop_last_state();
                                    mips_state.exit_status = None;
                                    // avoid infinite loop of scrolling back
                                    if runtime.timeline().state().pc() == mipsy_lib::KTEXT_BOT {
                                        break;
                                    }
                                } else {
                                    let stepped_runtime = runtime.step();
                                    match stepped_runtime {
                                        Ok(Ok(next_runtime)) => {

                                            let pc = next_runtime.timeline().state().pc();

                                            runtime = next_runtime;
                                            // we want to stop at the instruction before the breakpoint
                                            // so that the line of breakpoint doesnt get executed
                                            if binary.breakpoints.contains(&pc)
                                                && !self.config.ignore_breakpoints
                                            {
                                                breakpoint = true;
                                                break;
                                            }
                                        }
                                        Ok(Err(guard)) => {
                                            use RuntimeSyscallGuard::*;
                                            match guard {
                                                ExitStatus(exit_status_args, next_runtime) => {
                                                    info!("Exit in kernel");

                                                    mips_state.exit_status =
                                                        Some(exit_status_args.exit_code);
                                                    runtime = next_runtime;
                                                }
                                                _ => {
                                                    error!("Some non-exit status syscall exists in the kernel");
                                                    return;
                                                }
                                            }
                                        }
                                        Err((prev_runtime, err)) => {
                                            runtime = prev_runtime;
                                            mips_state.update_registers(&runtime);
                                            mips_state.update_current_instr(&runtime);
                                            mips_state.update_memory(&runtime);
                                            self.runtime = Some(RuntimeState::Running(runtime));
                                            mips_state.mipsy_stdout.push(format!("{:?}", err));
                                            let response =
                                                Self::Output::UpdateMipsState(mips_state);
                                            self.link.respond(id, response);
                                            error!("error when fast forwarding: {:?}", err);
                                            return;
                                        }
                                    }
                                }
                                if mips_state.exit_status.is_some() {
                                    break;
                                };
                            }
                            mips_state.update_registers(&runtime);
                            mips_state.update_current_instr(&runtime);
                            mips_state.update_memory(&runtime);
                            let pc = runtime.timeline().state().pc();
                            let binary = self.binary.as_ref().unwrap();
                            self.runtime = Some(RuntimeState::Running(runtime));
                            if mips_state.exit_status.is_some() {
                                let response = Self::Output::ProgramExited(mips_state);
                                self.link.respond(id, response);
                            } else if breakpoint {
                                // the label or address
                                let label = binary
                                    .labels
                                    .iter()
                                    .find(|(_, &addr)| addr == pc)
                                    .map(|(name, _)| name.to_string())
                                    .unwrap_or(format!("0x{:08x}", pc));
                                mips_state
                                    .mipsy_stdout
                                    .push(format!("BREAKPOINT - {label}"));

                                let response = Self::Output::UpdateMipsState(mips_state);
                                self.link.respond(id, response);
                            } else if mips_state.is_stepping {
                                let response = Self::Output::UpdateMipsState(mips_state);
                                self.link.respond(id, response);
                            } else {
                                let response = Self::Output::InstructionOk(mips_state);
                                self.link.respond(id, response);
                            }
                            return;
                        }

                        // prevent us from stepping too far in kernel and exiting
                        if step_size == -1 {
                            runtime.timeline_mut().pop_last_state();
                            mips_state.exit_status = None;
                        }

                        // now let's us step the right number of times
                        for _ in 1..=step_size {
                            if runtime.timeline().state().pc() >= mipsy_lib::KTEXT_BOT {
                                break;
                            }

                            // info!("stepping text: {:08x}", runtime.timeline().state().pc());
                            let stepped_runtime = runtime.step();
                            match stepped_runtime {
                                // instruction ran okay
                                Ok(Ok(next_runtime)) => {
                                    let pc = next_runtime.timeline().state().pc();

                                    runtime = next_runtime;
                                    // we want to stop at the instruction before the breakpoint
                                    // so that the line of breakpoint doesnt get executed
                                    if binary.breakpoints.contains(&pc)
                                        && !self.config.ignore_breakpoints
                                    {
                                        breakpoint = true;
                                        break;
                                    }
                                }

                                // instruction ran but we need to handle a syscall
                                Ok(Err(guard)) => {
                                    use RuntimeSyscallGuard::*;
                                    match guard {
                                        PrintInt(print_int_args, next_runtime) => {
                                            info!("printing integer {}", print_int_args.value);

                                            mips_state
                                                .stdout
                                                .push(print_int_args.value.to_string());

                                            runtime = next_runtime;
                                        }

                                        PrintFloat(print_float_args, next_runtime) => {
                                            info!("printing float {}", print_float_args.value);

                                            mips_state
                                                .stdout
                                                .push(print_float_args.value.to_string());

                                            runtime = next_runtime;
                                        }

                                        PrintDouble(print_double_args, next_runtime) => {
                                            info!("printing double {}", print_double_args.value);

                                            mips_state
                                                .stdout
                                                .push(print_double_args.value.to_string());

                                            runtime = next_runtime;
                                        }

                                        PrintString(print_string_args, next_runtime) => {
                                            let string_value =
                                                String::from_utf8_lossy(&print_string_args.value)
                                                    .to_string();

                                            info!("printing string {:?}", string_value);

                                            mips_state.stdout.push(string_value);

                                            runtime = next_runtime;
                                        }

                                        PrintChar(print_char_args, next_runtime) => {
                                            let string =
                                                String::from_utf8_lossy(&[print_char_args.value])
                                                    .to_string();

                                            info!("printing! char {:?}", string);

                                            mips_state.stdout.push(string);

                                            runtime = next_runtime;
                                        }

                                        ReadInt(guard) => {
                                            info!("reading int");
                                            self.runtime = Some(RuntimeState::WaitingInt(guard));

                                            self.link
                                                .respond(id, WorkerResponse::NeedInt(mips_state));

                                            return;
                                        }

                                        ReadFloat(guard) => {
                                            info!("reading float");
                                            self.runtime = Some(RuntimeState::WaitingFloat(guard));

                                            self.link
                                                .respond(id, WorkerResponse::NeedFloat(mips_state));

                                            return;
                                        }

                                        ReadString(_str_args, guard) => {
                                            info!("reading string");
                                            self.runtime = Some(RuntimeState::WaitingString(guard));

                                            self.link.respond(
                                                id,
                                                WorkerResponse::NeedString(mips_state),
                                            );

                                            return;
                                        }

                                        ReadChar(guard) => {
                                            info!("Reading char");
                                            self.runtime = Some(RuntimeState::WaitingChar(guard));

                                            self.link
                                                .respond(id, WorkerResponse::NeedChar(mips_state));

                                            return;
                                        }

                                        Sbrk(_sbrk_args, next_runtime) => {
                                            info!("sbrk");

                                            runtime = next_runtime;
                                        }

                                        Exit(next_runtime) => {
                                            info!("exit syscall");

                                            mips_state.exit_status = Some(0);

                                            runtime = next_runtime;
                                        }

                                        Open(_open_args, _fn_ptr) => {
                                            error!("open syscall is not supported by mipsy_web.");

                                            mips_state
                                                .mipsy_stdout
                                                .push("Open syscall not supported".to_string());
                                            runtime = _fn_ptr(42);
                                        }

                                        Write(_write_args, _fn_ptr) => {
                                            error!("write syscall is not supported by mipsy_web ");

                                            mips_state
                                                .mipsy_stdout
                                                .push("Write syscall not supported".to_string());
                                            runtime = _fn_ptr(42);
                                        }

                                        Close(_close_args, _fn_ptr) => {
                                            info!("Close");

                                            mips_state
                                                .mipsy_stdout
                                                .push("Close syscall not supported".to_string());
                                            runtime = _fn_ptr(42);
                                        }

                                        ExitStatus(exit_status_args, next_runtime) => {
                                            info!("Exit");

                                            mips_state.exit_status =
                                                Some(exit_status_args.exit_code);
                                            runtime = next_runtime;
                                        }

                                        Breakpoint(next_runtime) => {
                                            runtime = next_runtime;
                                            if !self.config.ignore_breakpoints {
                                                breakpoint = true;
                                                break;
                                            }
                                        }

                                        _ => unreachable!("Invalid Syscall"), /*

                                                                              Sbrk       (SbrkArgs, Runtime),
                                                                              Exit       (Runtime),
                                                                              PrintChar  (PrintCharArgs, Runtime),
                                                                              ReadChar   (           Box<dyn FnOnce(u8)             -> Runtime>),
                                                                              Open       (OpenArgs,  Box<dyn FnOnce(i32)            -> Runtime>),
                                                                              Read       (ReadArgs,  Box<dyn FnOnce((i32, Vec<u8>)) -> Runtime>),
                                                                              Write      (WriteArgs, Box<dyn FnOnce(i32)            -> Runtime>),
                                                                              Close      (CloseArgs, Box<dyn FnOnce(i32)            -> Runtime>),
                                                                              ExitStatus (ExitStatusArgs, Runtime),

                                                                              // other
                                                                              Breakpoint     (Runtime),
                                                                              */
                                    }
                                }

                                // mipsy runtime error
                                Err((prev_runtime, error)) => {
                                    runtime = prev_runtime;
                                    mips_state.update_registers(&runtime);
                                    mips_state.update_current_instr(&runtime);
                                    mips_state.update_memory(&runtime);
                                    self.runtime = Some(RuntimeState::Running(runtime));

                                    match &error {
                                        MipsyError::Runtime(runtime_error) => {
                                            let filename: Rc<str> = Rc::from(filename);
                                            let file: Rc<str> = Rc::from(file.clone());
                                            let source_code = [(filename.clone(), file)];
                                            let runtime = match self.runtime {
                                                Some(RuntimeState::Running(ref runtime)) => runtime,
                                                _ => unreachable!("runtime not running"),
                                            };
                                            let binary = self
                                                .binary
                                                .as_ref()
                                                .expect("binary should exist if runtime error");
                                            let message = format!(
                                                "error: {}\ntips: {}\n",
                                                runtime_error.error().message(
                                                    ErrorContext::Binary,
                                                    &source_code[..],
                                                    &self.inst_set,
                                                    binary,
                                                    runtime
                                                ),
                                                runtime_error
                                                    .error()
                                                    .tips(
                                                        &source_code[..],
                                                        &self.inst_set,
                                                        binary,
                                                        runtime
                                                    )
                                                    .join("\n")
                                            );

                                            mips_state.mipsy_stdout.push(message);
                                            let response =
                                                Self::Output::RuntimeError(RuntimeErrorResponse {
                                                    mips_state: mips_state.clone(),
                                                    error,
                                                });
                                            self.link.respond(id, response);
                                            return;
                                        }

                                        MipsyError::Parser(_) | MipsyError::Compiler(_) => {
                                            unreachable!("parser errors and compiler errors should not happen at runtime")
                                        }
                                    };
                                }
                            }

                            if mips_state.exit_status.is_some() {
                                break;
                            };

                            // usually we only update registers after all steps ran, but in case
                            // next instruction is a syscall, we need to update registers before it
                            // early exits
                            if let Ok(next_instr_is_read_syscall) = runtime.next_inst_may_guard() {
                                if next_instr_is_read_syscall {
                                    mips_state.update_registers(&runtime);
                                    mips_state.update_current_instr(&runtime);
                                    mips_state.update_memory(&runtime);
                                }
                            }
                        }
                        mips_state.update_registers(&runtime);
                        mips_state.update_current_instr(&runtime);
                        mips_state.update_memory(&runtime);
                        let pc = runtime.timeline().state().pc();
                        let binary = self.binary.as_ref().unwrap();
                        self.runtime = Some(RuntimeState::Running(runtime));

                        let response;
                        if mips_state.exit_status.is_some() {
                            mips_state.stdout.push(format!(
                                "\nProgram exited with exit status {}",
                                mips_state
                                    .exit_status
                                    .expect("infinite loop guarantees Some return")
                            ));
                            response = Self::Output::ProgramExited(mips_state);
                        } else if breakpoint {
                            // the label or address
                            let label = binary
                                .labels
                                .iter()
                                .find(|(_, &addr)| addr == pc)
                                .map(|(name, _)| name.to_string())
                                .unwrap_or(format!("0x{:08x}", pc));
                            mips_state
                                .mipsy_stdout
                                .push(format!("BREAKPOINT - {label}"));

                            response = Self::Output::UpdateMipsState(mips_state);
                        } else if step_size.abs() == 1 {
                            // just update the state
                            response = Self::Output::UpdateMipsState(mips_state);
                        } else {
                            // update state and tell frontend to run more isntructions
                            response = Self::Output::InstructionOk(mips_state);
                        }

                        self.link.respond(id, response);
                    }
                }
            }
        }
    }
}

impl Worker {
    fn upload_syscall_value<T>(
        &mut self,
        mut mips_state: MipsState,
        guard: Guard<T>,
        val: T,
        id: HandlerId,
        serialized: String,
    ) {
        mips_state.stdout.push(serialized);

        let runtime = guard(val);

        mips_state.update_registers(&runtime);
        mips_state.update_current_instr(&runtime);
        mips_state.update_memory(&runtime);

        self.runtime = Some(RuntimeState::Running(runtime));

        // if running we want InstructionOk
        // if stepping we want UpdateMipsState

        if mips_state.is_stepping {
            self.link
                .respond(id, <Worker as Agent>::Output::UpdateMipsState(mips_state))
        } else {
            self.link
                .respond(id, <Worker as Agent>::Output::InstructionOk(mips_state))
        }
    }
}
