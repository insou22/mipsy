use crate::worker::ReadSyscallInputs;
use crate::{
    components::{
        about_modal::Modal, banner::Banner, data_segment::DataSegment, decompiled::DecompiledCode,
        navbar::NavBar, outputarea::OutputArea, pagebackground::PageBackground,
        registers::Registers, settings_modal::SettingsModal, sourcecode::SourceCode,
    },
    state::{
        config::MipsyWebConfig,
        state::{DisplayedCodeTab, ErrorType, MipsState, RegisterTab, RunningState, State},
        update,
    },
    worker::{FileInformation, Worker, WorkerRequest},
};
use bounce::use_atom;
use gloo_console::log;
use gloo_file::callbacks::{read_as_text, FileReader};
use gloo_file::File;
use log::{error, info, trace};
use mipsy_lib::MipsyError;
use std::ops::Deref;
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

    let display_modal: UseStateHandle<bool> = use_state_eq(|| false);
    let settings_modal: UseStateHandle<bool> = use_state_eq(|| false);
    let show_io: UseStateHandle<bool> = use_state_eq(|| true);
    let input_ref: UseStateHandle<NodeRef> = use_state_eq(NodeRef::default);
    let filename: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let file: UseStateHandle<Option<String>> = use_state_eq(|| None);
    let show_code_tab: UseStateHandle<DisplayedCodeTab> = use_state_eq(|| DisplayedCodeTab::Source);
    let show_register_tab: UseStateHandle<RegisterTab> =
        use_state_eq(|| RegisterTab::UsedRegisters);
    let tasks: UseStateHandle<Vec<FileReader>> = use_state(std::vec::Vec::new);
    let is_saved: UseStateHandle<bool> = use_state_eq(|| false);
    let show_analytics_banner: UseStateHandle<bool> = use_state_eq(|| {
        // if we have ack'd analytics
        // dont show the banner
        // this is false for now, as analytics is not yet implemented
        crate::get_localstorage("analytics_ack")
            .map(|item| !(item.as_str() == "true"))
            .unwrap_or(false)
    });
    let worker = use_bridge(|_| {});
    let worker: UseBridgeHandle<Worker> = {
        let state = state.clone();
        let show_code_tab = show_code_tab.clone();
        let show_io = show_io.clone();
        let file = file.clone();
        let input_ref = input_ref.clone();
        let worker = worker.clone();
        let is_saved = is_saved.clone();
        let filename = filename.clone();
        use_bridge(move |response: <Worker as yew_agent::Agent>::Output| {
            update::handle_response_from_worker(
                state.clone(),
                show_code_tab.clone(),
                show_io.clone(),
                file.clone(),
                filename.clone(),
                response,
                worker.clone(),
                input_ref.clone(),
                is_saved.clone(),
            )
        })
    };

    let config = use_atom::<MipsyWebConfig>();

    if let State::NoFile = *state {
        is_saved.set(false);
    }

    // REFACTOR - move to fn/file
    {
        let file = file.clone();
        let file2 = file.clone();
        let file3 = file.clone();
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
                                let cb = Closure::wrap(Box::new(move || {
                                    let editor_contents = crate::get_editor_value();
                                    let editor_contents2 = editor_contents.clone();
                                    file3.set(Some(editor_contents2));

                                    let last_saved_contents =
                                        crate::get_localstorage_file_contents();

                                    is_saved.set(editor_contents == last_saved_contents);
                                })
                                    as Box<dyn Fn()>);

                                crate::set_model_change_listener(&cb);
                                cb.forget();
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
                        }
                    }
                };
                move || {} //do stuff when your componet is unmounted
            },
            (filename.clone(), file2, show_code_tab.clone()), // run use_effect when these dependencies change
        );
    }
    // now that we have an editor, restore some config
    {
        let config = config.clone();
        use_effect_with_deps(
            move |_| {
                let localstorage_config = crate::get_localstorage("mipsy_web_config");
                if let Some(localstorage_config) = localstorage_config {
                    if let Ok(parsed_localstorage) =
                        serde_json::from_str::<MipsyWebConfig>(&localstorage_config)
                    {
                        if parsed_localstorage != MipsyWebConfig::default() {
                            MipsyWebConfig::apply(&parsed_localstorage);
                            config.set(parsed_localstorage);
                        }
                    }
                } else {
                    crate::set_localstorage(
                        "mipsy_web_config",
                        &serde_json::to_string(&*config).unwrap(),
                    );
                }
                move || {}
            },
            (),
        )
    };
    if web_sys::window().unwrap().get("editor").is_some() {
        MipsyWebConfig::apply(&*config);
    }

    {
        let show_io = show_io.clone();
        let config = config.clone();
        let display_modal = display_modal.clone();
        let settings_modal = settings_modal.clone();
        use_effect_with_deps(move |_| {
            MipsyWebConfig::apply(&*config);
            move || {}
        }, (show_io, settings_modal, display_modal));
    }

    // REFACTOR - move to fn/file
    // when the state or show_tab changes
    // update the highlights
    {
        let state_copy = state.clone();
        let is_saved = is_saved.clone();
        let file = file.clone();
        use_effect(move || {
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

                                            let spaces_to_insert =
                                                config.deref().mipsy_config.tab_size
                                                    - (updated_line.len() as u32
                                                        % config.deref().mipsy_config.tab_size);

                                            updated_line
                                                .push_str(&" ".repeat(spaces_to_insert as usize));
                                        }

                                        updated_line
                                    };

                                    let last_column = updated_line.len() as u32 + 1;

                                    log!("highlighting from", err.col(), "to", last_column);
                                    crate::highlight_section(
                                        line_num,
                                        err.col(),
                                        last_column as u32,
                                    );
                                }
                            }
                            MipsyError::Runtime(_) => unreachable!(
                                "Runtime error should not be in ErrorType::CompilerOrParserError"
                            ),
                        }
                    }

                    ErrorType::RuntimeError(_) => {
                        // runtime errors expose no highlights
                    }
                }
            };

            if let State::Compiled(_) = &*state_copy {
                crate::remove_highlight();
            }
            move || {}
        });
    }

    /*    CALLBACKS   */
    let load_onchange: Callback<Event> = {
        let worker = worker.clone();
        let filename = filename.clone();
        let tasks = tasks;
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Some(file_list) = input.files() {
                if let Some(file_blob) = file_list.item(0) {
                    let gloo_file = File::from(file_blob);

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

                            worker.send(input);
                        }

                        Err(_e) => {}
                    }));

                    tasks.set(tasks_new);
                }
            }
        })
    };

    // REFACTOR - move this
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
                worker.send(WorkerRequest::CompileCode(FileInformation {
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
                update::submit_input(&worker, &input_ref, &state);
            };
            if event.key() == "d" && event.ctrl_key() {
                event.prevent_default();
                update::submit_eof(&worker, &input_ref, &state);
            };
        })
    };

    /* what is the html content of the body? */
    let text_html_content = match &*state {
        State::Compiled(_) | &State::Error(_) | &State::NoFile => render_running(
            file.clone(),
            state.clone(),
            filename.clone(),
            save_keydown,
            is_saved.clone(),
            show_code_tab.clone(),
            worker.clone(),
        ),
    };

    trace!("rendering");

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

    // REFACTOR - refactor this to use classes! macro somehow?
    let (source_tab_classes, decompiled_tab_classes, data_tab_classes) = {
        let (tab_select, tab_unselect, tab_left_select, tab_left_unselect) = get_tab_classes();

        match *show_code_tab {
            DisplayedCodeTab::Source => (tab_left_select, tab_unselect.clone(), tab_unselect),

            DisplayedCodeTab::Decompiled => (tab_left_unselect, tab_select, tab_unselect),

            DisplayedCodeTab::Data => (tab_left_unselect, tab_select, tab_unselect),
        }
    };

    let (used_registers_tab_classes, all_registers_tab_classes) = {
        let (tab_select, tab_unselect, tab_left_select, tab_left_unselect) = get_tab_classes();

        match *show_register_tab {
            RegisterTab::UsedRegisters => (tab_left_select, tab_unselect),
            RegisterTab::AllRegisters => (tab_left_unselect, tab_select),
        }
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
                    let settings_modal = settings_modal.clone();
                    Callback::from(move |_| {
                        if *display_modal {
                            display_modal.set(false);
                        } else if *settings_modal {
                            settings_modal.set(false);
                        }
                    })
                }}
                class={
                    classes!("bg-th-secondary", "bg-opacity-90", "absolute",
                             "top-0", "left-0", "h-screen", "w-screen", "z-20",
                              if !(*display_modal) && !(*settings_modal) { "hidden" } else { "" })
                }
            >
            </div>

            <Modal should_display={display_modal.clone()} />
            <SettingsModal
                analytics={show_analytics_banner.clone()}
                should_display={settings_modal.clone()}
            />

            <PageBackground>

                <NavBar
                    show_tab={show_code_tab.clone()}
                    {load_onchange}
                    {display_modal}
                    {settings_modal}
                    {file_loaded}
                    {waiting_syscall}
                    state={state.clone()}
                    worker={worker.clone()}
                    {filename}
                    {file}
                    {is_saved}
                />

                <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 137px)">
                    <div id="file_data" class="pl-2">
                        <div style="height: 4%;" class="flex overflow-hidden border-1 border-current">
                            <button class={source_tab_classes} onclick={{
                                let show_tab = show_code_tab.clone();
                                Callback::from(move |_| {
                                    show_tab.set(DisplayedCodeTab::Source);
                                })
                            }}>
                                {"source"}
                            </button>
                            <button class={decompiled_tab_classes} onclick={{
                                let show_tab = show_code_tab.clone();
                                Callback::from(move |_| {
                                    show_tab.set(DisplayedCodeTab::Decompiled);
                                })
                            }}>
                                {"decompiled"}
                            </button>
                            <button class={data_tab_classes} onclick={{
                                let show_tab = show_code_tab.clone();
                                Callback::from(move |_| {
                                    show_tab.set(DisplayedCodeTab::Data);
                                })
                            }}>
                                {"data"}
                            </button>
                        </div>
                        <div style="height: 96%;" class="py-2 overflow-y-auto bg-th-secondary px-2 border-2 border-current">
                                { text_html_content }
                        </div>
                    </div>


                    <div id="information" class="split pr-2 ">
                        <div style="height: 4%;" class="flex overflow-hidden border-1 border-current">
                            <button class={used_registers_tab_classes} onclick={{
                                let show_register_tab = show_register_tab.clone();
                                Callback::from(move |_| {
                                    show_register_tab.set(RegisterTab::UsedRegisters);
                                })
                            }}>
                                {"used registers"}
                            </button>
                            <button class={all_registers_tab_classes} onclick={{
                                let show_register_tab = show_register_tab.clone();
                                Callback::from(move |_| {
                                    show_register_tab.set(RegisterTab::AllRegisters);
                                })
                            }}>
                                {"all registers"}
                            </button>
                        </div>

                        <div id="regs" class="overflow-y-auto bg-th-secondary px-2 border-2 border-current">
                            <Registers state={state.clone()} tab={show_register_tab} />
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
                if *show_analytics_banner {
                    <Banner show_analytics_banner={show_analytics_banner}/>
                }
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

    matches!(event.key().as_str(), "Backspace | -")
}

