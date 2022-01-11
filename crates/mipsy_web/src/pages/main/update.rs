use crate::worker::ReadSyscallInputs;
use crate::worker::{WorkerRequest, WorkerResponse};
use crate::{
    pages::main::{
        app::{
            process_syscall_request, process_syscall_response, ReadSyscalls,
            NUM_INSTR_BEFORE_RESPONSE,
        },
        state::{MipsState, RunningState, State},
    },
    worker::Worker,
};
use gloo_file::callbacks::read_as_text;
use log::{error, info, trace};
use mipsy_lib::{MipsyError, Safe};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::UseBridgeHandle;

pub fn handle_response_from_worker(
    state: UseStateHandle<State>,
    response: WorkerResponse,
    rerender_hook: UseStateHandle<bool>,
    worker: Rc<RefCell<Option<UseBridgeHandle<Worker>>>>,
    input_ref: UseStateHandle<NodeRef>,
) {
    // TODO - hande response
    // TODO - fix compiler errors
    match response {
        WorkerResponse::DecompiledCode(decompiled) => {
            info!("recieved decompiled code from worker");
            state.set(State::Running(RunningState {
                decompiled,
                mips_state: MipsState {
                    stdout: Vec::new(),
                    exit_status: None,
                    register_values: vec![Safe::Uninitialised; 32],
                    current_instr: None,
                    mipsy_stdout: Vec::new(),
                    is_stepping: true,
                },
                input_needed: None,
                should_kill: false,
            }));
            true
        }

        WorkerResponse::CompilerError(err) => match &err {
            MipsyError::Parser(_error) => true,

            MipsyError::Compiler(error) => {
                match *state {
                    State::Running(ref curr) => {
                        let mut new_vec = curr.mips_state.mipsy_stdout.clone();
                        new_vec.push(error.error().message());
                        let new_mips_state = MipsState {
                            mipsy_stdout: new_vec,
                            ..curr.mips_state.clone()
                        };
                        state.set(State::Running(RunningState {
                            mips_state: new_mips_state,
                            ..curr.clone()
                        }))
                    }
                    State::NoFile | State::CompilerError(_) => {
                        state.set(State::CompilerError(err));
                        error!("Compiler errors are not supported.");
                    }
                }
                true
            }

            MipsyError::Runtime(_error) => {
                error!("Cannot get runtime error at compile time");
                unreachable!();
            }
        },

        WorkerResponse::ProgramExited(mips_state) => match *state {
            State::Running(ref curr) => {
                state.set(State::Running(RunningState {
                    mips_state,
                    ..curr.clone()
                }));
                true
            }
            State::NoFile | State::CompilerError(_) => false,
        },

        WorkerResponse::InstructionOk(mips_state) => {
            if let State::Running(ref curr) = *state {
                info!("{:?}", mips_state);
                info!("HERE");
                state.set(State::Running(RunningState {
                    mips_state: mips_state.clone(),
                    ..curr.clone()
                }));

                // if the isntruction was ok, run another instruction
                // unless the user has said it should be killed
                if !curr.should_kill {
                    let input = WorkerRequest::Run(mips_state.clone(), NUM_INSTR_BEFORE_RESPONSE);
                    worker.borrow().as_ref().unwrap().send(input);
                }

                state.set(State::Running(RunningState {
                    should_kill: false,
                    mips_state,
                    ..curr.clone()
                }));
            } else {
                info!("No File loaded, cannot run");
            }
            true
        }

        WorkerResponse::UpdateMipsState(mips_state) => match *state {
            State::Running(ref curr) => {
                state.set(State::Running(RunningState {
                    mips_state,
                    ..curr.clone()
                }));
                info!("updating state");
                true
            }

            State::NoFile | State::CompilerError(_) => false,
        },

        WorkerResponse::NeedInt(mips_state) => {
            process_syscall_request(mips_state, ReadSyscalls::ReadInt, state, input_ref)
        }
        WorkerResponse::NeedFloat(mips_state) => {
            process_syscall_request(mips_state, ReadSyscalls::ReadFloat, state, input_ref)
        }
        WorkerResponse::NeedDouble(mips_state) => {
            process_syscall_request(mips_state, ReadSyscalls::ReadDouble, state, input_ref)
        }
        WorkerResponse::NeedChar(mips_state) => {
            process_syscall_request(mips_state, ReadSyscalls::ReadChar, state, input_ref)
        }
        WorkerResponse::NeedString(mips_state) => {
            process_syscall_request(mips_state, ReadSyscalls::ReadString, state, input_ref)
        }
    };
}

pub fn submit_input(
    worker: &UseBridgeHandle<Worker>,
    input_ref: &UseStateHandle<NodeRef>,
    state: &UseStateHandle<State>,
) {
    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
        if let State::Running(curr) = &**state {
            use ReadSyscallInputs::*;
            use ReadSyscalls::*;
            match curr.input_needed.as_ref().unwrap_throw() {
                ReadInt => match input.value().parse::<i32>() {
                    Ok(num) => {
                        process_syscall_response(state.clone(), worker.clone(), input, Int(num));
                    }
                    Err(_e) => {
                        let error_msg =
                            format!("Failed to parse input '{}' as an i32", input.value());
                        error!("{}", error_msg);
                        let mut new_vec = curr.mips_state.mipsy_stdout.clone();
                        new_vec.push(error_msg);
                        let new_mips_state = MipsState {
                            mipsy_stdout: new_vec,
                            ..curr.mips_state.clone()
                        };
                        state.set(State::Running(RunningState {
                            mips_state: new_mips_state,
                            ..curr.clone()
                        }))
                    }
                },

                ReadFloat => match input.value().parse::<f32>() {
                    Ok(num) => {
                        process_syscall_response(state.clone(), worker.clone(), input, Float(num));
                    }

                    Err(_e) => {
                        let error_msg =
                            format!("Failed to parse input '{}' as an f32", input.value());
                        error!("{}", error_msg);
                        let mut new_vec = curr.mips_state.mipsy_stdout.clone();
                        new_vec.push(error_msg);
                        let new_mips_state = MipsState {
                            mipsy_stdout: new_vec,
                            ..curr.mips_state.clone()
                        };
                        state.set(State::Running(RunningState {
                            mips_state: new_mips_state,
                            ..curr.clone()
                        }))
                    }
                },

                ReadDouble => match input.value().parse::<f64>() {
                    Ok(num) => {
                        process_syscall_response(state.clone(), worker.clone(), input, Double(num));
                    }
                    Err(_e) => {
                        error!("Failed to parse input '{}' as an f64", input.value());
                    }
                },

                ReadChar => match input.value().parse::<char>() {
                    Ok(char) => process_syscall_response(
                        state.clone(),
                        worker.clone(),
                        input,
                        Char(char as u8),
                    ),
                    Err(_e) => {
                        let error_msg =
                            format!("Failed to parse input '{}' as an u8", input.value());
                        error!("{}", error_msg);
                        let mut new_vec = curr.mips_state.mipsy_stdout.clone();
                        new_vec.push(error_msg);
                        let new_mips_state = MipsState {
                            mipsy_stdout: new_vec,
                            ..curr.mips_state.clone()
                        };
                        state.set(State::Running(RunningState {
                            mips_state: new_mips_state,
                            ..curr.clone()
                        }))
                    }
                },

                ReadString => {
                    let string = format!("{}{}", input.value(), "\n").as_bytes().to_vec();
                    process_syscall_response(state.clone(), worker.clone(), input, String(string));
                }
            }
        } else {
            error!("Should not be able to submit with no file");
        }
    };
}
