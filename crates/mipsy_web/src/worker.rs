use mipsy_lib::{runtime::RuntimeSyscallGuard, Binary, InstSet, MipsyError, Runtime};
use mipsy_parser::TaggedFile;
use serde::{Deserialize, Serialize};
use yew::{
    services::ConsoleService,
    worker::{Agent, AgentLink, HandlerId, Public},
};

use crate::app::MipsState;

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
    counter: i8,
}

type Guard<T> = Box<dyn FnOnce(T) -> Runtime>;
enum RuntimeState {
    Running(Runtime),
    WaitingInt(Guard<i32>),
    WaitingFloat(Guard<f32>),
    WaitingDouble(Guard<f64>),
    WaitingString(Guard<Vec<u8>>),
    WaitingChar(Guard<u8>),
    WaitingOpen(Guard<i32>),
    WaitingRead(Guard<(i32, Vec<u8>)>),
    WaitingWrite(Guard<i32>),
    WaitingClose(Guard<i32>),
    Stopped,
}

type File = String;

#[derive(Serialize, Deserialize)]
pub enum WorkerRequest {
    // The struct that worker can obtain
    CompileCode(File),
    RunCode(MipsState),
}

#[derive(Serialize, Deserialize)]
pub enum WorkerResponse {
    DecompiledCode(String),
    CompilerError(MipsyError),
    MipsyState(MipsState),
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = WorkerRequest;
    type Output = WorkerResponse;

    fn create(link: AgentLink<Self>) -> Self {
        ConsoleService::info("CREATING WORKER");
        Self {
            link,
            inst_set: mipsy_codegen::instruction_set!("../../mips.yaml"),
            runtime: None,
            binary: None,
            counter: 0,
        }
    }

    fn name_of_resource() -> &'static str {
        "wasm.js"
    }

    fn update(&mut self, msg: Self::Message) {
        // no messaging exists
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        ConsoleService::warn("Recieved input");
        match msg {
            Self::Input::CompileCode(f) => {
                let compiled =
                    mipsy_lib::compile(&self.inst_set, vec![TaggedFile::new(None, f.as_str())], 8);

                match compiled {
                    Ok(binary) => {
                        let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                        let response = Self::Output::DecompiledCode(decompiled);
                        self.binary = Some(binary);
                        self.link.respond(id, response)
                    }

                    Err(err) => self.link.respond(id, Self::Output::CompilerError(err)),
                }
            }

            Self::Input::RunCode(mut mips_state) => {
                ConsoleService::warn("Recieved run request");
                if let Some(binary) = &self.binary {
                    mips_state.stdout.drain(..);
                    ConsoleService::warn("here");
                    let mut runtime = mipsy_lib::runtime(&binary, &[]);
                    ConsoleService::warn("there");
                    loop {
                        let stepped_runtime = runtime.step();

                        ConsoleService::warn("step");
                        match stepped_runtime {
                            Ok(Ok(next_runtime)) => runtime = next_runtime,
                            Ok(Err(guard)) => {
                                use RuntimeSyscallGuard::*;
                                match guard {
                                    PrintInt(print_int_args, next_runtime) => {
                                        ConsoleService::warn(&format!(
                                            "printing integer {}",
                                            print_int_args.value
                                        ));

                                        runtime = next_runtime;
                                    }

                                    PrintFloat(print_float_args, next_runtime) => {
                                        ConsoleService::warn(&format!(
                                            "printing float {}",
                                            print_float_args.value
                                        ));

                                        runtime = next_runtime;
                                    }

                                    PrintDouble(print_double_args, next_runtime) => {
                                        ConsoleService::warn(&format!(
                                            "printing double {}",
                                            print_double_args.value
                                        ));

                                        runtime = next_runtime;
                                    }

                                    PrintString(print_string_args, next_runtime) => {
                                        ConsoleService::warn(&format!(
                                            "printing string {:?}",
                                            print_string_args.value
                                        ));

                                        runtime = next_runtime;
                                    }

                                    PrintChar(print_char_args, next_runtime) => {
                                        ConsoleService::warn(&format!(
                                            "printing char {:?}",
                                            print_char_args.value
                                        ));

                                        runtime = next_runtime;
                                    }

                                    ReadInt(_fn_ptr) => {
                                        ConsoleService::warn(&format!("reading int"));
                                        runtime = _fn_ptr(42);
                                    }

                                    ReadFloat(fn_ptr) => {
                                        ConsoleService::warn(&format!("reading float"));
                                        runtime = fn_ptr(42.0);
                                        todo!();
                                    }

                                    ReadString(_str_args, fn_ptr) => {
                                        ConsoleService::warn(&format!("reading string"));
                                        runtime = fn_ptr(vec![99, 99, 99, 99]);
                                        todo!();
                                    }

                                    ReadChar(fn_ptr) => {
                                        ConsoleService::warn(&format!("Reading char"));
                                        fn_ptr(79);
                                        todo!();
                                    }

                                    Sbrk(_sbrk_args, next_runtime) => {
                                        ConsoleService::warn(&format!("sbrk"));
                                        runtime = next_runtime;
                                        todo!();
                                    }

                                    Exit(next_runtime) => {
                                        ConsoleService::warn(&format!("exit syscall"));
                                        runtime = next_runtime;
                                    }

                                    Open(_open_args, _fn_ptr) => {
                                        ConsoleService::warn(&format!("open"));
                                        runtime = _fn_ptr(42);
                                        todo!();
                                    }

                                    Write(_write_args, _fn_ptr) => {
                                        ConsoleService::warn(&format!("write"));
                                        runtime = _fn_ptr(42);
                                        todo!();
                                    }

                                    Close(_close_args, _fn_ptr) => {
                                        ConsoleService::warn(&format!("Close"));
                                        runtime = _fn_ptr(42);
                                    }

                                    ExitStatus(_exit_status_args, next_runtime) => {
                                        ConsoleService::warn(&format!("Exit"));
                                        runtime = next_runtime;
                                        self.counter += 1;
                                        ConsoleService::warn(&format!("counter: {}", self.counter));
                                    }

                                    Breakpoint(next_runtime) => {
                                        ConsoleService::warn(&format!("breakpoint"));
                                        runtime = next_runtime;
                                    }

                                    UnknownSyscall(_unknown_syscall_args, next_runtime) => {
                                        ConsoleService::warn(&format!("Unknown"));
                                        runtime = next_runtime;
                                    }

                                    _ => todo!(), /*

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
                                                  UnknownSyscall (UnknownSyscallArgs, Runtime)
                                                  */
                                }
                            }
                            Err((prev_runtime, err)) => {
                                runtime = prev_runtime;
                                todo!("Send error to frontend iguess");
                            }
                        }

                        if mips_state.exit_status.is_some() {
                            break;
                        }
                    }

                    mips_state.stdout.push(format!(
                        "\nProgram exited with exit status {}",
                        mips_state
                            .exit_status
                            .expect("infinite loop guarantees Some return")
                    ));

                    let response = Self::Output::MipsyState(mips_state);
                    self.link.respond(id, response);
                }
            }
        }
    }
}
