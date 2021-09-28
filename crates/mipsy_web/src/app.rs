use crate::{
    components::{navbar::NavBar, pagebackground::PageBackground},
    worker::{Worker, WorkerRequest, WorkerResponse},
};
use mipsy_lib::{Binary, InstSet};
use yew::{
    prelude::*,
    services::{
        reader::{FileData, ReaderTask},
        ConsoleService, ReaderService,
    },
    web_sys::File,
};

fn crimes<T>() -> T {
    panic!()
}

pub struct RunningState {
    decompiled: String,
    mips_state: MipsState,
}

pub struct MipsState {
    stdout: Vec<String>,
    exit_status: Option<i32>,
}

pub enum Msg {
    FileChanged(File),
    FileRead(FileData),
    Run,
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
    inst_set: InstSet,
    worker: Box<dyn Bridge<Worker>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let worker = Worker::bridge(link.callback(Self::Message::FromWorker));
        Self {
            link,
            state: State::NoFile,
            tasks: vec![],
            inst_set: mipsy_codegen::instruction_set!("../../mips.yaml"),
            worker,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FileChanged(file) => {
                ConsoleService::info("file changed msg");
                // FIXME -- check result
                let result = ReaderService::read_file(
                    file,
                    self.link.callback(|file_data| Msg::FileRead(file_data)),
                );

                match result {
                    Ok(service) => self.tasks.push(service),

                    Err(err) => {
                        ConsoleService::error(&format!("{:?}", err));
                    }
                }
                false
            }

            Msg::FileRead(file_data) => {
                ConsoleService::info(&format!("{:?}", file_data));
                // TODO -- this should not be lossy

                let file = String::from_utf8_lossy(&file_data.content).to_string();

                let input = WorkerRequest::CompileCode(file);
                ConsoleService::info("sending to worker");
                self.worker.send(input);

                true
            }

            Msg::Run => {
                todo!()
                /*
                if let State::Running(RunningState {
                    decompiled,
                    mips_state,
                }) = &mut self.state
                {
                    // clears stdout
                    mips_state.stdout.drain(..);

                    //let mut rh = Handler {exit_status: 0, exit_status: None, stdout};
                    let mut runtime = mipsy_lib::runtime(&binary, &[]);
                    loop {
                        // dont error pls
                        runtime.step(mips_state);

                        if mips_state.exit_status.is_some() {
                            break;
                        }
                    }

                    mips_state.stdout.push(format!(
                        "\nProgram exited with exit status {}",
                        mips_state
                            .exit_status
                            .expect("infinite loop guarantees Some return")
                    ));
                } else {
                    ConsoleService::error("No File loaded, cannot run");
                    return false;
                }
                true*/
            }

            Msg::StepForward => {
                todo!();
            }

            Msg::StepBackward => {
                todo!();
            }

            Msg::FromWorker(worker_output) => match worker_output {
                WorkerResponse::DecompiledCode(decompiled) => {
                    ConsoleService::info("recieved decompiled code from worker");
                    ConsoleService::info(&decompiled);
                    match self.state {
                        // Overwrite existing state,
                        State::NoFile | State::Running(_) => {
                            self.state = State::Running(RunningState {
                                decompiled,
                                mips_state: MipsState {
                                    stdout: Vec::new(),
                                    exit_status: None,
                                },
                            });
                            true
                        }
                    }
                }
                WorkerResponse::CompilerError(err) => {
                    todo!();
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
            ConsoleService::info("onchange fired");
            match event {
                ChangeData::Files(file_list) => {
                    if let Some(file) = file_list.item(0) {
                        ConsoleService::info(&format!("{:?}", file.name()));
                        Some(Msg::FileChanged(file))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        });

        let run_onclick = self.link.callback(|_| {
            ConsoleService::info("Run fired");
            Msg::Run
        });

        let text_html_content = match &self.state {
            &State::NoFile => "no file loaded".into(),
            &State::Running(ref state) => self.render_running(state),
        };

        let output_html_content = match &self.state {
                    &State::NoFile => "mipsy_web v0.1\nSchool of Computer Science and Engineering, University of New South Wales, Sydney.".into(),
                    &State::Running(ref state) => self.render_running_output(state),
        };

        ConsoleService::info("rendering");
        html! {
            <>
                <PageBackground>
                    <NavBar load_onchange=onchange run_onclick=run_onclick />
                    <div id="pageContentContainer" style="height: calc(100vh - 122px)">
                        <div id="text" class="flex flex-row px-2 ">
                            <div id="regs" class="overflow-y-auto bg-gray-300 px-2 border-2 border-gray-600">
                                {"Register garbage"}
                            </div>
                            <div id="text_data" class="overflow-y-auto bg-gray-300 px-2 border-2 border-gray-600">
                                <pre class="text-xs whitespace-pre-wrap">
                                { text_html_content }
                                </pre>
                            </div>
                        </div>

                        <div id="output" class="overflow-y-auto bg-gray-300 px-2 border-2 border-gray-600">
                            <pre class="h-full whitespace-pre-wrap">
                                {output_html_content}
                            </pre>
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
}
