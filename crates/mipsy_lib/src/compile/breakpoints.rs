use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::Register;
use std::fmt::Display;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TargetAction {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl PartialEq for TargetAction {
    // eq is used to check if a watchpoint should trigger based on the action,
    // so a watchpoint checking for both reads and writes should always trigger
    fn eq(&self, other: &Self) -> bool {
        match *self {
            TargetAction::ReadWrite => true,
            _ => match other {
                    TargetAction::ReadWrite => true,
                    _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }
}

impl Display for TargetAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match *self {
                TargetAction::ReadOnly  => "read",
                TargetAction::WriteOnly => "write",
                TargetAction::ReadWrite => "read/write",
            }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchpointTarget {
    Register(Register),
    MemAddr(u32),
}

impl Display for WatchpointTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                WatchpointTarget::Register(r) => r.to_string(),
                WatchpointTarget::MemAddr(m)  => format!("{}{:08x}", "0x".yellow(), m),
            }
        )
    }
}

#[derive(PartialEq)]
pub struct TargetWatch {
    pub target: WatchpointTarget,
    pub action: TargetAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Watchpoint {
    pub id: u32,
    pub action: TargetAction,
    pub ignore_count: u32,
    pub enabled: bool,
    pub commands: Vec<String>,
}

impl Watchpoint {
    pub fn new(id: u32, action: TargetAction) -> Self {
        Self {
            id,
            action,
            ignore_count: 0,
            enabled: true,
            commands: Vec::new(),
        }
    }
}

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Breakpoint {
    pub id: u32,
    pub enabled: bool,
    pub commands: Vec<String>,
    pub ignore_count: u32,
}

impl Breakpoint {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            enabled: true,
            commands: Vec::new(),
            ignore_count: 0,
        }
    }
}

pub trait Point {
    fn get_id(&self) -> u32;
    fn get_commands(&'_ mut self) -> &'_ mut Vec<String>;
}

impl Point for Breakpoint {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_commands(&'_ mut self) -> &'_ mut Vec<String> {
        &mut self.commands
    }
}

impl Point for Watchpoint {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_commands(&'_ mut self) -> &'_ mut Vec<String> {
        &mut self.commands
    }
}
