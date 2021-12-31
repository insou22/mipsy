use crate::worker::ReadSyscallInputs;
use crate::{
    components::{
        decompiled::DecompiledCode, modal::Modal, navbar::NavBar, outputarea::OutputArea,
        pagebackground::PageBackground, registers::Registers, sourcecode::SourceCode,
    },
    pages::main::{
        state::{MipsState, RunningState},
        update,
    },
    worker::{Worker, WorkerRequest, WorkerResponse},
};
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use gloo_file::FileReadError;
use log::{error, info, trace};
use mipsy_lib::MipsyError;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

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

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    NoFile,
    CompilerError(MipsyError),
    Running(RunningState),
}

pub struct App {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    // a tasks vec to keep track of any file uploads
    pub state: State,
    pub worker: Box<dyn Bridge<Worker>>,
    pub display_modal: bool,
    pub show_io: bool,
    pub input_ref: NodeRef,
    pub filename: Option<String>,
    pub file: Option<String>,
    pub show_source: bool,
    pub tasks: Vec<FileReader>,
}

pub const NUM_INSTR_BEFORE_RESPONSE: i32 = 40;

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let worker = Worker::bridge(ctx.link().callback(Self::Message::FromWorker));
        wasm_logger::init(wasm_logger::Config::default());
        Self {
            state: State::NoFile,
            worker,
            display_modal: false,
            show_io: true,
            input_ref: NodeRef::default(),
            filename: None,
            file: None,
            show_source: false,
            tasks: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let test = update::handle_update(self, ctx, msg);
        info!("re-render: {:?}", test);
        test
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        unsafe {
            crate::highlight();
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        /*    CALLBACKS   */
        let load_onchange = ctx.link().batch_callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Some(file_list) = input.files() {
                if let Some(file_blob) = file_list.item(0) {
                    let file = File::from(web_sys::File::from(file_blob));
                    Some(Msg::FileChanged(file))
                } else {
                    None
                }
            } else {
                None
            }
        });

        let on_input_keydown = ctx.link().callback(|event: KeyboardEvent| {
            if event.key() == "Enter" {
                return Msg::SubmitInput;
            };

            Msg::ProcessKeypress(event)
        });

        let run_onclick = ctx.link().callback(|_| Msg::Run);
        let kill_onclick = ctx.link().callback(|_| Msg::Kill);
        let reset_onclick = ctx.link().callback(|_| Msg::Reset);
        let step_forward_onclick = ctx.link().callback(|_| Msg::StepForward);
        let step_back_onclick = ctx.link().callback(|_| Msg::StepBackward);
        let open_modal_onclick = ctx.link().callback(|_| Msg::OpenModal);
        let show_io_tab = ctx.link().callback(|_| Msg::ShowIoTab);
        let show_mipsy_tab = ctx.link().callback(|_| Msg::ShowMipsyTab);
        let show_source_tab = ctx.link().callback(|_| Msg::ShowSourceTab);
        let show_decompiled_tab = ctx.link().callback(|_| Msg::ShowDecompiledTab);
        let noop_onclick = ctx.link().callback(|_| Msg::NoOp);

        /* HELPER FNS */
        let text_html_content = match &self.state {
            State::NoFile => "no file loaded".into(),
            State::CompilerError(_error) => "File has syntax errors, check your file with mipsy and try again.\nMipsy Web does not yet support displaying compiler errors".into(),
            State::Running(state) => self.render_running(state.clone()),
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

        let input_needed = match &self.state {
            State::Running(curr) => curr.input_needed.clone(),
            State::NoFile | State::CompilerError(_) => None,
        };

        info!("render");

        html! {
            <>
                <div onclick={open_modal_onclick.clone()} class={modal_overlay_classes}></div>

                <Modal should_display={self.display_modal} toggle_modal_onclick={open_modal_onclick.clone()} />

                <PageBackground>

                    <NavBar
                        {step_back_onclick}
                        {step_forward_onclick}
                        {reset_onclick}
                        {run_onclick}
                        {kill_onclick}
                        {noop_onclick}
                        {open_modal_onclick}
                        {file_loaded}
                        {waiting_syscall}
                        {exit_status}
                        {load_onchange}
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
                                <Registers state={self.state.clone()}/>
                            </div>

                            <OutputArea
                                {show_io_tab}
                                {show_mipsy_tab}
                                {mipsy_output_tab_title}
                                {input_needed}
                                show_io={self.show_io}
                                input_ref={&self.input_ref}
                                on_input_keydown={on_input_keydown.clone()}
                                running_output={self.render_running_output()}
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
    pub fn is_nav_or_special_key(&self, event: &KeyboardEvent) -> bool {
        if event.alt_key() || event.ctrl_key() || event.meta_key() {
            return true;
        }

        match event.key().as_str() {
            "Backspace" => true,
            "-" => true,
            _ => false,
        }
    }

    fn render_running(&self, state: RunningState) -> Html {
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
                        if self.show_source {
                            <SourceCode file={self.file.clone()} />
                        } else {
                            <DecompiledCode {state} />
                        }
                    </tbody>
                </table>
            </>
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

    pub fn process_syscall_request(
        &mut self,
        mips_state: MipsState,
        required_type: ReadSyscalls,
    ) -> bool {
        match self.state {
            State::Running(ref mut curr) => {
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

    pub fn process_syscall_response(
        &mut self,
        input: HtmlInputElement,
        required_type: ReadSyscallInputs,
    ) {
        match self.state {
            State::Running(ref mut curr) => {
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
