use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::pages::main::state::DisplayedTab;
use crate::worker::ReadSyscallInputs;
use crate::{
    components::{
        decompiled::DecompiledCode, modal::Modal, navbar::NavBar, outputarea::OutputArea,
        pagebackground::PageBackground, registers::Registers, sourcecode::SourceCode,
        data_segment::DataSegment,
    },
    pages::main::{
        state::{MipsState, RunningState, State},
        update,
    },
    worker::{Worker, WorkerRequest},
};
use gloo_console::log;
use gloo_file::callbacks::{read_as_text, FileReader};
use gloo_file::File;
use log::{error, info, trace};
use mipsy_lib::MipsyError;
use web_sys::{Element, HtmlInputElement};
use yew::prelude::*;
use yew_agent::{use_bridge, UseBridgeHandle};

#[derive(Clone, Debug, PartialEq)]
pub enum ReadSyscalls {
    ReadInt,
    ReadFloat,
    ReadDouble,
    ReadString,
    ReadChar,
}

pub const NUM_INSTR_BEFORE_RESPONSE: i32 = 40;

#[function_component(App)]
pub fn render_app() -> Html {
    /* State Handlers */
    let state: UseStateHandle<State> = use_state_eq(|| State::NoFile);

    let worker = Rc::new(RefCell::new(None));

    let display_modal: UseStateHandle<bool> = use_state_eq(|| false);
    let show_io: UseStateHandle<bool> = use_state_eq(|| true);
    let input_ref: UseStateHandle<NodeRef> = use_state_eq(|| NodeRef::default());
    let filename: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let file: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let show_tab: UseStateHandle<DisplayedTab> = use_state_eq(|| DisplayedTab::Source);
    let tasks: UseStateHandle<Vec<FileReader>> = use_state(|| vec![]);
    let is_saved: UseStateHandle<bool> = use_state_eq(|| true);

    {
        let file = file.clone();
        let file2 = file.clone();
        use_effect_with_deps(
            move |_| {
                //do stuff here for first render/mounted
                unsafe {
                    log!("Running initialise editor");
                    let document = web_sys::window().unwrap().document().unwrap();
                    let element: Option<Element> = document.get_element_by_id("monaco_editor");
                    match element {
                        Some(e) => {
                            if e.child_element_count() == 0 {
                                crate::init_editor();
                            }
                            if let Some(file) = &*file {
                                crate::set_editor_value(file.as_str())
                            } else {
                                crate::set_editor_value("");
                            }
                        }
                        None => {
                            log!("Could not find element with id 'monaco_editor'");
                        }
                    }
                };
                move || {} //do stuff when your componet is unmounted
            },
            (filename.clone(), file2.clone(), show_tab.clone()), // empty toople dependecy is what enables this
        );
    }

    {
        let state_copy = state.clone();
        use_effect_with_deps(
            move |_| {
                if let State::CompilerError(comp_err_state) = &*state_copy {
                    if let MipsyError::Compiler(err) = &comp_err_state.error {
                        info!("calling highlight_section");
                        crate::highlight_section(err.line(), err.col(), err.col_end());
                    }
                };
                move || {}
            },
            (state.clone(), show_tab.clone()),
        );
    }


    // if we have not yet setup the worker bridge, do so now
    if worker.borrow().is_none() {
        *worker.borrow_mut() = {
            let state = state.clone();
            let show_tab = show_tab.clone();
            let show_io = show_io.clone();
            let file = file.clone();
            let input_ref = input_ref.clone();
            let worker = worker.clone();

            Some(use_bridge(move |response| {
                let state = state.clone();
                let show_tab = show_tab.clone();
                let show_io = show_io.clone();
                let file = file.clone();
                let input_ref = input_ref.clone();
                let worker = worker.clone();
                update::handle_response_from_worker(
                    state, show_tab, show_io, file, response, worker, input_ref,
                )
            }))
        };
    }

    /*    CALLBACKS   */
    let load_onchange: Callback<Event> = {
        let worker = worker.clone();
        let filename = filename.clone();
        let show_tab = show_tab.clone();
        let tasks = tasks.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Some(file_list) = input.files() {
                if let Some(file_blob) = file_list.item(0) {
                    let gloo_file = File::from(web_sys::File::from(file_blob));

                    let file_name = gloo_file.name();
                    filename.set(Some(file_name));

                    // prep items for closure below
                    let worker = worker.clone();

                    let mut tasks_new = vec![];
                    tasks_new.push(read_as_text(&gloo_file, move |res| match res {
                        Ok(ref file_contents) => {
                            // file.set(Some(file_contents.to_string()));
                            let input = WorkerRequest::CompileCode(file_contents.to_string());
                            log!("sending to worker");

                            worker.borrow().as_ref().unwrap().send(input);
                        }

                        Err(_e) => {}
                    }));

                    tasks.set(tasks_new);
                }
            }
        })
    };

    let save_keydown: Callback<KeyboardEvent> = {
        let file = file.clone();
        let worker = worker.clone();
        let is_saved = is_saved.clone();
        Callback::from(move |e: KeyboardEvent| {
            if !e.ctrl_key() {
                is_saved.set(false);
            }
            if e.key() == "s" && e.ctrl_key() {
                e.prevent_default();
                log!("ctrl+s");
                is_saved.set(true);
                let updated_content = crate::get_editor_value();
                let clone = updated_content.clone();
                file.set(Some(updated_content));
                worker
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .send(WorkerRequest::CompileCode(clone));
            };
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
            if event.key() == "d" && event.ctrl_key() {
                event.prevent_default();
                update::submit_eof(worker.borrow().as_ref().unwrap(), &input_ref, &state);
            };
        })
    };

    /* what is the html content of the body? */
    let text_html_content = match &*state {
        State::Compiled(_) | &State::CompilerError(_) | &State::NoFile=> render_running(
            file.clone(),
            state.clone(),
            filename.clone(),
            save_keydown.clone(),
            is_saved.clone(),
            show_tab.clone(),
        ),
    };

    trace!("rendering");

    let modal_overlay_classes = if *display_modal {
        "bg-th-secondary bg-opacity-90 absolute top-0 left-0 h-screen w-screen z-20"
    } else {
        "hidden"
    };

    let file_loaded = match *state {
        State::NoFile | State::CompilerError(_) => false,
        State::Compiled(_) => true,
    };

    let waiting_syscall = match &*state {
        State::Compiled(curr) => curr.input_needed.is_some(),
        State::NoFile | State::CompilerError(_) => false,
    };

    // TODO - make this nicer when refactoring compiler errs
    let mipsy_output_tab_title = match &*state {
        State::NoFile => "Mipsy Output - (0)".to_string(),
        State::CompilerError(_) => "Mipsy Output - (1)".to_string(),
        State::Compiled(curr) => {
            format!("Mipsy Output - ({})", curr.mips_state.mipsy_stdout.len())
        }
    };

    let (decompiled_tab_classes, source_tab_classes, data_tab_classes) = {
        let mut default = (
            String::from("w-1/2 leading-none hover:bg-white float-left border-t-2 border-r-2 border-black cursor-pointer px-1"),
            String::from("w-1/2 leading-none hover:bg-white float-left border-t-2 border-r-2 border-l-2 border-black cursor-pointer px-1 "),
            String::from("w-1/2 leading-none hover:bg-white float-left border-t-2 border-r-2 border-black cursor-pointer px-1 ")
        );

        match *show_tab {
            DisplayedTab::Source => {
                default.1 = format!("{} {}", &default.1, String::from("bg-th-tabclicked"));
            }

            DisplayedTab::Decompiled => {
                default.0 = format!("{} {}", &default.0, String::from("bg-th-tabclicked"));
            }

            DisplayedTab::Data => {
                default.2 = format!("{} {}", &default.2, String::from("bg-th-tabclicked"));
            }
        };

        default
    };

    let input_needed = match &*state {
        State::Compiled(curr) => curr.input_needed.clone(),
        State::NoFile | State::CompilerError(_) => None,
    };

    let rendered_running = render_running_output(show_io.clone(), state.clone());
    html! {
        <>
            <div
                onclick={{
                    let display_modal = display_modal.clone();
                    Callback::from(move |_| {
                        display_modal.set(!*display_modal);
                    })
                }}
                class={modal_overlay_classes}
            >
            </div>

            <Modal should_display={display_modal.clone()} />

            <PageBackground>

                <NavBar
                    {load_onchange}
                    display_modal={display_modal.clone()}
                    {file_loaded}
                    {waiting_syscall}
                    state={state.clone()}
                    worker={worker.borrow().as_ref().unwrap().clone()}
                    filename={filename.clone()}
                    file={file.clone()}
                />

                <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 122px)">
                    <div id="file_data">
                        <div style="height: 4%;" class="flex overflow-hidden border-1 border-black">
                            <button class={source_tab_classes} onclick={{
                                let show_tab = show_tab.clone();
                                Callback::from(move |_| {
                                    show_tab.set(DisplayedTab::Source);
                                })
                            }}>
                                {"source"}
                            </button>
                            <button class={decompiled_tab_classes} onclick={{
                                let show_tab = show_tab.clone();
                                Callback::from(move |_| {
                                    show_tab.set(DisplayedTab::Decompiled);
                                })
                            }}>
                                {"decompiled"}
                            </button>
                            <button class={data_tab_classes} onclick={{
                                let show_tab = show_tab.clone();
                                Callback::from(move |_| {
                                    show_tab.set(DisplayedTab::Data);
                                })
                            }}>
                                {"data"}
                            </button>
                        </div>
                        <div style="height: 96%;" class="py-2 overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                                { text_html_content }
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
                            running_output={rendered_running}
                        />
                    </div>

                </div>

            </PageBackground>

        </>
    }
}

