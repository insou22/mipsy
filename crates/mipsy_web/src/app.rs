use crate::worker::ReadSyscallInputs;
use crate::{
    components::{
        modal::Modal, navbar::NavBar, outputarea::OutputArea, pagebackground::PageBackground,
    },
    worker::{Worker, WorkerRequest, WorkerResponse},
};
use mipsy_lib::{MipsyError, Register, Runtime, Safe};
use serde::{Deserialize, Serialize};
use std::u32;
use wasm_bindgen::UnwrapThrowExt;
use yew::{
    prelude::*,
    services::{
        reader::{FileData, ReaderTask},
        ReaderService,
    },
    web_sys::{File, HtmlInputElement},
};

use log::{error, info, trace};

/*
// useful for debugigng, set the instruction set to be constructed by crimes
fn crimes<T>() -> T {
    panic!()
}
*/

#[derive(Clone)]
pub struct RunningState {
    decompiled: String,
    mips_state: MipsState,
    should_kill: bool,
    input_needed: Option<ReadSyscalls>,
}

#[derive(Clone, Debug)]
pub enum ReadSyscalls {
    ReadInt,
    ReadFloat,
    ReadDouble,
    ReadString,
    ReadChar,
    //Open(i32),
    //Read((i32, Vec<u8>)),
    //Write(i32),
    //Close(i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MipsState {
    pub stdout: Vec<String>,
    pub mipsy_stdout: Vec<String>,
    pub exit_status: Option<i32>,
    pub register_values: Vec<Safe<i32>>,
    pub current_instr: Option<u32>,
    pub is_stepping: bool,
}

impl MipsState {
    pub fn update_registers(&mut self, runtime: &Runtime) {
        self.register_values = runtime
            .timeline()
            .state()
            .registers()
            .iter()
            .cloned()
            .collect();
    }

    pub fn update_current_instr(&mut self, runtime: &Runtime) {
        self.current_instr = Some(runtime.timeline().state().pc());
    }
}

pub enum Msg {
    FileChanged(File),
    FileRead(FileData),
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
}

pub enum State {
    NoFile,
    CompilerError(MipsyError),
    // File is loaded, and compiled?
    Running(RunningState),
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,

    // a tasks vec to keep track of any file uploads
    tasks: Vec<ReaderTask>,
    state: State,
    worker: Box<dyn Bridge<Worker>>,
    display_modal: bool,
    show_io: bool,
    input_ref: NodeRef,
    filename: Option<String>,
    file: Option<String>,
    show_source: bool,
}