fn render_running(
    file: UseStateHandle<Option<String>>,
    state: UseStateHandle<State>,
    filename: UseStateHandle<Option<String>>,
    save_keydown: Callback<KeyboardEvent>,
    is_saved: UseStateHandle<bool>,
    show_tab: UseStateHandle<DisplayedCodeTab>,
    worker: UseBridgeHandle<Worker>,
) -> Html {
    let display_filename = (&*filename.as_deref().unwrap_or("Untitled")).to_string();

    html! {
        <>
            <h3 class="flex flex-row justify-between">
                <p class="text-lg">{display_filename}</p>
                if !*is_saved {
                    <p style="color: #E06969">{"(unsaved file changes)"}</p>
                }
            </h3>
            {
                match *show_tab {
                    DisplayedCodeTab::Source => {
                        html!{
                            <SourceCode save_keydown={save_keydown} file={(*file).clone()}/>
                        }
                    },
                    DisplayedCodeTab::Decompiled => {
                        match &*state {
                            State::Compiled(curr) => {
                                html! {
                                    <DecompiledCode
                                        current_instr={curr.mips_state.current_instr}
                                        decompiled={curr.decompiled.clone()}
                                        state={state.clone()}
                                        worker={worker.clone()}
                                    />
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
                                                        state={state.clone()}
                                                        worker={worker.clone()}
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
                    DisplayedCodeTab::Data => {
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
                                <p>{"there was an error! See the Mipsy Output Tab for more :)"}</p>
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
            State::Error(curr) => match curr {
                ErrorType::RuntimeError(error) => {
                    html! {error.mips_state.mipsy_stdout.join("\n")}
                }
                ErrorType::CompilerOrParserError(error) => {
                    html! {error.mipsy_stdout.join("\n")}
                }
            },
        }
    }
}

pub fn process_syscall_request(
    mips_state: MipsState,
    required_type: ReadSyscalls,
    state: UseStateHandle<State>,
    input_ref: UseStateHandle<NodeRef>,
    show_io: UseStateHandle<bool>,
) {
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

fn get_tab_classes() -> (Classes, Classes, Classes, Classes) {
    let default_tab_classes =
        "w-1/2 leading-none float-left border-t-2 border-r-2 border-current cursor-pointer px-1";
    let left_tab_classes = classes!("border-l-2", default_tab_classes);
    let selected_classes = "bg-th-primary";
    let unselected_classes = "bg-th-tabunselected hover:bg-th-primary";

    (
        classes!(default_tab_classes, selected_classes), // selected tab
        classes!(default_tab_classes, unselected_classes), // unselected tab
        classes!(left_tab_classes.clone(), selected_classes), // selected left tab
        classes!(left_tab_classes, unselected_classes),  // unselected left tab
    )
}
