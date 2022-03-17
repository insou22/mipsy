use std::collections::HashMap;

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
    // cannot be a big array due to serde not using const-generics yet
    // https://github.com/serde-rs/serde/issues/631
    pub memory: HashMap<u32, Vec<Safe<u8> /*; PAGE_SIZE] */>>,
    pub is_stepping: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorState {
    pub error: MipsyError,
    pub mipsy_stdout: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeErrorState {
    pub error: MipsyError,
    pub mips_state: MipsState,
    pub decompiled: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    CompilerOrParserError(ErrorState),
    RuntimeError(RuntimeErrorState),
}

impl MipsState {
    pub fn update_registers(&mut self, runtime: &Runtime) {
        self.previous_registers = runtime
            .timeline()
            .prev_state()
            .map(|state| state.registers().iter().cloned().collect())
            .unwrap_or_else(|| vec![Safe::Uninitialised; 32]);

        self.register_values = runtime
            .timeline()
            .state()
            .registers()
            .iter()
            .cloned()
            .collect();
    }

    pub fn update_current_instr(&mut self, runtime: &Runtime) {
        self.current_instr = runtime.timeline().prev_state().map(|state| state.pc());
    }

    pub fn update_memory(&mut self, runtime: &Runtime) {
        self.memory = runtime
            .timeline()
            .state()
            .pages()
            .iter()
            .map(|(key, val)| (*key, val.iter().copied().collect()))
            .collect()
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
    Error(ErrorType),
    Compiled(RunningState),
}
