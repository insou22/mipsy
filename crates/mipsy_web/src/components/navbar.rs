use crate::{
    pages::main::app::NUM_INSTR_BEFORE_RESPONSE,
    state::state::{DisplayedCodeTab, ErrorType, MipsState, RunningState, State},
    worker::{FileInformation, MipsyWebWorker, WorkerRequest},
};
use bounce::{use_atom, UseAtomHandle};
use derivative::Derivative;
use gloo_worker::{WorkerBridge, Worker};
use log::info;
use yew::{prelude::*, Html};

#[derive(Properties, Derivative)]
#[derivative(PartialEq)]
pub struct NavBarProps {
    #[prop_or_default]
    pub load_onchange: Callback<Event>,
    pub display_modal: UseStateHandle<bool>,
    pub settings_modal: UseStateHandle<bool>,
    pub file_loaded: bool,
    pub waiting_syscall: bool,
    pub is_saved: UseStateHandle<bool>,
    #[derivative(PartialEq = "ignore")]
    pub worker: UseStateHandle<WorkerBridge<MipsyWebWorker>>,
    pub filename: UseStateHandle<Option<String>>,
    pub file: UseStateHandle<Option<String>>,
    pub show_tab: UseStateHandle<DisplayedCodeTab>,
}

struct Icon {
    label: String,
    callback: Option<yew::Callback<yew::MouseEvent>>,
    title: String,
    html: Html,
    is_disabled: bool,
}

