use crate::components::{navbar::NavBar, pagebackground::PageBackground};
use mipsy_lib::{Binary, InstSet, RuntimeHandler};
use mipsy_parser::TaggedFile;
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

struct Handler {
    exited: bool,
}

impl RuntimeHandler for Handler {
    fn sys1_print_int(&mut self, val: i32) {
        ConsoleService::info(val.to_string().as_str());
    }

    fn sys2_print_float(&mut self, val: f32) {
        todo!()
    }

    fn sys3_print_double(&mut self, val: f64) {
        todo!()
    }

    fn sys4_print_string(&mut self, val: String) {
        todo!()
    }

    fn sys5_read_int(&mut self) -> i32 {
        42
    }

    fn sys6_read_float(&mut self) -> f32 {
        todo!()
    }

    fn sys7_read_double(&mut self) -> f64 {
        todo!()
    }

    fn sys8_read_string(&mut self, max_len: u32) -> String {
        todo!()
    }

    fn sys9_sbrk(&mut self, val: i32) {
        todo!()
    }

    fn sys10_exit(&mut self) {
        self.exited = true;
    }

    fn sys11_print_char(&mut self, val: char) {
        ConsoleService::info(val.to_string().as_str());
    }

    fn sys12_read_char(&mut self) -> char {
        todo!()
    }

    fn sys13_open(
        &mut self,
        path: String,
        flags: mipsy_lib::flags,
        mode: mipsy_lib::mode,
    ) -> mipsy_lib::fd {
        todo!()
    }

    fn sys14_read(
        &mut self,
        fd: mipsy_lib::fd,
        buffer: mipsy_lib::void_ptr,
        len: mipsy_lib::len,
    ) -> mipsy_lib::n_bytes {
        todo!()
    }

    fn sys15_write(
        &mut self,
        fd: mipsy_lib::fd,
        buffer: mipsy_lib::void_ptr,
        len: mipsy_lib::len,
    ) -> mipsy_lib::n_bytes {
        todo!()
    }

    fn sys16_close(&mut self, fd: mipsy_lib::fd) {
        todo!()
    }

    fn sys17_exit_status(&mut self, val: i32) {
        self.exited = true;
    }

    fn breakpoint(&mut self) {
        todo!()
    }
}

pub enum Msg {
    FileChanged(File),
    FileRead(FileData),
    Run,
}

pub struct RunningState {
    file: String,
    binary: Binary,
    decompiled: String,
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
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: State::NoFile,
            tasks: vec![],
            inst_set: mipsy_codegen::instruction_set!("../../mips.yaml"),
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

                let compiled = mipsy_lib::compile(
                    &self.inst_set,
                    vec![TaggedFile::new(None, file.as_str())],
                    8,
                );

                match compiled {
                    Ok(binary) => {
                        let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                        match self.state {
                            // Overwrite existing state,
                            State::NoFile | State::Running(_) => {
                                self.state = State::Running(RunningState {
                                    file,
                                    binary,
                                    decompiled,
                                })
                            }
                        }
                    }

                    Err(err) => {
                        // this should eventually go to a new state
                        // errorstate
                        // and show on page
                        ConsoleService::error(&format!("{:?}", err));
                    }
                }

                true
            }

            Msg::Run => {
                if let State::Running(RunningState {
                    file,
                    binary,
                    decompiled,
                }) = &self.state
                {
                    let mut rh = Handler { exited: false };
                    let mut runtime = mipsy_lib::runtime(&binary, &[]);
                    loop {
                        runtime.step(&mut rh);
                        if rh.exited {
                            break;
                        }
                    }
                } else {
                    ConsoleService::error("No File, cannot run");
                    return false;
                }

                true
            }
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

        let inner = match &self.state {
            &State::NoFile => "no file loaded".into(),
            &State::Running(ref state) => self.render_running(state),
        };
        
        html! {
            <>
                <PageBackground>
                    <NavBar load_onchange=onchange run_onclick=run_onclick />
                    <div id="pageContentContainer" style="height: calc(100vh - 122px)">
                        <div id="text" class="flex flex-row px-2 ">
                            <div id="regs" class="px-2 border-2 border-gray-600">
                                {"Register garbage"}
                            </div>
                            <div id="text_data" class="px-2 border-2 border-gray-600">
                                { inner }
                            </div>
                        </div>
                    
                        <div id="output" class="border-2 border-gray-600">
                        {"Mipsy Output Here"}
                        </div>
                    
                    </div>

                </PageBackground>

            </>
        }
    }
}

impl App {
    fn render_running(&self, state: &RunningState) -> Html {
        state
            .file
            .split("\n")
            .map(|line| {
                html! {
                    <p>
                        {line}
                    </p>
                }
            })
            .collect::<Html>()
    }
}
