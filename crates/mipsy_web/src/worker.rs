use mipsy_lib::{Binary, InstSet, MipsyError, Runtime};
use mipsy_parser::TaggedFile;
use serde::{Deserialize, Serialize};
use yew::worker::{Agent, AgentLink, HandlerId, Public};

pub struct Worker {
    // the link that allows to communicate to main thread
    link: AgentLink<Self>,
    inst_set: InstSet,
    // the runtime may not exist if no binary
    runtime: Option<RuntimeState>,
    // the binary will not exist if we have not been sent a file
    // realistically we should have a state that encapsulates binary and runtime
    // but that's a shift in the worker's behaviour
    // we can do that later
    binary: Option<Binary>,
}

type Guard<T> = Box<dyn FnOnce(T) -> Runtime>;
enum RuntimeState {
    Running(Runtime),
    WaitingInt(Guard<i32>),
    WaitingFloat(Guard<f32>),
    WaitingDouble(Guard<f64>),
    WaitingString(Guard<Vec<u8>>),
    WaitingChar(Guard<u8>),
    WaitingOpen(Guard<i32>),
    WaitingRead(Guard<(i32, Vec<u8>)>),
    WaitingWrite(Guard<i32>),
    WaitingClose(Guard<i32>),
    Stopped,
}

type File = String;

#[derive(Serialize, Deserialize)]
pub enum WorkerRequest {
    // The struct that worker can obtain
    CompileCode(File),
}

#[derive(Serialize, Deserialize)]
pub enum WorkerResponse {
    DecompiledCode(String),
    CompilerError(MipsyError),
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = WorkerRequest;
    type Output = WorkerResponse;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            inst_set: mipsy_codegen::instruction_set!("../../mips.yaml"),
            runtime: None,
            binary: None,
        }
    }

    fn name_of_resource() -> &'static str {
        "wasm.js"
    }

    fn update(&mut self, msg: Self::Message) {
        // no messaging exists
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Self::Input::CompileCode(f) => {
                let compiled =
                    mipsy_lib::compile(&self.inst_set, vec![TaggedFile::new(None, f.as_str())], 8);

                match compiled {
                    Ok(binary) => {
                        let decompiled = mipsy_lib::decompile(&self.inst_set, &binary);
                        let response = Self::Output::DecompiledCode(decompiled);
                        self.link.respond(id, response)
                    }

                    Err(err) => self.link.respond(id, Self::Output::CompilerError(err)),
                }
            }
        }
    }
}
