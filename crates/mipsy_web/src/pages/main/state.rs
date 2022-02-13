use crate::pages::main::app::ReadSyscalls;
use mipsy_lib::{MipsyError, Runtime, Safe};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq)]
pub enum DisplayedTab {
    Source,
    Decompiled,
    Data,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MipsState {
    pub stdout: Vec<String>,
    pub mipsy_stdout: Vec<String>,
    pub exit_status: Option<i32>,
    pub register_values: Vec<Safe<i32>>,
    pub previous_registers: Vec<Safe<i32>>,
    pub current_instr: Option<u32>,
    pub is_stepping: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompilerErrorState {
    pub error: MipsyError,
    pub mipsy_stdout: Vec<String>,
}

impl MipsState {
    pub fn update_registers(&mut self, runtime: &Runtime) {
        self.previous_registers = self.register_values.clone();
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

#[derive(Clone, PartialEq, Debug)]
pub struct RunningState {
    pub decompiled: String,
    pub mips_state: MipsState,
    pub should_kill: bool,
    pub input_needed: Option<ReadSyscalls>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    NoFile,
    CompilerError(CompilerErrorState),
    Compiled(RunningState),
}
