use crate::{
    components::{navbar::NavBar, pagebackground::PageBackground},
    worker::{Worker, WorkerRequest, WorkerResponse},
};

use mipsy_lib::{Register, Safe};
use serde::{Deserialize, Serialize};
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

pub struct RunningState {
    decompiled: String,
    mips_state: MipsState,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MipsState {
    pub stdout: Vec<String>,
    pub exit_status: Option<i32>,
    pub register_values: Vec<Safe<i32>>,
}

pub enum Msg {
    FileChanged(File),
    FileRead(FileData),
    Run,
    Reset,
    StepForward,
    StepBackward,
    FromWorker(WorkerResponse),
}

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
}

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
                if let State::Running(RunningState {
                    decompiled: _,
                    mips_state,
                }) = &mut self.state
                {
                    info!("Sending Run Code Instr");
                    let input = WorkerRequest::RunCode(mips_state.clone());
                    self.worker.send(input);
                } else {
                    info!("No File loaded, cannot run");
                    return false;
                }
                true
            }

            Msg::StepForward => {
                todo!();
            }

            Msg::StepBackward => {
                todo!();
            }

            Msg::Reset => {
                if let State::Running(RunningState {
                    decompiled: _,
                    mips_state,
                }) = &mut self.state
                {
                    warn!("Sending Reset Instr");
                    let input = WorkerRequest::ResetRuntime(mips_state.clone());
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
                                },
                            });
                            true
                        }
                    }
                }

                WorkerResponse::CompilerError(err) => {
                    todo!();
                }

                WorkerResponse::MipsyState(mips_state) => {
                    warn!("recieved mips_state response");
                    match &mut self.state {
                        State::Running(curr) => {
                            curr.mips_state = mips_state;
                            true
                        }

                        State::NoFile => false,
                    }
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

        let run_onclick = self.link.callback(|_| {
            info!("run fired");
            Msg::Run
        });

        let reset_onclick = self.link.callback(|_| Msg::Reset);

        let text_html_content = match &self.state {
            &State::NoFile => "no file loaded".into(),
            &State::Running(ref state) => self.render_running(state),
        };

        let output_html_content = match &self.state {
                    &State::NoFile => "mipsy_web beta\nSchool of Computer Science and Engineering, University of New South Wales, Sydney.".into(),
                    &State::Running(ref state) => self.render_running_output(state),
        };

        info!("rendering");
        html! {
            <>
                <PageBackground>
                    <NavBar load_onchange=onchange reset_onclick=reset_onclick run_onclick=run_onclick />
                    <div id="pageContentContainer" class="split flex flex-row" style="height: calc(100vh - 122px)">
                        <div id="source_file" class="py-2 overflow-y-auto bg-gray-300 px-2 border-2 border-gray-600">
                            <pre class="text-xs whitespace-pre-wrap">
                                { text_html_content }
                            </pre>
                        </div>


                        <div id="information" class="split pr-2 ">
                            <div id="regs" class="overflow-y-auto bg-gray-301 px-2 border-2 border-gray-600">
                                { self.render_running_registers() }
                            </div>

                           <div id="output" class="py-2 overflow-y-auto bg-gray-300 px-2 border-2 border-gray-600">
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
        html! {
            {&state.decompiled}
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
