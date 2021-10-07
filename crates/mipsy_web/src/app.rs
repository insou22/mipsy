use crate::worker::ReadSyscallInputs;
use crate::{
    components::{modal::Modal, navbar::NavBar, pagebackground::PageBackground},
    worker::{Worker, WorkerRequest, WorkerResponse},
};
use mipsy_lib::{Register, Runtime, Safe};
use serde::{Deserialize, Serialize};
use std::u32;
use wasm_bindgen::UnwrapThrowExt;
use yew::{
    prelude::*,
    services::{
        reader::{FileData, ReaderTask},
        ReaderService,
    },
    web_sys::{Element, File, HtmlInputElement},
};

use log::{error, info, warn};

fn crimes<T>() -> T {
    panic!()
}

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
    //ReadString(Vec<u8>),
    //ReadChar(u8),
    //Open(i32),
    //Read((i32, Vec<u8>)),
    //Write(i32),
    //Close(i32),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MipsState {
    pub stdout: Vec<String>,
    pub exit_status: Option<i32>,
    pub register_values: Vec<Safe<i32>>,
    pub current_instr: Option<u32>,
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
    StepForward,
    StepBackward,
    SubmitInput,
    ProcessKeypress(KeyboardEvent),
    FromWorker(WorkerResponse),
}

#[derive(Clone)]
pub enum State {
    NoFile,
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
    input_ref: NodeRef,
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
            input_ref: NodeRef::default(),
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

                let file = String::from_utf8_lossy(&file_data.content).to_string();

                let input = WorkerRequest::CompileCode(file);
                info!("sending to worker");
                self.worker.send(input);

