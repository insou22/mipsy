use log::{error, info, warn, LevelFilter};
use mipsy_lib::{runtime::RuntimeSyscallGuard, Binary, InstSet, MipsyError, Runtime, Safe};
use mipsy_parser::TaggedFile;
use mipsy_utils::MipsyConfig;
use serde::{Deserialize, Serialize};
use yew::worker::{Agent, AgentLink, HandlerId, Public};

use crate::app::MipsState;

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
type NumSteps = i32;

#[derive(Serialize, Deserialize)]
pub enum WorkerRequest {
    // The struct that worker can obtain
    CompileCode(File),
    ResetRuntime(MipsState),
    Run(MipsState, NumSteps),
}

#[derive(Serialize, Deserialize)]
pub enum WorkerResponse {
    DecompiledCode(String),
    CompilerError(MipsyError),
    UpdateMipsState(MipsState),
    InstructionOk(MipsState),
    ProgramExited(MipsState),

    NeedInt(MipsState),
    //NeedFloat(f32),
    //NeedDouble(f64),
    //NeedString(Vec<u8>),
    //NeedChar(u8),
    //NeedRead((i32, Vec<u8>)),

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

    fn update(&mut self, _msg: Self::Message) {
        // no messaging exists
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Self::Input::CompileCode(f) => {
                // TODO(shreys): this is a hack to get the file to compile
                let config = MipsyConfig {
                    tab_size: 8,
                    spim: false,
                };
                let compiled =
                    mipsy_lib::compile(&self.inst_set, vec![TaggedFile::new(None, f.as_str())], &config);

                match compiled {
                    Ok(binary) => {
                        let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                        let response = Self::Output::DecompiledCode(decompiled);
                        let runtime = mipsy_lib::runtime(&binary, &[]);
                        self.binary = Some(binary);
                        self.runtime = Some(RuntimeState::Running(runtime));

                        self.link.respond(id, response)
                    }

                    Err(err) => self.link.respond(id, Self::Output::CompilerError(err)),
                }
            }

            Self::Input::ResetRuntime(mut mips_state) => {
                if let Some(runtime_state) = &mut self.runtime {
                    match runtime_state {
                        RuntimeState::Running(runtime) => {
                            runtime.timeline_mut().reset();
                            mips_state.stdout.drain(..);
                            mips_state.exit_status = None;
                            mips_state.current_instr = None;
                            mips_state.register_values = vec![Safe::Uninitialised; 32];
                            self.link
                                .respond(id, WorkerResponse::UpdateMipsState(mips_state));
                        }
                        _ => {}
                    }
                }
            }


            Self::Input::Run(mut mips_state, step_size) => {

                if let Some(runtime_state) = self.runtime.take() {
                    if let RuntimeState::Running(mut runtime) = runtime_state {
                        if step_size == -1 {
                            runtime.timeline_mut().pop_last_state();
                            mips_state.exit_status = None;
                        } 
                        
                        
                        // kernel instructions start with 0x80000000
                        // we want to keep looping forever if we're in a kernel section
                        // so that step only steps user space
                        let mut fast_forwarded = false;
                        while runtime.timeline().state().pc() >= 0x80000000 {
                            
                            if step_size == -1 {
                                runtime.timeline_mut().pop_last_state();
                                mips_state.exit_status = None;
                            } else {
                                fast_forwarded = true;
                                let stepped_runtime = runtime.step();
                                match stepped_runtime {
                                    Ok(Ok(next_runtime)) => {runtime = next_runtime},
                                    Ok(Err(guard)) => {
                                        use RuntimeSyscallGuard::*;
                                        match guard {
                                            ExitStatus(exit_status_args, next_runtime) => {
                                                info!("Exit");

                                                mips_state.exit_status = Some(exit_status_args.exit_code);
                                                runtime = next_runtime;
                                            }
                                            _ => { unreachable!() }
                                        }
                                    }
                                    Err((prev_runtime, err)) => {
                                        runtime = prev_runtime;
                                        error!("{:?}", err);
                                        todo!("Send error to frontend iguess");
                                    }
                                }
                            }
                        }

                        if fast_forwarded {
                            runtime.timeline_mut().pop_last_state();
                            mips_state.exit_status = None;
                        }


                        for _ in 1..=step_size {
                            let old_runtime = runtime.clone();
                            let stepped_runtime = runtime.step();
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

                                        ReadInt(guard) => {
                                            info!("reading int");
                                            self.runtime = Some(RuntimeState::WaitingInt(guard));

                                            // we want to send the state of mipsy BEFORE executing
                                            // this instruction, but I can't access previous
                                            // runtime T_T_T_T_T__T_T
                                            // do a clone for now and hope zac doesn't yell at me
                                            mips_state.current_instr = Some(old_runtime.timeline().state().pc());
                                            mips_state.register_values = old_runtime
                                                .timeline()
                                                .state()
                                                .registers()
                                                .iter()
                                                .cloned()
                                                .collect();
                                            self.link.respond(id, WorkerResponse::NeedInt(mips_state));
                                            return;
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
                                    error!("{:?}", err);
                                    todo!("Send error to frontend iguess");
                                }
                            }
                            
                            if mips_state.exit_status.is_some() { break };
                        }

                        // runtime has been swallowed at this point, but the result is a guard
                        // until we get a response from the main thread
                        // I think the solution is to do a link.respond and end function inside the
                        // guard match
                        // BUT ALSO then, it's possible that the registers shown to the user while
                        // they are waiting on syscall may be incorrect or outdated (since they are
                        // only updated every 40 instructions);
                        mips_state.current_instr = Some(runtime.timeline().state().pc());
                        mips_state.register_values = runtime
                            .timeline()
                            .state()
                            .registers()
                            .iter()
                            .cloned()
                            .collect();

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
                        } else if step_size.abs() == 1 {
                            // just update the state
                            response = Self::Output::UpdateMipsState(mips_state);
                        } 
                        else {
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
