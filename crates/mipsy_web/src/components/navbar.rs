use crate::{
    pages::main::{app::NUM_INSTR_BEFORE_RESPONSE, state::State},
    worker::Worker,
};
use derivative::Derivative;
use log::{info, trace};
use yew::prelude::*;
use yew::Html;
use yew_agent::{Agent, UseBridgeHandle};

#[derive(Properties, Derivative)]
#[derivative(PartialEq)]
pub struct NavBarProps {
    #[prop_or_default]
    pub load_onchange: Callback<Event>,
    pub display_modal: UseStateHandle<bool>,
    pub file_loaded: bool,
    pub waiting_syscall: bool,
    pub state: UseStateHandle<State>,

    #[derivative(PartialEq = "ignore")]
    pub worker: UseBridgeHandle<Worker>,
}

struct Icon {
    label: String,
    callback: Option<yew::Callback<yew::MouseEvent>>,
    title: String,
    html: Html,
}

fn icons(props: &NavBarProps) -> Vec<Icon> {
    let icons = vec![
        Icon {
            label: String::from("Run"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Run program"),
            callback: Some({
                let worker = props.worker.clone();
                let state = props.state.clone();
                Callback::from(move |_| {
                    trace!("Run button clicked");
                    if let State::Running(ref curr) = *state {
                        todo!("replace state");
                        curr.mips_state.is_stepping = false;
                        let input = <Worker as Agent>::Input::Run(
                            curr.mips_state.clone(),
                            NUM_INSTR_BEFORE_RESPONSE,
                        );
                        worker.send(input);
                    } else {
                        info!("No File loaded, cannot run");
                    };
                })
            }),
        },
        Icon {
            label: String::from("Reset"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                     <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Reset Runtime"),
            callback: Some({
                let worker = props.worker.clone();
                let state = props.state.clone();
                Callback::from(move |_| {
                    trace!("Reset button clicked");
                    if let State::Running(curr) = &*state {
                        let input = <Worker as Agent>::Input::ResetRuntime(curr.mips_state.clone());
                        worker.send(input);
                    } else {
                        state.set(State::NoFile);
                    }
                })
            }),
        },
        Icon {
            label: String::from("Kill"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Stop executing program"),
            callback: Some({
                let state = props.state.clone();
                Callback::from(move |_| {
                    trace!("Kill button clicked");
                    if let State::Running(curr) = &*state {
                        todo!("replace state");
                        curr.should_kill = true;
                    }
                })
            }),
        },
        Icon {
            label: String::from("Step Back"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Step backwards"),
            callback: Some({
                let worker = props.worker.clone();
                let state = props.state.clone();
                Callback::from(move |_| {
                    trace!("Step Back button clicked");
                    if let State::Running(curr) = &*state {
                        todo!("Replace state");
                        curr.mips_state.is_stepping = true;
                        let input = <Worker as Agent>::Input::Run(curr.mips_state.clone(), -1);
                        worker.send(input);
                    } else {
                        info!("No File loaded, cannot step");
                    };
                })
            }),
        },
        Icon {
            label: String::from("Step Next"),
            html: html! {
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd" />
                </svg>
            },
            title: String::from("Step forwards"),
            callback: Some({
                let worker = props.worker.clone();
                let state = props.state.clone();
                Callback::from(move |_| {
                    trace!("Step Back button clicked");
                    if let State::Running(curr) = &*state {
                        todo!("replace state");
                        curr.mips_state.is_stepping = true;
                        let input = <Worker as Agent>::Input::Run(curr.mips_state.clone(), 1);
                        worker.send(input);
                    } else {
                        info!("No File loaded, cannot step");
                    };
                })
            }),
        },
    ];

    icons
}

#[function_component(NavBar)]
pub fn render_navbar(props: &NavBarProps) -> Html {
    let icons = icons(props.clone());
    let exit_status = match &*props.state {
        State::Running(curr) => Some(curr.mips_state.exit_status),
        _ => None,
    };

    html! {
        <nav class="flex items-center justify-between flex-wrap bg-th-primary p-4">
          <div class="flex items-center flex-shrink-0 text-black mr-6">
            <span class="font-semibold text-xl tracking-tight">{"Mipsy"}</span>
          </div>
          <div class="w-full block flex-grow flex items-center w-auto">
            <div class="flex-grow flex flex-row">
              <label tabindex=0 for="load_file" class="mr-2 text-sm flex place-items-center flex-row inline-block cursor-pointer px-3 py-3 leading-none border rounded text-black border-black hover:border-transparent hover:text-teal-500 hover:bg-white">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
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
                            if let Some(Some(_)) = &exit_status {
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        // if we are waiting on a syscall value, or if there is no file
                        // then we hsouldn't be able to step
                        let is_disabled = props.waiting_syscall || !props.file_loaded || is_run_step_disabled;

                        let onclick = if item.callback.is_some() {
                            item.callback.clone().unwrap()
                        } else {
                            Callback::from(|_| {})
                        };

                        let mut button_classes = String::from("mr-2 flex place-items-center flex-row inline-block cursor-pointer \
                                                               text-sm px-2 py-2 border rounded text-black border-black \
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
            <button
                onclick={{
                    let display_modal = props.display_modal.clone();
                    Callback::from(move |_| {
                        display_modal.set(!*display_modal);
                    })
                }}
                class="mr-2 flex place-items-center flex-row inline-block cursor-pointer \
                       text-sm px-2 py-2 border rounded text-black border-black \
                       hover:border-transparent hover:text-teal-500 hover:bg-white"
            >
                {"About Mipsy Web"}
            </button>
          </div>
        </nav>

    }
}
