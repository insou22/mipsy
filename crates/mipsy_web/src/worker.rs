use log::{info, warn, LevelFilter};
use mipsy_lib::{
    inst::register::REGISTERS, runtime::RuntimeSyscallGuard, Binary, InstSet, MipsyError, Runtime,
};
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
    ResetRuntime(MipsState),
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
        info!("CREATING WORKER");
        wasm_logger::init(wasm_logger::Config::default());

        Self {
            link,
            inst_set: mipsy_codegen::instruction_set!("../../mips.yaml"),
            runtime: None,
            binary: None,
        }
    }

    fn name_of_resource() -> &'static str {
        "wasm.js"
    }

    fn update(&mut self, msg: Self::Message) {
        // no messaging exists
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        info!("Recieved input");
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

            Self::Input::ResetRuntime(mut mips_state) => {
                warn!("Recieved reset request ");
                if let Some(runtime_state) = &mut self.runtime {
                    match runtime_state {
                        RuntimeState::Running(runtime) => {
                            runtime.timeline_mut().reset();
                            mips_state.stdout.drain(..);
                            self.link
                                .respond(id, WorkerResponse::MipsyState(mips_state));
                        }
                        _ => {}
                    }
                }
            }

            Self::Input::RunCode(mut mips_state) => {
                info!("Recieved run request");

                if let Some(binary) = &self.binary {
                    mips_state.stdout.drain(..);
                    let mut runtime = mipsy_lib::runtime(&binary, &[]);
                    // runtime.timeline().state().registers;
                    loop {
                        let stepped_runtime = runtime.step();
                        info!("step");
                        match stepped_runtime {
                            Ok(Ok(next_runtime)) => runtime = next_runtime,
                            Ok(Err(guard)) => {
                                use RuntimeSyscallGuard::*;
                                match guard {
                                    PrintInt(print_int_args, next_runtime) => {
                                        info!("printing integer {}", print_int_args.value);

                                        mips_state.stdout.push(print_int_args.value.to_string());

                                        runtime = next_runtime;
                                    }

                                    PrintFloat(print_float_args, next_runtime) => {
                                        info!("printing float {}", print_float_args.value);

                                        mips_state.stdout.push(print_float_args.value.to_string());

                                        runtime = next_runtime;
                                    }

                                    PrintDouble(print_double_args, next_runtime) => {
                                        info!("printing double {}", print_double_args.value);

                                        mips_state.stdout.push(print_double_args.value.to_string());

                                        runtime = next_runtime;
                                    }

                                    PrintString(print_string_args, next_runtime) => {
                                        info!("printing string {:?}", print_string_args.value);

                                        mips_state.stdout.push(
                                            String::from_utf8_lossy(&print_string_args.value)
                                                .to_string(),
                                        );

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

                                    ReadInt(_fn_ptr) => {
                                        info!("reading int");
                                        runtime = _fn_ptr(42);
                                    }

                                    ReadFloat(fn_ptr) => {
                                        info!("reading float");

                                        runtime = fn_ptr(42.0);
                                        todo!();
                                    }

                                    ReadString(_str_args, fn_ptr) => {
                                        info!("reading string");

                                        runtime = fn_ptr(vec![99, 99, 99, 99]);
                                        todo!();
                                    }

                                    ReadChar(fn_ptr) => {
                                        info!("Reading char");

                                        fn_ptr(79);
                                        todo!();
                                    }

                                    Sbrk(_sbrk_args, next_runtime) => {
                                        info!("sbrk");

                                        runtime = next_runtime;
                                        todo!();
                                    }

                                    Exit(next_runtime) => {
                                        info!("exit syscall");

                                        mips_state.exit_status = Some(0);

                                        runtime = next_runtime;
                                    }

                                    Open(_open_args, _fn_ptr) => {
                                        info!("open");

                                        runtime = _fn_ptr(42);

                                        todo!();
                                    }

                                    Write(_write_args, _fn_ptr) => {
                                        info!("write");

                                        runtime = _fn_ptr(42);

                                        todo!();
                                    }

                                    Close(_close_args, _fn_ptr) => {
                                        info!("Close");

                                        runtime = _fn_ptr(42);
                                    }

                                    ExitStatus(exit_status_args, next_runtime) => {
                                        info!("Exit");

                                        mips_state.exit_status = Some(exit_status_args.exit_code);
                                        runtime = next_runtime;
                                    }

                                    Breakpoint(next_runtime) => {
                                        info!("breakpoint");

                                        runtime = next_runtime;
                                    }

                                    UnknownSyscall(_unknown_syscall_args, next_runtime) => {
                                        info!("Unknown");

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

                    mips_state.register_values = runtime
                        .timeline()
                        .state()
                        .registers()
                        .iter()
                        .cloned()
                        .collect();

                    self.runtime = Some(RuntimeState::Running(runtime));

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