                true
            }

            Msg::Run => {
                if let State::Running(curr) = &mut self.state {
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
                error!("KILL BUTTONM PRESSED");
                if let State::Running(curr) = &mut self.state {
                    curr.should_kill = true;
                };
                true
            }

            Msg::OpenModal => {
                self.display_modal = !self.display_modal;
                true
            }

            Msg::StepForward => {
                if let State::Running(curr) = &mut self.state {
                    let input = WorkerRequest::Run(curr.mips_state.clone(), 1);
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot step");
                    return false;
                }
                true
            }

            Msg::StepBackward => {
                if let State::Running(curr) = &mut self.state {
                    let input = WorkerRequest::Run(curr.mips_state.clone(), -1);
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot step");
                    return false;
                }
                true
            }

            Msg::Reset => {
                if let State::Running(curr) = &mut self.state {
                    info!("hello");
                    let input = WorkerRequest::ResetRuntime(curr.mips_state.clone());
                    self.worker.send(input);
                }
                true
            }

            Msg::ProcessKeypress(event) => {
                match &self.state {
                    State::Running(curr) => match &curr.input_needed {
                        Some(item) => match item {
                            // for ReadInt - we only want to allow intgers
                            ReadSyscalls::ReadInt => {
                                if self.is_nav_or_special_key(&event) {
                                    return true;
                                };

                                let key = event.key();
                                if key.parse::<i32>().is_err() {
                                    Event::from(event).prevent_default();
                                }
                            }
                        },
                        None => {}
                    },
                    State::NoFile => {}
                }
                true
            }

            Msg::SubmitInput => {
                if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                    if let State::Running(curr) = &self.state {
                        use ReadSyscallInputs::*;
                        use ReadSyscalls::*;
                        match &curr.input_needed.as_ref().unwrap_throw() {
                            ReadInt => {
                                match input.value().parse::<i32>() {
                                    Ok(num) => {
                                        Self::process_syscall_response(self, input, Int(num));    
                                    }
                                    Err(e) => {
                                        error!("Failed to parse input as an i32");
                                    }
                                }
                            }

                            ReadFloat => {
                                match input.value().parse::<f32>() {
                                    Ok(num) => {
                                        Self::process_syscall_response(self, input, Float(num));    
                                    }

                                    Err(e) => {
                                        error!("Failed to parse input as an f32");
                                    }
                                }
                            }

                            ReadDouble => {
                                match input.value().parse::<f64>() {
                                    Ok(num) => {
                                        Self::process_syscall_response(self, input, Double(num));    
                                    }
                                    Err(e) => {
                                        error!("Failed to parse input as an f64");
                                    }
                                }
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
                    info!("{}", &decompiled);
                    match self.state {
                        // Overwrite existing state,
                        State::NoFile | State::Running(_) => {
                            self.state = State::Running(RunningState {
                                decompiled,
                                mips_state: MipsState {
                                    stdout: Vec::new(),
                                    exit_status: None,
                                    register_values: vec![Safe::Uninitialised; 32],
                                    current_instr: None,
                                },
                                input_needed: None,
                                should_kill: false,
                            });
                            true
                        }
                    }
                }

                WorkerResponse::CompilerError(err) => {
                    todo!();
                }

                WorkerResponse::ProgramExited(mips_state) => match &mut self.state {
                    State::Running(curr) => {
                        curr.mips_state = mips_state;
                        true
                    }
                    State::NoFile => false,
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

                    State::NoFile => false,
                },

                WorkerResponse::NeedInt(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadInt)
                }, 
                WorkerResponse::NeedFloat(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadFloat)
                },
                WorkerResponse::NeedDouble(mips_state) => {
                    Self::process_syscall_request(self, mips_state, ReadSyscalls::ReadDouble)
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
            &State::NoFile => "no file loaded".into(),
            &State::Running(ref state) => self.render_running(state),
        };

        let output_html_content = match &self.state {
                    &State::NoFile => "mipsy_web beta\nSchool of Computer Science and Engineering, University of New South Wales, Sydney.".into(),
                    &State::Running(ref state) => self.render_running_output(state),
        };

        let exit_status = match &self.state {
            State::Running(curr) => Some(curr.mips_state.exit_status),
            _ => None,
        };

        info!("rendering");

        let modal_overlay_classes = if self.display_modal {
            "bg-th-secondary bg-opacity-90 absolute top-0 left-0 h-screen w-screen"
        } else {
            "hidden"
        };

        let input_classes = match &self.state {
            State::Running(curr) => {
                if curr.input_needed.is_none() {
                    "block w-full cursor-not-allowed"
                } else {
                    "block w-full bg-th-highlighting"
                }
            }
            State::NoFile => "block w-full",
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
                    />
                    <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 122px)">
                        <div id="source_file" class="py-2 overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                            <pre class="text-xs whitespace-pre-wrap">
                                { text_html_content }
                            </pre>
                        </div>


                        <div id="information" class="split pr-2 ">
                            <div id="regs" class="overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                                { self.render_running_registers() }
                            </div>


                            <div id="output">
                                <div style="height: 90%;" class="py-2 overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                                    <h1> <strong> {"Output"} </strong> </h1>
                                    <pre class="h-full whitespace-pre-wrap">
                                        {output_html_content}
                                    </pre>
                                </div>
                                <div style="height: 10%;" class="border-2 border-black">
                                    <input
                                        ref={self.input_ref.clone()}
                                        id="user_input"
                                        type="text"
                                        maxlength={
                                            match &self.state {
                                                State::Running(curr) => match &curr.input_needed {
                                                    Some(item) => match item {
                                                        ReadSyscalls::ReadInt => ""
                                                      //ReadSyscalls::ReadChar => "1"

                                                    },
                                                    None => {""}
                                                },
                                                State::NoFile => {
                                                    ""
                                                },
                                            }
                                        }
                                        disabled={
                                               match &self.state {
                                                    State::Running(curr) => {
                                                        if curr.input_needed.is_none() { true } else { false }
                                                    },
                                                    State::NoFile => { true },
                                                }

                                        }
                                        onkeydown={on_input_keydown}
                                        style="padding-left: 3px; height: 100%;"
                                        class={input_classes} placeholder="> ..."/>
                                </div>
                            </div>
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
        let decompiled = &state.decompiled;
        let runtime_instr = state.mips_state.current_instr.unwrap_or(0);
        html! {
            <table>
                <tbody>
                {
                     for decompiled.as_str().split("\n").into_iter().map(|item| {
                        if item == "" {
                            html! {
                                <tr>{"\u{00a0}"}</tr>
                            }
                        }
                        else {


                            let should_highlight = if item.starts_with("0x") {
                                let source_instr = u32::from_str_radix(&item[2..10], 16).unwrap_or(0);
                                source_instr == runtime_instr
                            } else {
                                false
                            };

                            if should_highlight {
                                html! {
                                    <tr class={"bg-th-highlighting"}>
                                        {item}
                                    </tr>
                                }
                            } else {

                                html!{
                                    <tr>
                                        {item}
                                    </tr>
                                }
                            }

                        }
                    })
                }
                </tbody>
            </table>
        }
    }

    fn render_running_output(&self, state: &RunningState) -> Html {
        html! {
            {state.mips_state.stdout.join("")}
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

    fn process_syscall_request(&mut self, mips_state: MipsState, required_type: ReadSyscalls) -> bool {
    
        match &mut self.state {
            State::Running(curr) => {
                curr.mips_state = mips_state;
                curr.input_needed = Some(required_type);

                if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
                    input.set_disabled(false);
                    input.focus().unwrap();
                };            
                true
            }

            State::NoFile => false,
        }

    }

    fn process_syscall_response(&mut self, input: HtmlInputElement, required_type: ReadSyscallInputs) {
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
            State::NoFile => {
                error!("Should not be possible to submit int with no file");
            }
        }
    }
}