fn icons(props: &NavBarProps, state: &UseAtomHandle<State>) -> Vec<Icon> {
    let icons = vec![
        Icon {
            label: String::from("Save"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M17.85 3.15l-2.99-3A.508.508 0 0 0 14.5 0H1.4A1.417 1.417 0 0 0 0 1.43v15.14A1.417 1.417 0 0 0 1.4 18h15.2a1.417 1.417 0 0 0 1.4-1.43V3.5a.47.47 0 0 0-.15-.35zM2 5V3a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1zm7 11a4 4 0 1 1 4-4 4 4 0 0 1-4 4z"/>
                </svg>
            },
            title: String::from("Save and compile the current file"),
            callback: Some({
                let file = props.file.clone();
                let worker = props.worker.clone();
                let filename = props.filename.clone();
                let is_saved = props.is_saved.clone();
                Callback::from(move |_| {
                    info!("Save button clicked");
                    is_saved.set(true);
                    let updated_content = crate::get_editor_value();
                    let clone = updated_content.clone();
                    let filename = &filename.as_deref().unwrap_or("Untitled");
                    crate::set_localstorage_file_contents(&updated_content);
                    crate::set_localstorage_filename(filename);
                    file.set(Some(updated_content));
                    worker.send(WorkerRequest::CompileCode(FileInformation {
                        filename: filename.to_string(),
                        file: clone,
                    }));
                })
            }),
            is_disabled: false,
        },
        Icon {
            label: String::from("Run"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Run program"),
            callback: Some({
                let worker = props.worker.clone();
                let state = state.clone();
                let file = props.file.clone();
                let filename = props.filename.clone();
                let show_tab = props.show_tab.clone();
                Callback::from(move |_| {
                    info!("Run button clicked");
                    if let State::Compiled(ref curr) = *state {
                        state.set(State::Compiled(RunningState {
                            mips_state: MipsState {
                                is_stepping: false,
                                ..curr.mips_state.clone()
                            },
                            ..curr.clone()
                        }));
                        let file_information = FileInformation {
                            filename: filename.as_deref().unwrap_or("Untitled").to_string(),
                            file: file.as_deref().unwrap_or("").to_string(),
                        };
                        let input = <MipsyWebWorker as Worker>::Input::Run(
                            MipsState {
                                is_stepping: false,
                                ..curr.mips_state.clone()
                            },
                            NUM_INSTR_BEFORE_RESPONSE,
                            file_information,
                        );

                        show_tab.set(DisplayedCodeTab::Decompiled);

                        worker.send(input);
                    } else {
                        info!("No File loaded, cannot run");
                    };
                })
            }),
            is_disabled: true,
        },
        Icon {
            label: String::from("Reset"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                     <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Reset Runtime"),
            callback: Some({
                let worker = props.worker.clone();
                let state = state.clone();
                Callback::from(move |_| {
                    info!("Reset button clicked");

                    match &*state {
                        State::Compiled(curr) => {
                            state.set(State::Compiled(RunningState {
                                should_kill: false,
                                ..curr.clone()
                            }));
                            let input =
                                <MipsyWebWorker as Worker>::Input::ResetRuntime(curr.mips_state.clone());
                            worker.send(input);
                        }
                        State::Error(ErrorType::RuntimeError(curr)) => {
                            state.set(State::Compiled(RunningState {
                                should_kill: false,
                                decompiled: curr.decompiled.clone(),
                                mips_state: curr.mips_state.clone(),
                                input_needed: None,
                            }));
                            let input =
                                <MipsyWebWorker as Worker>::Input::ResetRuntime(curr.mips_state.clone());
                            worker.send(input);
                        }

                        State::Error(ErrorType::CompilerOrParserError(_)) => {
                            // shouldnt be able to click
                            unreachable!("Reset button should not be clickable when compiler or parser error");
                        }

                        State::NoFile => {
                            unreachable!("Reset button should not be clickable when no file");
                        }
                    }
                })
            }),
            is_disabled: match **state {
                State::Compiled(_) => false,
                State::Error(ErrorType::RuntimeError(_)) => false,
                State::Error(ErrorType::CompilerOrParserError(_)) => true,
                State::NoFile => true,
            },
        },
        Icon {
            label: String::from("Kill"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Stop executing program"),
            callback: Some({
                let state = state.clone();
                Callback::from(move |_| {
                    info!("Kill button clicked");
                    if let State::Compiled(curr) = &*state {
                        state.set(State::Compiled(RunningState {
                            should_kill: true,
                            ..curr.clone()
                        }));
                    }
                })
            }),
            is_disabled: true,
        },
        Icon {
            label: String::from("Step Back"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Step backwards"),
            callback: Some({
                let worker = props.worker.clone();
                let state = state.clone();
                let file = props.file.clone();
                let filename = props.filename.clone();
                Callback::from(move |_| {
                    info!("Step Back button clicked");
                    if let State::Compiled(curr) = &*state {
                        let new_mips_state = MipsState {
                            is_stepping: true,
                            ..curr.mips_state.clone()
                        };

                        state.set(State::Compiled(RunningState {
                            mips_state: new_mips_state.clone(),
                            ..curr.clone()
                        }));
                        let file_information = FileInformation {
                            filename: filename.as_deref().unwrap_or("Untitled").to_string(),
                            file: file.as_deref().unwrap_or("").to_string(),
                        };
                        let input =
                            <MipsyWebWorker as Worker>::Input::Run(new_mips_state, -1, file_information);
                        worker.send(input);
                    } else {
                        info!("No File loaded, cannot step");
                    };
                })
            }),
            is_disabled: true,
        },
        Icon {
            label: String::from("Step Next"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Step forwards"),
            callback: Some({
                let worker = props.worker.clone();
                let state = state.clone();
                let file = props.file.clone();
                let filename = props.filename.clone();
                Callback::from(move |_| {
                    info!("Step Next button clicked");
                    if let State::Compiled(curr) = &*state {
                        let new_mips_state = MipsState {
                            is_stepping: true,
                            ..curr.mips_state.clone()
                        };

                        state.set(State::Compiled(RunningState {
                            mips_state: new_mips_state.clone(),
                            ..curr.clone()
                        }));
                        let file_information = FileInformation {
                            filename: filename.as_deref().unwrap_or("Untitled").to_string(),
                            file: file.as_deref().unwrap_or("").to_string(),
                        };
                        let input =
                            <MipsyWebWorker as Worker>::Input::Run(new_mips_state, 1, file_information);
                        worker.send(input);
                    } else {
                        info!("No File loaded, cannot step");
                    };
                })
            }),
            is_disabled: true,
        },
        Icon {
            label: String::from("Download"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 20 20" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>
            },
            title: String::from("Download the current saved file"),
            callback: Some({
                let filename = props.filename.clone();
                let file = props.file.clone();
                Callback::from(move |_| {
                    info!("Download button clicked");
                    crate::trigger_download_file(
                        filename.as_deref().unwrap_or("untitled.s"),
                        file.as_deref().unwrap_or(""),
                    );
                })
            }),
            is_disabled: false,
        },
    ];

    icons
}

#[function_component(NavBar)]
pub fn render_navbar(props: &NavBarProps) -> Html {
    let state = use_atom::<State>();
    let icons = icons(props.clone(), &state);
    let exit_status = match &*state {
        State::Compiled(curr) => Some(curr.mips_state.exit_status),
        _ => None,
    };

    html! {
        <nav class="flex items-center justify-between flex-wrap bg-th-primary p-4">
          <div class="flex items-center flex-shrink-0  mr-6">
            <span class="font-semibold text-xl tracking-tight">{"mipsy web"}</span>
          </div>
          <div class="w-full block flex-grow flex items-center w-auto">
            <div class="flex-grow flex flex-row">
              <label tabindex=0 for="load_file" class="mr-2 text-sm flex place-items-center flex-row inline-block cursor-pointer px-3 py-3 leading-none border rounded border-current hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1H8a3 3 0 00-3 3v1.5a1.5 1.5 0 01-3 0V6z" clip-rule="evenodd" />
                  <path d="M6 12a2 2 0 012-2h8a2 2 0 012 2v2a2 2 0 01-2 2H2h2a2 2 0 002-2v-2z" />
                </svg>
                {"Load"}
              </label>
              <input id="load_file" onchange={&props.load_onchange} type="file" accept=".s" class="hidden" />
                {
                    for icons.iter().map(|item| {

                        // run or step buttons should be disabled
                        // if we have exited
                        let is_run_step_disabled = if item.label == "Run" || item.label == "Step Next" {
                            matches!(&exit_status, Some(Some(_)))
                        } else {
                            false
                        };

                        // if we are waiting on a syscall value, or if there is no file
                        // then we hsouldn't be able to step
                        let is_disabled = item.is_disabled && (props.waiting_syscall || !props.file_loaded || is_run_step_disabled);

                        let onclick = if item.callback.is_some() {
                            item.callback.clone().unwrap()
                        } else {
                            Callback::from(|_| {})
                        };

                        let mut button_classes = String::from("mr-2 flex place-items-center flex-row inline-block cursor-pointer \
                                                               text-sm px-2 py-2 border rounded border-current \
                                                               hover:border-transparent hover:text-teal-500 hover:bg-white");

                        let title = if is_disabled {
                            // space at front is needed otherwise classes will combine
                            button_classes.push_str(" opacity-50 cursor-not-allowed");
                            if props.waiting_syscall {
                                String::from("enter syscall value")
                            } else if !props.file_loaded {
                                String::from("please load file")
                            } else {
                                String::from("cannot step past end of program")
                            }

                        } else {
                            item.title.clone()
                        };

                        html! {
                            <button tabindex=0 {title} disabled={is_disabled} {onclick} class={button_classes}>
                                { item.html.clone() }
                                { item.label.clone() }
                            </button>
                        }
                    })
                }
            </div>
            <a
                href="https://cgi.cse.unsw.edu.au/~cs1521/current/resources/mips-guide.html"
                target="_blank"
                class="mr-2 flex place-items-center flex-row inline-block cursor-pointer \
                       text-sm px-2 py-2 border rounded button border-current \
                       hover:border-transparent hover:text-teal-500 hover:bg-white"
            >
                {"MIPS Docs"}
            </a>
            <button
                onclick={{
                    let display_modal = props.display_modal.clone();
                    Callback::from(move |_| {
                        display_modal.set(!*display_modal);
                    })
                }}
                class="mr-2 flex place-items-center flex-row inline-block cursor-pointer \
                       text-sm px-2 py-2 border rounded border-current \
                       hover:border-transparent hover:text-teal-500 hover:bg-white"
            >
                {"About"}
            </button>
            <button
                onclick={{
                    let settings_modal = props.settings_modal.clone();
                    Callback::from(move |_| {
                        settings_modal.set(!*settings_modal);
                    })
                }}
                class="hover:bg-white border rounded border-current hover:border-transparent px-2 py-2 focus:outline-2 focus:outline-dashed" id="settings_button"
            >
                <svg aria-hidden="true" fill="#000000" tabindex="0"
                     xmlns="http://www.w3.org/2000/svg"  viewBox="0 0 24 24"
                     width="24px" height="24px" stroke="currentColor"
                >
                    <path d="M 9.6660156 2 L 9.1757812 4.5234375 C 8.3516137 4.8342536 7.5947862 5.2699307 6.9316406 5.8144531 L 4.5078125 4.9785156 L 2.171875 9.0214844 L 4.1132812 10.708984 C 4.0386488 11.16721 4 11.591845 4 12 C 4 12.408768 4.0398071 12.832626 4.1132812 13.291016 L 4.1132812 13.292969 L 2.171875 14.980469 L 4.5078125 19.021484 L 6.9296875 18.1875 C 7.5928951 18.732319 8.3514346 19.165567 9.1757812 19.476562 L 9.6660156 22 L 14.333984 22 L 14.824219 19.476562 C 15.648925 19.165543 16.404903 18.73057 17.068359 18.185547 L 19.492188 19.021484 L 21.826172 14.980469 L 19.886719 13.291016 C 19.961351 12.83279 20 12.408155 20 12 C 20 11.592457 19.96113 11.168374 19.886719 10.710938 L 19.886719 10.708984 L 21.828125 9.0195312 L 19.492188 4.9785156 L 17.070312 5.8125 C 16.407106 5.2676813 15.648565 4.8344327 14.824219 4.5234375 L 14.333984 2 L 9.6660156 2 z M 11.314453 4 L 12.685547 4 L 13.074219 6 L 14.117188 6.3945312 C 14.745852 6.63147 15.310672 6.9567546 15.800781 7.359375 L 16.664062 8.0664062 L 18.585938 7.40625 L 19.271484 8.5917969 L 17.736328 9.9277344 L 17.912109 11.027344 L 17.912109 11.029297 C 17.973258 11.404235 18 11.718768 18 12 C 18 12.281232 17.973259 12.595718 17.912109 12.970703 L 17.734375 14.070312 L 19.269531 15.40625 L 18.583984 16.59375 L 16.664062 15.931641 L 15.798828 16.640625 C 15.308719 17.043245 14.745852 17.36853 14.117188 17.605469 L 14.115234 17.605469 L 13.072266 18 L 12.683594 20 L 11.314453 20 L 10.925781 18 L 9.8828125 17.605469 C 9.2541467 17.36853 8.6893282 17.043245 8.1992188 16.640625 L 7.3359375 15.933594 L 5.4140625 16.59375 L 4.7285156 15.408203 L 6.265625 14.070312 L 6.0878906 12.974609 L 6.0878906 12.972656 C 6.0276183 12.596088 6 12.280673 6 12 C 6 11.718768 6.026742 11.404282 6.0878906 11.029297 L 6.265625 9.9296875 L 4.7285156 8.59375 L 5.4140625 7.40625 L 7.3359375 8.0683594 L 8.1992188 7.359375 C 8.6893282 6.9567546 9.2541467 6.6314701 9.8828125 6.3945312 L 10.925781 6 L 11.314453 4 z M 12 8 C 9.8034768 8 8 9.8034768 8 12 C 8 14.196523 9.8034768 16 12 16 C 14.196523 16 16 14.196523 16 12 C 16 9.8034768 14.196523 8 12 8 z M 12 10 C 13.111477 10 14 10.888523 14 12 C 14 13.111477 13.111477 14 12 14 C 10.888523 14 10 13.111477 10 12 C 10 10.888523 10.888523 10 12 10 z"/>
                </svg>
            </button>
          </div>
        </nav>

    }
}
