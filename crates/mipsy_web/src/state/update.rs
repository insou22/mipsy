use crate::{
    pages::main::app::{
        process_syscall_request, process_syscall_response, ReadSyscalls, NUM_INSTR_BEFORE_RESPONSE,
    },
    state::state::{DisplayedTab, MipsState, RunningState, State},
    utils::generate_highlighted_line,
    worker::{Worker, WorkerRequest, ReadSyscallInputs, WorkerResponse},
};
use log::{error, info};

use super::state::ErrorState;
use gloo_console::log;
use mipsy_lib::{MipsyError, Safe};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::UseBridgeHandle;

pub fn handle_response_from_worker(
    state: UseStateHandle<State>,
    show_tab: UseStateHandle<DisplayedTab>,
    show_io: UseStateHandle<bool>,
    file: UseStateHandle<Option<String>>,
    response: WorkerResponse,
    worker: Rc<RefCell<Option<UseBridgeHandle<Worker>>>>,
    input_ref: UseStateHandle<NodeRef>,
    is_saved: UseStateHandle<bool>,
) {
    match response {
        WorkerResponse::DecompiledCode(response_struct) => {
            log!("recieved decompiled code from worker");
            state.set(State::Compiled(RunningState {
                decompiled: response_struct.decompiled,
                mips_state: MipsState {
                    stdout: Vec::new(),
                    exit_status: None,
                    register_values: vec![Safe::Uninitialised; 32],
                    previous_registers: vec![Safe::Uninitialised; 32],
                    current_instr: None,
                    mipsy_stdout: Vec::new(),
                    memory: HashMap::new(),
                    is_stepping: true,
                },
                input_needed: None,
                should_kill: false,
            }));
            if response_struct.file.is_some() {
                file.set(Some(response_struct.file.clone().unwrap()));
                show_tab.set(DisplayedTab::Source);
                crate::set_editor_value(&response_struct.file.clone().unwrap());
                crate::set_localstorage_file_contents(&response_struct.file.unwrap());
                is_saved.set(true);
            }
        }

        WorkerResponse::WorkerError(response_struct) => {
            log!("recieved compiler error from worker");
            let mut mipsy_stdout = vec![];

            match response_struct.error {
                MipsyError::Compiler(ref compiler_err) => {
                    mipsy_stdout.push(format!(
                        "{}\n{}\n{}",
                        generate_highlighted_line(response_struct.file.clone(), compiler_err),
                        compiler_err.error().message(),
                        compiler_err.error().tips().join("\n")
                    ));
                    show_io.set(false);
                }
                MipsyError::Parser(_) => {
                    mipsy_stdout.push("failed to parse".into());
                    show_io.set(false);
                }
                _ => {
                    mipsy_stdout.push("Runtime errors not yet supported".into());
                }
            };

            let state_struct = ErrorState {
                error: response_struct.error.clone(),
                mipsy_stdout,
            };

            file.set(Some(response_struct.file.clone()));
            show_tab.set(DisplayedTab::Source);
            state.set(State::Error(state_struct));
        }

        WorkerResponse::ProgramExited(mips_state) => {
            if let State::Compiled(ref curr) = *state {
                state.set(State::Compiled(RunningState {
                    mips_state,
                    ..curr.clone()
                }));
            }
        }

        WorkerResponse::InstructionOk(mips_state) => {
            if let State::Compiled(ref curr) = *state {
                state.set(State::Compiled(RunningState {
                    mips_state: mips_state.clone(),
                    ..curr.clone()
                }));

                // if the isntruction was ok, run another instruction
                // unless the user has said it should be killed
                if !curr.should_kill {
                    let input = WorkerRequest::Run(mips_state.clone(), NUM_INSTR_BEFORE_RESPONSE);
                    worker.borrow().as_ref().unwrap().send(input);
                }

                state.set(State::Compiled(RunningState {
                    should_kill: false,
                    mips_state,
                    ..curr.clone()
                }));
            } else {
                info!("No File loaded, cannot run");
            }
        }

        WorkerResponse::UpdateMipsState(mips_state) => {
            if let State::Compiled(ref curr) = *state {
                state.set(State::Compiled(RunningState {
                    mips_state,
                    ..curr.clone()
                }));
                info!("updating state");
            }
        }

        WorkerResponse::NeedInt(mips_state) => {
            process_syscall_request(mips_state, ReadSyscalls::ReadInt, state, input_ref, show_io)
        }
        WorkerResponse::NeedFloat(mips_state) => process_syscall_request(
            mips_state,
            ReadSyscalls::ReadFloat,
            state,
            input_ref,
            show_io,
        ),
        WorkerResponse::NeedDouble(mips_state) => process_syscall_request(
            mips_state,
            ReadSyscalls::ReadDouble,
            state,
            input_ref,
            show_io,
        ),
        WorkerResponse::NeedChar(mips_state) => process_syscall_request(
            mips_state,
            ReadSyscalls::ReadChar,
            state,
            input_ref,
            show_io,
        ),
        WorkerResponse::NeedString(mips_state) => process_syscall_request(
            mips_state,
            ReadSyscalls::ReadString,
            state,
            input_ref,
            show_io,
        ),
    };
}

pub fn submit_input(
    worker: &UseBridgeHandle<Worker>,
    input_ref: &UseStateHandle<NodeRef>,
    state: &UseStateHandle<State>,
) {
    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
        if let State::Compiled(curr) = &**state {
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
                        state.set(State::Compiled(RunningState {
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
                        state.set(State::Compiled(RunningState {
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
                        state.set(State::Compiled(RunningState {
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

// same default values for EOF as crates/mipsy/src/main.rs
pub fn submit_eof(
    worker: &UseBridgeHandle<Worker>,
    input_ref: &UseStateHandle<NodeRef>,
    state: &UseStateHandle<State>,
) {
    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
        if let State::Compiled(curr) = &**state {
            use ReadSyscallInputs::*;
            use ReadSyscalls::*;
            match curr.input_needed.as_ref().unwrap_throw() {
                ReadInt => {
                    process_syscall_response(state.clone(), worker.clone(), input, Int(0));
                }

                ReadFloat => {
                    process_syscall_response(state.clone(), worker.clone(), input, Double(0.0));
                }

                ReadDouble => {
                    process_syscall_response(state.clone(), worker.clone(), input, Double(0.0));
                }

                ReadChar => {
                    process_syscall_response(state.clone(), worker.clone(), input, Char('\0' as u8))
                }

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