const NUM_INSTR_BEFORE_RESPONSE: i32 = 40;

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let worker = Worker::bridge(link.callback(Self::Message::FromWorker));
        wasm_logger::init(wasm_logger::Config::default());
        Self {
            link,
            state: State::NoFile,
            tasks: vec![],
            worker,
            display_modal: false,
            show_io: true,
            input_ref: NodeRef::default(),
            filename: None,
            file: None,
            show_source: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FileChanged(file) => {
                info!("file changed msg");
                // FIXME -- check result
                let result = ReaderService::read_file(
                    file,
                    self.link.callback(|file_data| Msg::FileRead(file_data)),
                );

                match result {
                    Ok(service) => self.tasks.push(service),

                    Err(err) => {
                        info!("{:?}", err);
                    }
                }
                false
            }

            Msg::FileRead(file_data) => {
                info!("{:?}", file_data);
                // TODO -- this should not be lossy
                self.filename = Some(file_data.name);
                let file = String::from_utf8_lossy(&file_data.content).to_string();
                self.file = Some(file.clone());
                let input = WorkerRequest::CompileCode(file);
                info!("sending to worker");
                self.worker.send(input);
                self.show_source = false;
                true
            }

            Msg::Run => {
                trace!("Run button clicked");
                if let State::Running(curr) = &mut self.state {
                    curr.mips_state.is_stepping = false;
                    let input =
                        WorkerRequest::Run(curr.mips_state.clone(), NUM_INSTR_BEFORE_RESPONSE);
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot run");
                    return false;
                }
                true
            }

            Msg::Kill => {
                trace!("Kill button clicked");
                if let State::Running(curr) = &mut self.state {
                    curr.should_kill = true;
                };
                true
            }

            Msg::OpenModal => {
                self.display_modal = !self.display_modal;
                true
            }

            Msg::ShowIoTab => {
                trace!("Show IO Button clicked");
                // only re-render upon change
                let prev_show = self.show_io;
                self.show_io = true;
                prev_show != true
            }

            Msg::ShowMipsyTab => {
                trace!("Show mipsy button clicked");
                // only re-render upon change
                let prev_show = self.show_io;
                self.show_io = false;
                prev_show != false
            }
            Msg::ShowSourceTab => {
                trace!("Show source button clicked");
                // only re-render upon change
                self.show_source = true;
                true
            }

            Msg::ShowDecompiledTab => {
                trace!("Show decompiled button clicked");
                self.show_source = false;
                true
            }

            Msg::StepForward => {
                trace!("Step forward button clicked");
                if let State::Running(curr) = &mut self.state {
                    curr.mips_state.is_stepping = true;
                    let input = WorkerRequest::Run(curr.mips_state.clone(), 1);
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot step");
                    return false;
                }
                true
            }

            Msg::StepBackward => {
                trace!("Step backward button clicked");
                if let State::Running(curr) = &mut self.state {
                    curr.mips_state.is_stepping = true;
                    let input = WorkerRequest::Run(curr.mips_state.clone(), -1);
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot step");
                    return false;
                }
                true
            }

            Msg::Reset => {
                trace!("Reset button clicked");
                if let State::Running(curr) = &mut self.state {
                    let input = WorkerRequest::ResetRuntime(curr.mips_state.clone());
                    self.worker.send(input);
                } else {
                    self.state = State::NoFile;
                }
                true
            }

            Msg::ProcessKeypress(event) => {
                if self.is_nav_or_special_key(&event) {
                    return true;
                };
                info!("processing {}", event.key());
                true
            }

            Msg::SubmitInput => {
                if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                    if let State::Running(curr) = &mut self.state {
                        use ReadSyscallInputs::*;
                        use ReadSyscalls::*;
                        match &curr.input_needed.as_ref().unwrap_throw() {
                            ReadInt => match input.value().parse::<i32>() {
                                Ok(num) => {
                                    Self::process_syscall_response(self, input, Int(num));
                                }
                                Err(_e) => {
                                    let error_msg = format!(
                                        "Failed to parse input '{}' as an i32",
                                        input.value()
                                    );
                                    error!("{}", error_msg);
                                    curr.mips_state.mipsy_stdout.push(error_msg);
                                }
                            },

                            ReadFloat => match input.value().parse::<f32>() {
                                Ok(num) => {
                                    Self::process_syscall_response(self, input, Float(num));
                                }

                                Err(_e) => {
                                    let error_msg = format!(
                                        "Failed to parse input '{}' as an f32",
                                        input.value()
                                    );
                                    error!("{}", error_msg);
                                    curr.mips_state.mipsy_stdout.push(error_msg);
                                }
                            },

                            ReadDouble => match input.value().parse::<f64>() {
                                Ok(num) => {
                                    Self::process_syscall_response(self, input, Double(num));
                                }
                                Err(_e) => {
                                    error!("Failed to parse input '{}' as an f64", input.value());
                                }
                            },

                            ReadChar => match input.value().parse::<char>() {
                                Ok(char) => {
                                    Self::process_syscall_response(self, input, Char(char as u8))
                                }
                                Err(_e) => {
                                    let error_msg = format!(
                                        "Failed to parse input '{}' as an u8",
                                        input.value()
                                    );
                                    error!("{}", error_msg);
                                    curr.mips_state.mipsy_stdout.push(error_msg);
                                }
                            },

                            ReadString => {
                                let string = format!("{}{}", input.value(), "\n").as_bytes().to_vec();

                                Self::process_syscall_response(self, input, String(string));
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
                    self.state = State::Running(RunningState {
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
                            match &mut self.state {
                                State::Running(curr) => {
                                    curr.mips_state.mipsy_stdout.push(error.error().message())
                                }
                                State::NoFile | State::CompilerError(_) => {
                                    self.state = State::CompilerError(err);
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

                WorkerResponse::ProgramExited(mips_state) => match &mut self.state {
                    State::Running(curr) => {
                        curr.mips_state = mips_state;
                        true
                    }
                    State::NoFile | State::CompilerError(_) => false,
                },

                WorkerResponse::InstructionOk(mips_state) => {
                    if let State::Running(curr) = &mut self.state {
                        curr.mips_state = mips_state;
                        // if the isntruction was ok, run another instruction
                        // unless the user has said it should be killed
                        if !curr.should_kill {
                            let input = WorkerRequest::Run(
                                curr.mips_state.clone(),
                                NUM_INSTR_BEFORE_RESPONSE,
                            );
                            self.worker.send(input);
                        }
                        curr.should_kill = false;
                    } else {
                        info!("No File loaded, cannot run");
                        return false;
                    }
                    true
                }

                WorkerResponse::UpdateMipsState(mips_state) => match &mut self.state {
                    State::Running(curr) => {
                        curr.mips_state = mips_state;
                        true
                    }

                    State::NoFile | State::CompilerError(_) => false,
                },

                WorkerResponse::NeedInt(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadInt)
                }
                WorkerResponse::NeedFloat(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadFloat)
                }
                WorkerResponse::NeedDouble(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadDouble)
                }
                WorkerResponse::NeedChar(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadChar)
                }
                WorkerResponse::NeedString(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadString)
                }
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }
    fn rendered(&mut self, _first_render: bool) {
        unsafe{crate::highlight();}
    }

    fn view(&self) -> Html {
        let onchange = self.link.batch_callback(|event| match event {
            ChangeData::Files(file_list) => {
                if let Some(file) = file_list.item(0) {
                    Some(Msg::FileChanged(file))
                } else {
                    None
                }
            }
            _ => None,
        });

        let on_input_keydown = self.link.callback(|event: KeyboardEvent| {
            if event.key() == "Enter" {
                return Msg::SubmitInput;
            };

            Msg::ProcessKeypress(event)
        });

        let run_onclick = self.link.callback(|_| Msg::Run);

        let kill_onclick = self.link.callback(|_| Msg::Kill);

        let reset_onclick = self.link.callback(|_| Msg::Reset);

        let step_forward_onclick = self.link.callback(|_| Msg::StepForward);

        let step_back_onclick = self.link.callback(|_| Msg::StepBackward);

        let toggle_modal_onclick = self.link.callback(|_| Msg::OpenModal);

        let text_html_content = match &self.state {
            State::NoFile => "no file loaded".into(),
            State::CompilerError(_error) => "File has syntax errors, check your file with mipsy and try again.\nMipsy Web does not yet support displaying compiler errors".into(),
            State::Running(ref state) => self.render_running(state),
        };

        let exit_status = match &self.state {
            State::Running(curr) => Some(curr.mips_state.exit_status),
            _ => None,
        };

        trace!("rendering");

        let modal_overlay_classes = if self.display_modal {
            "bg-th-secondary bg-opacity-90 absolute top-0 left-0 h-screen w-screen"
        } else {
            "hidden"
        };

        let show_io_tab = self.link.callback(|_| Msg::ShowIoTab);
        let show_mipsy_tab = self.link.callback(|_| Msg::ShowMipsyTab);
        let show_source_tab = self.link.callback(|_| Msg::ShowSourceTab);
        let show_decompiled_tab = self.link.callback(|_| Msg::ShowDecompiledTab);
        let file_loaded = match &self.state {
            State::NoFile | State::CompilerError(_) => false,
            State::Running(_) => true,
        };

        let waiting_syscall = match &self.state {
            State::Running(curr) => curr.input_needed.is_some(),
            State::NoFile | State::CompilerError(_) => false,
        };
        // TODO - make this nicer when refactoring compiler errs
        let mipsy_output_tab_title = match &self.state {
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

            if self.show_source {
                default.1 = format!("{} {}", &default.1, String::from("bg-th-tabclicked"));
            } else {
                default.0 = format!("{} {}", &default.0, String::from("bg-th-tabclicked"));
            };

            default
        };
        html! {
            <>
                <div onclick={toggle_modal_onclick.clone()} class={modal_overlay_classes}>
                </div>
                <Modal should_display={self.display_modal} toggle_modal_onclick={toggle_modal_onclick.clone()} />
                <PageBackground>
                    <NavBar
                        step_back_onclick=step_back_onclick step_forward_onclick=step_forward_onclick
                        exit_status=exit_status load_onchange=onchange
                        reset_onclick=reset_onclick run_onclick=run_onclick
                        kill_onclick=kill_onclick open_modal_onclick=toggle_modal_onclick
                        file_loaded=file_loaded waiting_syscall={waiting_syscall}
                    />
                    <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 122px)">
                        <div id="file_data">
                            <div style="height: 4%;" class="flex overflow-hidden border-1 border-black">
                                <button class={source_tab_classes} onclick={show_source_tab}>
                                    {"source"}
                                </button>
                                <button
                                    class={decompiled_tab_classes}
                                    onclick={show_decompiled_tab}
                                >
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
                                { self.render_running_registers() }
                            </div>

                            <OutputArea
                                show_io={self.show_io}
                                show_io_tab={show_io_tab}
                                show_mipsy_tab={show_mipsy_tab}
                                is_disabled={
                                    match &self.state {
                                        State::Running(curr) => {
                                            if curr.input_needed.is_none() { true } else { false }
                                        },
                                        State::NoFile | State::CompilerError(_) => { true },
                                    }
                                }
                                input_needed = {
                                    match &self.state {
                                        State::Running(curr) => {
                                            curr.input_needed.is_some()
                                        }
                                        State::NoFile | State::CompilerError(_)  => {
                                            false
                                        }
                                    }
                                }
                                input_ref={&self.input_ref}
                                on_input_keydown={on_input_keydown.clone()}
                                running_output={self.render_running_output()}
                                mipsy_output_tab_title={mipsy_output_tab_title}
                                input_maxlength={
                                    match &self.state {
                                        State::Running(curr) => match &curr.input_needed {
                                            Some(item) => match item {
                                                ReadSyscalls::ReadChar   => "1".to_string(),
                                                // should we have a limit for size?
                                                ReadSyscalls::ReadInt    => "".to_string(),
                                                ReadSyscalls::ReadDouble => "".to_string(),
                                                ReadSyscalls::ReadFloat  => "".to_string(),
                                                ReadSyscalls::ReadString => "".to_string(),
                                            },
                                            None => {"".to_string()}
                                        },
                                        State::NoFile | State::CompilerError(_) => {
                                            "".to_string()
                                        },
                                    }
                                }
                            />
                        </div>

                    </div>

                </PageBackground>

            </>
        }
    }
}

impl App {
    // if the key is a known nav key
    // or some other key return true
    fn is_nav_or_special_key(&self, event: &KeyboardEvent) -> bool {
        if event.alt_key() || event.ctrl_key() || event.meta_key() {
            return true;
        }

        match event.key().as_str() {
            "Backspace" => true,
            "-" => true,
            _ => false,
        }
    }

    fn render_running(&self, state: &RunningState) -> Html {
        html! {
            <>
            <h3>
            <strong class="text-lg">
            {
                self.filename.as_ref().unwrap_or(&"".to_string())
            }
            </strong>
            </h3>
            <table>
                <tbody>
                {
                    if self.show_source {
                        self.render_source_table()
                    } else {
                        self.render_decompiled_table(state)
                    }
                }
                </tbody>
            </table>
            </>
        }
    }

    fn render_source_table(&self) -> Html {
        info!("calling render source table");
        let file_len = self.file.as_ref().unwrap_or(&"".to_string()).len().to_string().len();
        info!("{}", file_len);
        html! {
            // if we ever want to do specific things on specific lines...
            {
                for self.file.as_ref().unwrap_or(&"".to_string()).as_str().split("\n").into_iter().enumerate().map(|(index, item)| {
                    if item == "" {
                        // this is &nbsp;
                        html! {
                            <tr>
                                <pre>
                                    <code class="language-mips" style="padding: 0 !important;">
                                    {format!("{:indent$} ",index, indent=file_len)}
                                    {"\u{00a0}"}
                                    </code>
                                </pre>
                            </tr>
                        }
                    }
                    else {
                        html! {
                            <tr>
                                <pre>
                                    <code class="language-mips" style="padding: 0 !important;">
                                        {format!("{:indent$} ",index, indent=file_len)}
                                        {item}
                                    </code>
                                </pre>
                            </tr>
                        }
                    }
                }
                )
            }
        }
    }

    fn render_decompiled_table(&self, state: &RunningState) -> Html {
        let runtime_instr = state.mips_state.current_instr.unwrap_or(0);
        let decompiled = &state.decompiled;
        html! {
            for decompiled.as_str().split("\n").into_iter().map(|item| {
                if item == "" {
                    // this is &nbsp;
                    html! {
                        <tr>{"\u{00a0}"}</tr>
                    }
                }
                else {
                    let should_highlight = if item.starts_with("0x") {
                        // the actual hex address lives from 2-10, 01 are 0x
                        let source_instr = u32::from_str_radix(&item[2..10], 16).unwrap_or(0);
                        source_instr == runtime_instr
                    } else {
                        false
                    };

                    html! {
                        <tr
                          class={
                            if should_highlight {
                              "bg-th-highlighting"
                            } else {
                              ""
                            }
                          }>
                            {item}
                        </tr>
                    }
                }
            })
        }
    }

    fn render_running_output(&self) -> Html {
        html! {
            {
                if self.show_io {
                    match &self.state {
                        State::Running(curr) => {curr.mips_state.stdout.join("")},
                        State::NoFile => {
                            "mipsy_web beta\nSchool of Computer Science and Engineering, University of New South Wales, Sydney."
                                .into()
                        },
                        State::CompilerError(_) => "File has syntax errors, check your file with mipsy and try again".into()

                    }
                } else {
                    match &self.state {
                        State::Running(curr) => {curr.mips_state.mipsy_stdout.join("\n")},
                        State::NoFile => {"".into()},
                        State::CompilerError(_) => "File has syntax errors, check your file with mipsy and try again".into()
                    }
                }
            }
        }
    }

    fn render_running_registers(&self) -> Html {
        let mut registers = &vec![Safe::Uninitialised; 32];
        if let State::Running(state) = &self.state {
            registers = &state.mips_state.register_values;
        };

        html! {
            <table class="w-full border-collapse table-auto">
                <thead>
                    <tr>
                        <th class="w-1/4">
                        {"Register"}
                        </th>
                        <th class="w-3/4">
                        {"Value"}
                        </th>
                    </tr>
                </thead>
                <tbody>
                {
                    for registers.iter().enumerate().map(|(index, item)| {
                        html! {
                            <tr>
                            {
                                match item {
                                    Safe::Valid(val) => {
                                        html! {
                                            <>
                                                <td class="border-gray-500 border-b-2 pl-4">
                                                    {"$"}
                                                    {Register::from_u32(index as u32).unwrap().to_lower_str()}
                                                </td>
                                                <td class="pl-4 border-b-2 border-gray-500">
                                                    <pre>
                                                        {format!("0x{:08x}", val)}
                                                    </pre>
                                                </td>
                                            </>
                                        }
                                    }

                                    Safe::Uninitialised => {html!{}}
                                }
                            }
                            </tr>
                        }
                    })
                }
                </tbody>
            </table>
        }
    }

    fn process_syscall_request(
        &mut self,
        mips_state: MipsState,
        required_type: ReadSyscalls,
    ) -> bool {
        match &mut self.state {
            State::Running(curr) => {
                curr.mips_state = mips_state;
                curr.input_needed = Some(required_type);
                self.focus_input();
                true
            }

            State::NoFile | State::CompilerError(_) => false,
        }
    }
    fn focus_input(&self) {
        if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
            input.set_disabled(false);
            input.focus().unwrap();
        };
    }

    fn process_syscall_response(
        &mut self,
        input: HtmlInputElement,
        required_type: ReadSyscallInputs,
    ) {
        match &mut self.state {
            State::Running(curr) => {
                self.worker.send(WorkerRequest::GiveSyscallValue(
                    curr.mips_state.clone(),
                    required_type,
                ));
                curr.input_needed = None;
                input.set_value("");
                input.set_disabled(true);
            }
            State::NoFile | State::CompilerError(_) => {
                error!("Should not be possible to give syscall value with no file");
            }
        }
    }
}