// if the key is a known nav key
// or some other key return true
// this fn is unused, but kept as documentation for keyboard events
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
    state: UseStateHandle<State>,
    filename: UseStateHandle<Option<String>>,
    save_keydown: Callback<KeyboardEvent>,
    is_saved: UseStateHandle<bool>,
    show_tab: UseStateHandle<DisplayedTab>,
) -> Html {
    let display_filename = if *is_saved {
        format!("{}", &*filename.as_deref().unwrap_or("Untitled"))
    } else {
        format!("{}*", &*filename.as_deref().unwrap_or("Untitled"))
    };

    html! {
        <>
            <h3>
                <strong class="text-lg">
                    {
                        display_filename
                    }
                </strong>
            </h3>
                    {
                        match *show_tab {
                            DisplayedTab::Source => {
                                html!{
                                    <SourceCode save_keydown={save_keydown.clone()} file={(*file).clone()}/>
                                }
                            },
                            DisplayedTab::Decompiled => {
                                match &*state {
                                    State::Compiled(curr) => {
                                        html! {
                                            <pre class="text-xs whitespace-pre-wrap">
                                                <table>
                                                    <tbody>
                                                        <DecompiledCode
                                                            state={curr.clone()}
                                                        />
                                                    </tbody>
                                                </table>
                                            </pre>
                                        }
                                    },
                                    State::NoFile => html! {
                                        <pre class="text-xs whitespace-pre-wrap">
                                            {"No file loaded or saved"}
                                        </pre>
                                    },
                                    State::CompilerError(_) => html! {
                                        <p>{"Compiler error! See the Mipsy Output Tab for more :)"}</p>
                                    },
                                }
                            },
                            DisplayedTab::Data => {
                                match &*state {
                                    State::Compiled(curr) => {
                                        html! {
                                            <DataSegment state={curr.clone()} />
                                        }
                                    },
                                    State::NoFile => html! {
                                        <pre class="text-xs whitespace-pre-wrap">
                                            {"No file loaded or saved"}
                                        </pre>
                                    },
                                    State::CompilerError(_) => html! {
                                        <p>{"Compiler error! See the Mipsy Output Tab for more :)"}</p>
                                    },
                                }
                            },
                        }
                    }
        </>
    }
}

