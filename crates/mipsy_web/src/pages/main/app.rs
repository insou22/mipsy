use crate::worker::ReadSyscallInputs;
use crate::{
    components::{
        data_segment::DataSegment, decompiled::DecompiledCode, modal::Modal, navbar::NavBar,
        outputarea::OutputArea, pagebackground::PageBackground, registers::Registers,
        sourcecode::SourceCode,
    },
    state::{
        config::MipsyWebConfig,
        state::{DisplayedTab, MipsState, RunningState, State, ErrorType},
        update,
    },
    worker::{FileInformation, Worker, WorkerRequest},
};
use gloo_console::log;
use gloo_file::callbacks::{read_as_text, FileReader};
use gloo_file::File;
use log::{error, info, trace};
use mipsy_lib::MipsyError;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use web_sys::{window, Element, HtmlInputElement};
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
    let on_content_change_closure_handle: UseStateHandle<bool> = use_state_eq(|| false);
    let display_modal: UseStateHandle<bool> = use_state_eq(|| false);
    let show_io: UseStateHandle<bool> = use_state_eq(|| true);
    let input_ref: UseStateHandle<NodeRef> = use_state_eq(|| NodeRef::default());
    let filename: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let file: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let show_tab: UseStateHandle<DisplayedTab> = use_state_eq(|| DisplayedTab::Source);
    let tasks: UseStateHandle<Vec<FileReader>> = use_state(|| vec![]);
    let is_saved: UseStateHandle<bool> = use_state_eq(|| false);
    let config: UseStateHandle<MipsyWebConfig> = use_state_eq(|| MipsyWebConfig::default());

    if let State::NoFile = *state {
        is_saved.set(false);
    }
    {
        let file = file.clone();
        let file2 = file.clone();
        let on_content_change_closure_handle = on_content_change_closure_handle.clone();
        let is_saved = is_saved.clone();
        let filename2 = filename.clone();
        use_effect_with_deps(
            move |_| {
                unsafe {
                    let document = web_sys::window().unwrap().document().unwrap();
                    let element: Option<Element> = document.get_element_by_id("monaco_editor");
                    match element {
                        Some(e) => {
                            // if the editor does not exist, create it
                            if e.child_element_count() == 0 {
                                crate::init_editor();
                            }

                            // if window element is on the page, create, leak, and add the onchange callback
                            // only if we have not already added it
                            if window().unwrap().get("editor").is_some() {
                                if !*on_content_change_closure_handle {
                                    let cb = Closure::wrap(Box::new(move || {
                                        let editor_contents = crate::get_editor_value();

                                        let last_saved_contents =
                                            crate::get_localstorage_file_contents();

                                        if last_saved_contents != editor_contents {
                                            info!("File has changed");
                                            is_saved.set(false);
                                        } else {
                                            is_saved.set(true);
                                        }
                                    })
                                        as Box<dyn Fn()>);

                                    crate::set_model_change_listener(&cb);
                                    cb.forget();
                                    on_content_change_closure_handle.set(true);
                                };
                            }

                            if let Some(file) = &*file {
                                crate::set_editor_value(file.as_str())
                            } else {
                                let local_editor_contents = crate::get_localstorage_file_contents();
                                if local_editor_contents.is_empty() {
                                    crate::set_editor_value("")
                                } else {
                                    crate::set_editor_value(local_editor_contents.as_str());
                                    filename2.set(Some(crate::get_localstorage_filename()));
                                }
                            }
                        }
                        None => {
                            log!("Could not find element with id 'monaco_editor'");
                        }
                    }
                };
                move || {} //do stuff when your componet is unmounted
            },
            (filename.clone(), file2.clone(), show_tab.clone()), // run use_effect when these dependencies change
        );
    }

    // when the state or show_tab changes
    // update the highlights
    {
        let state_copy = state.clone();
        let is_saved = is_saved.clone();
        let file = file.clone();
        use_effect_with_deps(
            move |_| {
                if let State::Error(comp_err_state) = &*state_copy {
                    
                    match &comp_err_state {
                        ErrorType::CompilerOrParserError(error_state) => {
                            match &error_state.error {
                                MipsyError::Compiler(err) => {
                                    info!("adding higlight decorations on line {}", err.line());
                                    
                                    if err.error().should_highlight_line() {
                                        crate::highlight_section(err.line(), err.col(), err.col_end());
                                    }
                                    is_saved.set(true);
                                }
                                MipsyError::Parser(err) => {
                                    info!("adding higlights for parser err");

                                    let line_num = err.line();

                                    let line = {
                                        let target_line = (line_num - 1) as usize;

                                        let file = file.as_deref().unwrap_or("");
                                        let line = file.lines().nth(target_line);

                                        // special case: file is empty and ends with a newline, in which case the
                                        // parser will point to char 1-1 of the final line, but .lines() won't consider
                                        // that an actual line, as it doesn't contain any actual content.
                                        //
                                        // the only way this can actually occur is if the file contains no actual items,
                                        // as otherwise it would be happy to reach the end of the file, and return the
                                        // program. so we can just give a customised error message instead.
                                        if line.is_none()
                                            && file.ends_with('\n')
                                            && target_line == file.lines().count()
                                        {
                                            eprintln!("file contains no MIPS contents!");
                                            None
                                        } else {
                                            Some(line.expect("invalid line position in compiler error"))
                                        }
                                    };

                                    if let Some(line) = line {
                                        let updated_line = {
                                            let mut updated_line = String::new();

                                            for char in line.chars() {
                                                if char != '\t' {
                                                    updated_line.push(char);
                                                    continue;
                                                }

                                                let spaces_to_insert = config.deref().mipsy_config.tab_size
                                                    - (updated_line.len() as u32
                                                        % config.deref().mipsy_config.tab_size);

                                                updated_line.push_str(&" ".repeat(spaces_to_insert as usize));
                                            }

                                            updated_line
                                        };

                                        let last_column = updated_line.len() as u32 + 1;

                                        log!("highlighting from", err.col(), "to", last_column);
                                        crate::highlight_section(line_num, err.col(), last_column as u32);
                                    }
                                }
                                MipsyError::Runtime(_) => unreachable!("Runtime error should not be in ErrorType::CompilerOrParserError"),
                            }

                        }

                        ErrorType::RuntimeError(_) => {
                            // runtime errors expose no highlights
                        }
                    }
                };

                if let State::Compiled(_) = &*state_copy {
                    info!("removing highlight decorations");
                    crate::remove_highlight();
                }
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
            let is_saved = is_saved.clone();
            let filename = filename.clone();
            Some(use_bridge(move |response| {
                let state = state.clone();
                let show_tab = show_tab.clone();
                let show_io = show_io.clone();
                let file = file.clone();
                let input_ref = input_ref.clone();
                let worker = worker.clone();
                let is_saved = is_saved.clone();
                let filename = filename.clone();
                update::handle_response_from_worker(
                    state, show_tab, show_io, file, filename, response, worker, input_ref, is_saved,
                )
            }))
        };
    }

    /*    CALLBACKS   */
    let load_onchange: Callback<Event> = {
        let worker = worker.clone();
        let filename = filename.clone();
        let tasks = tasks.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Some(file_list) = input.files() {
                if let Some(file_blob) = file_list.item(0) {
                    let gloo_file = File::from(web_sys::File::from(file_blob));

                    let file_name = gloo_file.name();
                    filename.set(Some(file_name.clone()));
                    crate::set_localstorage_filename(&file_name);
                    // prep items for closure below
                    let worker = worker.clone();

                    let mut tasks_new = vec![];
                    tasks_new.push(read_as_text(&gloo_file, move |res| match res {
                        Ok(ref file_contents) => {
                            // file.set(Some(file_contents.to_string()));
                            let input = WorkerRequest::CompileCode(FileInformation {
                                filename: file_name.clone(),
                                file: file_contents.to_string(),
                            });
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
        let filename = filename.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "s" && (e.ctrl_key() || e.meta_key()) {
                e.prevent_default();
                log!("ctrl+s save pressed");
                is_saved.set(true);
                let updated_content = crate::get_editor_value();
                let clone = updated_content.clone();
                let filename = &filename.as_deref().unwrap_or("Untitled");
                crate::set_localstorage_file_contents(&updated_content);
                crate::set_localstorage_filename(filename);
                file.set(Some(updated_content));
                worker
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .send(WorkerRequest::CompileCode(FileInformation {
                        filename: filename.to_string(),
                        file: clone,
                    }));
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
        State::Compiled(_) | &State::Error(_) | &State::NoFile => render_running(
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
        State::NoFile | State::Error(_) => false,
        State::Compiled(_) => true,
    };

    let waiting_syscall = match &*state {
        State::Compiled(curr) => curr.input_needed.is_some(),
        State::NoFile | State::Error(_) => false,
    };

    // TODO - make this nicer when refactoring compiler errs
    let mipsy_output_tab_title = match &*state {
        State::NoFile => "Mipsy Output - (0)".to_string(),
        State::Error(_) => "Mipsy Output - (1)".to_string(),
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
        State::NoFile | State::Error(_) => None,
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
                    is_saved={is_saved.clone()}
                />

                <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 107px)">
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
        format!(
            "{} - [unsaved]",
            &*filename.as_deref().unwrap_or("Untitled")
        )
    };

    let callback_filename = filename.clone();
    #[allow(unused_must_use)]
    let filename_keydown = Callback::from(move |event: InputEvent| {
        let element: HtmlInputElement = event.target_unchecked_into();
        let value = element.value();
        crate::set_localstorage_filename(&value);
        callback_filename.set(Some(value));
    });

    html! {
        <>
            <h3>
                <input value={display_filename} oninput={filename_keydown} id="filename" class="text-lg" />
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
                                                            current_instr={curr.mips_state.current_instr}
                                                            decompiled={curr.decompiled.clone()}
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
                                    State::Error(error_type) => {
                                        if let ErrorType::RuntimeError(error) = error_type {
                                            html! {
                                                <pre class="text-xs whitespace-pre-wrap">
                                                    <table>
                                                        <tbody>
                                                            <DecompiledCode
                                                                current_instr={error.mips_state.current_instr}
                                                                decompiled={error.decompiled.clone()}
                                                            />
                                                        </tbody>
                                                    </table>
                                                </pre>
                                            }
                                        } else {
                                            html! {
                                                <p>{"There was an error when compiling! See the Mipsy Output Tab for more :)"}</p>
                                            }
                                        }
                                        
                                    }
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
                                    State::Error(_) => html! {
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
            State::Error(_) => {
                html! {""}
            }
        }
    } else {
        match &*state {
            State::Compiled(curr) => html! {curr.mips_state.mipsy_stdout.join("\n")},
            State::NoFile => html! {""},
            State::Error(curr) => {
                match curr {
                    ErrorType::RuntimeError(error) => {
                        html! {error.mips_state.mipsy_stdout.join("\n")}
                    }
                    ErrorType::CompilerOrParserError(error) => {
                        html! {error.mipsy_stdout.join("\n")}
                    }
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
    show_io: UseStateHandle<bool>,
) -> () {
    if let State::Compiled(ref curr) = &*state {
        state.set(State::Compiled(RunningState {
            mips_state,
            input_needed: Some(required_type),
            ..curr.clone()
        }));
        show_io.set(true);
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
        State::NoFile | State::Error(_) => {
            error!("Should not be possible to give syscall value with no file");
        }
    }
}
