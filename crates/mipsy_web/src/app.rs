use crate::{
    components::{navbar::NavBar, modal::Modal, pagebackground::PageBackground},
    worker::{Worker, WorkerRequest, WorkerResponse},
};
use mipsy_lib::{Register, Safe};
use serde::{Deserialize, Serialize};
use std::u32;
use yew::{
    prelude::*,
    services::{
        reader::{FileData, ReaderTask},
        ReaderService,
    },
    web_sys::File,
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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MipsState {
    pub stdout: Vec<String>,
    pub exit_status: Option<i32>,
    pub register_values: Vec<Safe<i32>>,
    pub current_instr: Option<u32>,
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
                if let State::Running(RunningState {
                    decompiled: _,
                    mips_state,
                    should_kill: _,
                }) = &mut self.state
                {
                    let input = WorkerRequest::Run(mips_state.clone(), 1);
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
                    let input = WorkerRequest::ResetRuntime(curr.mips_state.clone());
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot run");
                    return false;
                }
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
        let onchange = self.link.batch_callback(|event| {
            info!("onchange fired");
            match event {
                ChangeData::Files(file_list) => {
                    if let Some(file) = file_list.item(0) {
                        Some(Msg::FileChanged(file))
                    } else {
                        None
                    }
                }
                _ => None,
            }
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

        let classes = if self.display_modal {
            "modal bg-th-primary border-black border-2 absolute top-1/4 h-1/3 w-3/4"
        } else {
            "modal hidden"
        };

        let modal_overlay_classes = if self.display_modal {
            "bg-th-secondary bg-opacity-90 absolute top-0 left-0 h-screen w-screen"
        } else {
            "hidden"
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

                           <div id="output" class="py-2 overflow-y-auto bg-th-secondary px-2 border-2 border-gray-600">
                                <h1> <strong> {"Output"} </strong> </h1>
                                <pre class="h-full whitespace-pre-wrap">
                                    {output_html_content}
                                </pre>
                            </div>
                        </div>

                    </div>

                </PageBackground>

            </>
        }
    }
}

impl App {
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
}
