use mipsy_codegen::instruction_set;
use mipsy_lib::InstSet;
use mipsy_parser::TaggedFile;
use serde::{Deserialize, Serialize};
use yew::{
    services::reader::FileData,
    worker::{Agent, AgentLink, HandlerId, Public},
};

pub struct Worker {
    // the link that allows to communicate to main thread
    link: AgentLink<Self>,
    inst_set: InstSet,
}

type File = String;

#[derive(Serialize, Deserialize)]
pub enum Request {
    // The struct that worker can obtain
    CompileCode(File),
}

#[derive(Serialize, Deserialize)]
pub enum WorkerOutput {
    CompiledCode(mipsy_lib::Binary),
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = ();
    type Input = Request;
    type Output = WorkerOutput;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            inst_set: mipsy_codegen::instruction_set!("../../mips.yaml"),
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
            Request::CompileCode(f) => {
                let compiled =
                    mipsy_lib::compile(&self.inst_set, vec![TaggedFile::new(None, f.as_str())], 8);
            }
        }
    }
}
