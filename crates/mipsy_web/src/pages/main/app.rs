use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::worker::ReadSyscallInputs;
use crate::{
    components::{
        decompiled::DecompiledCode, modal::Modal, navbar::NavBar, outputarea::OutputArea,
        pagebackground::PageBackground, registers::Registers, sourcecode::SourceCode,
    },
    pages::main::{
        state::{MipsState, RunningState, State},
        update,
    },
    worker::{Worker, WorkerRequest, WorkerResponse},
};
use gloo_file::callbacks::{read_as_text, FileReader};
use gloo_file::File;
use gloo_file::FileReadError;
use log::{error, info, trace};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{use_bridge, Bridge, Bridged, UseBridgeHandle};

#[derive(Clone, Debug, PartialEq)]
pub enum ReadSyscalls {
    ReadInt,
    ReadFloat,
    ReadDouble,
    ReadString,
    ReadChar,
}

pub enum Msg {
    FileChanged(File),
    FileRead(String, Result<String, FileReadError>),
    Run,
    Reset,
    Kill,
    OpenModal,
    ShowIoTab,
    ShowMipsyTab,
    ShowSourceTab,
    ShowDecompiledTab,
    StepForward,
    StepBackward,
    SubmitInput,
    ProcessKeypress(KeyboardEvent),
    FromWorker(WorkerResponse),
    NoOp,
}

pub const NUM_INSTR_BEFORE_RESPONSE: i32 = 40;

/*fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        unsafe {
            crate::highlight();
        }
}
*/

#[function_component(App)]
pub fn render_app() -> Html {
    /* State Handlers */
    let state: UseStateHandle<State> = use_state_eq(|| State::NoFile);
    let force_rerender_toggle: UseStateHandle<bool> = use_state_eq(|| false);

    let worker = Rc::new(RefCell::new(None));

    let display_modal: UseStateHandle<bool> = use_state_eq(|| false);
    let show_io: UseStateHandle<bool> = use_state_eq(|| false);
    let input_ref: UseStateHandle<NodeRef> = use_state_eq(|| NodeRef::default());
    let filename: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let file: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let show_source: UseStateHandle<bool> = use_state_eq(|| false);
    let tasks: UseStateHandle<Vec<FileReader>> = use_state(|| vec![]);

    if worker.borrow().is_none() {
        *worker.borrow_mut() = {
            let state = state.clone();
            let force_rerender_toggle = force_rerender_toggle.clone();
            let input_ref = input_ref.clone();
            let worker = worker.clone();

            Some(use_bridge(move |response| {
                let state = state.clone();
                let force_rerender_toggle = force_rerender_toggle.clone();
                let input_ref = input_ref.clone();
                let worker = worker.clone();
                update::handle_response_from_worker(
                    state,
                    response,
                    force_rerender_toggle,
                    worker,
                    input_ref,
                )
            }))
        };
    }
    // TODO - figure out rendered fn

    /*    CALLBACKS   */
    let load_onchange: Callback<Event> = {
        let worker = worker.clone();
        let filename = filename.clone();
        let file = file.clone();
        let show_source = show_source.clone();
        let tasks = tasks.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Some(file_list) = input.files() {
                if let Some(file_blob) = file_list.item(0) {
                    let gloo_file = File::from(web_sys::File::from(file_blob));

                    let file_name = gloo_file.name();
                    filename.set(Some(file_name));

                    // prep items for closure below
                    let file = file.clone();
                    let worker = worker.clone();
                    let show_source = show_source.clone();

                    let mut tasks_new = vec![];
                    tasks_new.push(read_as_text(&gloo_file, move |res| match res {
                        Ok(ref file_contents) => {
                            file.set(Some(file_contents.to_string()));
                            let input = WorkerRequest::CompileCode(file_contents.to_string());
                            info!("sending to worker");
                            worker.borrow().as_ref().unwrap().send(input);
                            show_source.set(false);
                        }

                        Err(_e) => {}
                    }));

                    tasks.set(tasks_new);
                }
            }
        })
    };

    let on_input_keydown: Callback<KeyboardEvent> = {
        let worker = worker.clone();
        let state = state.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                update::submit_input(worker.borrow().as_ref().unwrap(), &input_ref, &state);
            };
        })
    };

    /* HELPER FNS */
    let text_html_content = match &*state {
        State::NoFile => "no file loaded".into(),
        State::CompilerError(_error) => "File has syntax errors, check your file with mipsy and try again.\nMipsy Web does not yet support displaying compiler errors".into(),
        State::Running(curr) => render_running(file.clone(), curr.clone(), filename.clone(), show_source.clone())
    };

    trace!("rendering");

    let modal_overlay_classes = if *display_modal {
        "bg-th-secondary bg-opacity-90 absolute top-0 left-0 h-screen w-screen"
    } else {
        "hidden"
    };

    let file_loaded = match *state {
        State::NoFile | State::CompilerError(_) => false,
        State::Running(_) => true,
    };

    let waiting_syscall = match &*state {
        State::Running(curr) => curr.input_needed.is_some(),
        State::NoFile | State::CompilerError(_) => false,
    };

    // TODO - make this nicer when refactoring compiler errs
    let mipsy_output_tab_title = match &*state {
        State::NoFile => "Mipsy Output - (0)".to_string(),
        State::CompilerError(_) => "Mipsy Output - (1)".to_string(),
        State::Running(curr) => {
            format!("Mipsy Output - ({})", curr.mips_state.mipsy_stdout.len())
        }
    };

    let (decompiled_tab_classes, source_tab_classes) = {
        let mut default = (
            String::from("w-1/2 leading-none hover:bg-white float-left border-t-2 border-r-2 border-black cursor-pointer px-1"),
            String::from("w-1/2 leading-none hover:bg-white float-left border-t-2 border-r-2 border-l-2 border-black cursor-pointer px-1 ")
        );

        if *show_source {
            default.1 = format!("{} {}", &default.1, String::from("bg-th-tabclicked"));
        } else {
            default.0 = format!("{} {}", &default.0, String::from("bg-th-tabclicked"));
        };

        default
    };

    let input_needed = match &*state {
        State::Running(curr) => curr.input_needed.clone(),
        State::NoFile | State::CompilerError(_) => None,
    };

    html! {
        <>
            <div onclick={{
                let display_modal = display_modal.clone();
                Callback::from(move |_| {
                display_modal.set(!*display_modal);
            })}} class={modal_overlay_classes}></div>

            <Modal should_display={display_modal.clone()} />

            <PageBackground>

                <NavBar
                    {load_onchange}
                    display_modal={display_modal.clone()}
                    {file_loaded}
                    {waiting_syscall}
                    state={state.clone()}
                    worker={worker.borrow().as_ref().unwrap().clone()}
                />

                <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 122px)">
                    <div id="file_data">
                        <div style="height: 4%;" class="flex overflow-hidden border-1 border-black">
                            <button class={source_tab_classes} onclick={{
                                let show_source = show_source.clone();
                                Callback::from(move |_| {
                                    show_source.set(!*show_source);
                                })
                            }}>
                                {"source"}
                            </button>
                            <button class={decompiled_tab_classes} onclick={{
                                let show_source = show_source.clone();
                                Callback::from(move |_| {
                                    show_source.set(!*show_source);
                                })
                            }}>
                                {"decompiled"}
                            </button>
                        </div>
                        <div style="height: 96%;" class="py-2 overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                            <pre class="text-xs whitespace-pre-wrap">
                                { text_html_content }
                            </pre>
                        </div>
                    </div>


                    <div id="information" class="split pr-2 ">
                        <div id="regs" class="overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                            <Registers state={state.clone()} />
                        </div>

                        <OutputArea
                            {mipsy_output_tab_title}
                            {input_needed}
                            show_io={show_io.clone()}
                            input_ref={(*input_ref).clone()}
                            on_input_keydown={on_input_keydown.clone()}
                            running_output={render_running_output(show_io.clone(), state.clone())}
                        />
                    </div>

                </div>

            </PageBackground>

        </>
    }
}

