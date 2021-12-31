use crate::worker::ReadSyscallInputs;
use crate::worker::{WorkerRequest, WorkerResponse};
use gloo_file::callbacks::read_as_text;
use log::{error, info, trace};
use mipsy_lib::{MipsyError, Safe};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use std::ops::DerefMut;
use std::rc::Rc;
use std::cell::RefCell;
use crate::pages::main::{
    app::{App, Msg, ReadSyscalls, State, NUM_INSTR_BEFORE_RESPONSE},
    state::{MipsState, RunningState},
};

pub fn handle_update(app: &mut App, ctx: &Context<App>, msg: <App as Component>::Message) -> bool {
    match msg {
        Msg::FileChanged(file) => {
            info!("file changed msg");
            // FIXME -- check result
            {
                let file_name = file.name();
                let link = ctx.link().clone();
                info!("file name: {}", file_name);
                app.tasks.push(read_as_text(&file, move |res| {
                    link.send_message(Msg::FileRead(file_name, res))
                }));
            }

            true
        }

        Msg::FileRead(filename, res) => {
            info!("file Read msg");
            match res {
                Ok(ref file) => {
                    app.filename = Some(filename);
                    app.file = Some(file.to_string());
                    let input = WorkerRequest::CompileCode(file.to_string());
                    info!("sending to worker");
                    app.worker.send(input);
                    app.show_source = false;
                }
                Err(_e) => {}
            }

            true
        }

        Msg::Run => {
            trace!("Run button clicked");
            if let State::Running(ref mut curr) = app.state {
                curr.mips_state.is_stepping = false;
                let input = WorkerRequest::Run(curr.mips_state.clone(), NUM_INSTR_BEFORE_RESPONSE);
                app.worker.send(input);
            } else {
                info!("No File loaded, cannot run");
                return false;
            }
            true
        }

        Msg::Kill => {
            trace!("Kill button clicked");
            if let State::Running(ref mut curr) = app.state{
                curr.should_kill = true;
            };
            true
        }

        Msg::OpenModal => {
            app.display_modal = !app.display_modal;
            true
        }

        Msg::ShowIoTab => {
            trace!("Show IO Button clicked");
            // only re-render upon change
            let prev_show = app.show_io;
            app.show_io = true;
            prev_show != true
        }

        Msg::ShowMipsyTab => {
            trace!("Show mipsy button clicked");
            // only re-render upon change
            let prev_show = app.show_io;
            app.show_io = false;
            prev_show != false
        }
        Msg::ShowSourceTab => {
            trace!("Show source button clicked");
            // only re-render upon change
            app.show_source = true;
            true
        }

        Msg::ShowDecompiledTab => {
            trace!("Show decompiled button clicked");
            app.show_source = false;
            true
        }

        Msg::StepForward => {
            trace!("Step forward button clicked");
            if let State::Running(ref mut curr) = app.state {
                curr.mips_state.is_stepping = true;
                let input = WorkerRequest::Run(curr.mips_state.clone(), 1);
                app.worker.send(input);
            } else {
                info!("No File loaded, cannot step");
                return false;
            }
            true
        }

        Msg::StepBackward => {
            trace!("Step backward button clicked");
            if let State::Running(ref mut curr) = app.state {
                curr.mips_state.is_stepping = true;
                let input = WorkerRequest::Run(curr.mips_state.clone(), -1);
                app.worker.send(input);
            } else {
                info!("No File loaded, cannot step");
                return false;
            }
            true
        }

        Msg::Reset => {
            trace!("Reset button clicked");
            if let State::Running(curr) = &app.state {
                let input = WorkerRequest::ResetRuntime(curr.mips_state.clone());
                app.worker.send(input);
            } else {
                app.state = State::NoFile;
            }
            true
        }

        Msg::ProcessKeypress(event) => {
            if app.is_nav_or_special_key(&event) {
                return true;
            };
            info!("processing {}", event.key());
            true
        }

        Msg::SubmitInput => {
            if let Some(input) = app.input_ref.cast::<HtmlInputElement>() {
                if let State::Running(ref mut curr) = app.state {
                    use ReadSyscallInputs::*;
                    use ReadSyscalls::*;
                    match curr.input_needed.as_ref().unwrap_throw() {
                        ReadInt => match input.value().parse::<i32>() {
                            Ok(num) => {
                                App::process_syscall_response(app, input, Int(num));
                            }
                            Err(_e) => {
                                let error_msg =
                                    format!("Failed to parse input '{}' as an i32", input.value());
                                error!("{}", error_msg);
                                curr.mips_state.mipsy_stdout.push(error_msg);
                            }
                        },

                        ReadFloat => match input.value().parse::<f32>() {
                            Ok(num) => {
                                App::process_syscall_response(app, input, Float(num));
                            }

                            Err(_e) => {
                                let error_msg =
                                    format!("Failed to parse input '{}' as an f32", input.value());
                                error!("{}", error_msg);
                                curr.mips_state.mipsy_stdout.push(error_msg);
                            }
                        },

                        ReadDouble => match input.value().parse::<f64>() {
                            Ok(num) => {
                                App::process_syscall_response(app, input, Double(num));
                            }
                            Err(_e) => {
                                error!("Failed to parse input '{}' as an f64", input.value());
                            }
                        },

                        ReadChar => match input.value().parse::<char>() {
                            Ok(char) => {
                                App::process_syscall_response(app, input, Char(char as u8))
                            }
                            Err(_e) => {
                                let error_msg =
                                    format!("Failed to parse input '{}' as an u8", input.value());
                                error!("{}", error_msg);
                                curr.mips_state.mipsy_stdout.push(error_msg);
                            }
                        },

                        ReadString => {
                            let string = format!("{}{}", input.value(), "\n").as_bytes().to_vec();
                            App::process_syscall_response(app, input, String(string));
                        }
                    }
                } else {
                    error!("Should not be able to submit with no file");
                }
            };
            true
        }

        Msg::FromWorker(worker_output) => match worker_output {
            WorkerResponse::DecompiledCode(decompiled) => {
                info!("recieved decompiled code from worker");
                app.state = State::Running(RunningState {
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
                });
                true
            }

            WorkerResponse::CompilerError(err) => {
                match &err {
                    MipsyError::Parser(_error) => {
                        //zkol TODO
                        true
                    }

                    MipsyError::Compiler(error) => {
                        match app.state {
                            State::Running(ref mut curr) => {
                                curr.mips_state.mipsy_stdout.push(error.error().message())
                            }
                            State::NoFile | State::CompilerError(_) => {
                                app.state = State::CompilerError(err);
                                error!("Compiler errors are not supported.");
                            }
                        }
                        true
                    }

                    MipsyError::Runtime(_error) => {
                        error!("Cannot get runtime error at compile time");
                        unreachable!();
                    }
                }
            }

            WorkerResponse::ProgramExited(mips_state) => match app.state {
                State::Running(ref mut curr) => {
                    curr.mips_state = mips_state;
                    true
                }
                State::NoFile | State::CompilerError(_) => false,
            },

            WorkerResponse::InstructionOk(mips_state) => {
                if let State::Running(ref mut curr) = app.state {
                    info!("{:?}", mips_state);
                    info!("HERE");
                    curr.mips_state = mips_state;
                    // if the isntruction was ok, run another instruction
                    // unless the user has said it should be killed
                    if !curr.should_kill {
                        let input =
                            WorkerRequest::Run(curr.mips_state.clone(), NUM_INSTR_BEFORE_RESPONSE);
                        app.worker.send(input);
                    }
                    curr.should_kill = false;
                } else {
                    info!("No File loaded, cannot run");
                    return false;
                }
                true
            }

            WorkerResponse::UpdateMipsState(mips_state) => match app.state {
                State::Running(ref mut curr) => {
                    curr.mips_state = mips_state;
                    info!("updating state");
                    true
                }

                State::NoFile | State::CompilerError(_) => false,
            },

            WorkerResponse::NeedInt(mips_state) => {
                App::process_syscall_request(app, mips_state, ReadSyscalls::ReadInt)
            }
            WorkerResponse::NeedFloat(mips_state) => {
                App::process_syscall_request(app, mips_state, ReadSyscalls::ReadFloat)
            }
            WorkerResponse::NeedDouble(mips_state) => {
                App::process_syscall_request(app, mips_state, ReadSyscalls::ReadDouble)
            }
            WorkerResponse::NeedChar(mips_state) => {
                App::process_syscall_request(app, mips_state, ReadSyscalls::ReadChar)
            }
            WorkerResponse::NeedString(mips_state) => {
                App::process_syscall_request(app, mips_state, ReadSyscalls::ReadString)
            }
        },
    }
}