fn render_running_output(show_io: UseStateHandle<bool>, state: UseStateHandle<State>) -> Html {
    if *show_io {
        match &*state {
            State::Compiled(curr) => {
                trace!("rendering running output");
                trace!("{:?}", curr.mips_state.mipsy_stdout);
                match curr.mips_state.exit_status.as_ref() {
                    Some(_) => {
                        html! {curr.mips_state.stdout.join("")}
                    }
                    None => {
                        html! {curr.mips_state.stdout.join("") + "▌"}
                    }
                }
            }
            State::NoFile => {
                html! {"mipsy_web beta\nSchool of Computer Science and Engineering, University of New South Wales, Sydney."}
            }
            State::CompilerError(_) => {
                html! {"File has compiler errors!"}
            }
        }
    } else {
        match &*state {
            State::Compiled(curr) => html! {curr.mips_state.mipsy_stdout.join("\n")},
            State::NoFile => html! {""},
            State::CompilerError(curr) => {
                html! {curr.mipsy_stdout.join("")}
            }
        }
    }
}

pub fn process_syscall_request(
    mips_state: MipsState,
    required_type: ReadSyscalls,
    state: UseStateHandle<State>,
    input_ref: UseStateHandle<NodeRef>,
) -> () {
    if let State::Compiled(ref curr) = &*state {
        state.set(State::Compiled(RunningState {
            mips_state,
            input_needed: Some(required_type),
            ..curr.clone()
        }));
        focus_input(input_ref);
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
        State::Compiled(ref curr) => {
            worker.send(WorkerRequest::GiveSyscallValue(
                curr.mips_state.clone(),
                required_type,
            ));

            state.set(State::Compiled(RunningState {
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