// if the key is a known nav key
// or some other key return true
pub fn is_nav_or_special_key(event: &KeyboardEvent) -> bool {
    if event.alt_key() || event.ctrl_key() || event.meta_key() {
        return true;
    }

    match event.key().as_str() {
        "Backspace" => true,
        "-" => true,
        _ => false,
    }
}

fn render_running(
    file: UseStateHandle<Option<String>>,
    state: RunningState,
    filename: UseStateHandle<Option<String>>,
    show_source: UseStateHandle<bool>,
) -> Html {
    html! {
        <>
            <h3>
                <strong class="text-lg">
                    {
                        filename.as_ref().unwrap_or(&"".to_string())
                    }
                </strong>
            </h3>
            <table>
                <tbody>
                    if *show_source {
                        <SourceCode file={(*file).clone()} />
                    } else {
                        <DecompiledCode {state} />
                    }
                </tbody>
            </table>
        </>
    }
}

fn render_running_output(show_io: UseStateHandle<bool>, state: UseStateHandle<State>) -> Html {
    html! {
        {
            if *show_io {
                match &*state {
                    State::Running(curr) => {curr.mips_state.stdout.join("")},
                    State::NoFile => {
                        "mipsy_web beta\nSchool of Computer Science and Engineering, University of New South Wales, Sydney."
                            .into()
                    },
                    State::CompilerError(_) => "File has syntax errors, check your file with mipsy and try again".into()

                }
            } else {
                match &*state {
                    State::Running(curr) => {curr.mips_state.mipsy_stdout.join("\n")},
                    State::NoFile => {"".into()},
                    State::CompilerError(_) => "File has syntax errors, check your file with mipsy and try again".into()
                }
            }
        }
    }
}

pub fn process_syscall_request(
    mips_state: MipsState,
    required_type: ReadSyscalls,
    state: UseStateHandle<State>,
    input_ref: UseStateHandle<NodeRef>,
) -> bool {
    match &*state {
        State::Running(ref curr) => {
            state.set(State::Running(RunningState {
                mips_state,
                input_needed: Some(required_type),
                ..curr.clone()
            }));
            focus_input(input_ref);
            true
        }

        State::NoFile | State::CompilerError(_) => false,
    }
}

fn focus_input(input_ref: UseStateHandle<NodeRef>) {
    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
        input.set_disabled(false);
        input.focus().unwrap();
    };
}

pub fn process_syscall_response(
    state: UseStateHandle<State>,
    worker: UseBridgeHandle<Worker>,
    input: HtmlInputElement,
    required_type: ReadSyscallInputs,
) {
    match state.deref() {
        State::Running(ref curr) => {
            worker.send(WorkerRequest::GiveSyscallValue(
                curr.mips_state.clone(),
                required_type,
            ));

            state.set(State::Running(RunningState {
                input_needed: None,
                ..curr.clone()
            }));

            input.set_value("");
            input.set_disabled(true);
        }
        State::NoFile | State::CompilerError(_) => {
            error!("Should not be possible to give syscall value with no file");
        }
    }
}
