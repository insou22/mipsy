use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{
    runtime::{JAL, JUMP, SPECIAL, SPECIAL2, SPECIAL3},
    Register, Runtime,
};
use std::{fmt::Display, ops::Sub};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TargetAction {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl TargetAction {
    // eq is used to check if a watchpoint should trigger based on the action,
    // so a watchpoint checking for both reads and writes should always trigger
    pub fn fits(&self, other: &Self) -> bool {
        match *self {
            TargetAction::ReadWrite => true,
            _ => match other {
                TargetAction::ReadWrite => true,
                _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            },
        }
    }
}

impl Sub for TargetAction {
    type Output = Option<TargetAction>;

    fn sub(self, rhs: Self) -> Self::Output {
        match rhs {
            TargetAction::ReadWrite => None,
            rhs => match self {
                TargetAction::ReadWrite => Some(match rhs {
                    TargetAction::ReadOnly => TargetAction::WriteOnly,
                    TargetAction::WriteOnly => TargetAction::ReadOnly,
                    TargetAction::ReadWrite => unreachable!(),
                }),
                TargetAction::ReadOnly => {
                    if rhs == TargetAction::ReadOnly {
                        None
                    } else {
                        Some(TargetAction::ReadWrite)
                    }
                }
                TargetAction::WriteOnly => {
                    if rhs == TargetAction::WriteOnly {
                        None
                    } else {
                        Some(TargetAction::ReadWrite)
                    }
                }
            },
        }
    }
}

impl Display for TargetAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                TargetAction::ReadOnly => "read",
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
        write!(
            f,
            "{}",
            match self {
                WatchpointTarget::Register(r) => r.to_string(),
                WatchpointTarget::MemAddr(m) => format!("{}{:08x}", "0x".yellow(), m),
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

const LB: u32 = 0b100000;
const LBU: u32 = 0b100100;
const LH: u32 = 0b100001;
const LHU: u32 = 0b100101;
const LUI: u32 = 0b001111;
const LW: u32 = 0b100011;
const LWU: u32 = 0b100111;
const SB: u32 = 0b101000;
const SH: u32 = 0b101001;
const SW: u32 = 0b101011;

pub fn get_affected_registers(runtime: &Runtime, inst: u32) -> Vec<TargetWatch> {
    let opcode = inst >> 26;
    let rb = (inst >> 21) & 0x1F;
    let rs = (inst >> 21) & 0x1F;
    let rt = (inst >> 16) & 0x1F;
    let rd = (inst >> 11) & 0x1F;
    let offset = (inst & 0xFF) as i32;

    match opcode {
        LB | LBU | LH | LHU | LW | LWU | LUI => vec![
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rt).unwrap()),
                action: TargetAction::WriteOnly,
            },
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rb).unwrap()),
                action: TargetAction::ReadOnly,
            },
            TargetWatch {
                target: WatchpointTarget::MemAddr(
                    (runtime
                        .timeline()
                        .prev_state()
                        .expect("there should be a previous state")
                        .read_register(rb)
                        .expect("uninitialized read should already have been handled")
                        + offset) as u32,
                ),
                action: TargetAction::ReadOnly,
            },
        ],
        SB | SH | SW => vec![
            TargetWatch {
                target: WatchpointTarget::MemAddr(
                    (runtime
                        .timeline()
                        .state()
                        .read_register(rb)
                        .expect("uninitialized read should already have been handled")
                        + offset) as u32,
                ),
                action: TargetAction::WriteOnly,
            },
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rt).unwrap()),
                action: TargetAction::ReadOnly,
            },
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rb).unwrap()),
                action: TargetAction::ReadOnly,
            },
        ],
        SPECIAL | SPECIAL2 | SPECIAL3 => vec![
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rd).unwrap()),
                action: TargetAction::WriteOnly,
            },
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rs).unwrap()),
                action: TargetAction::ReadOnly,
            },
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rt).unwrap()),
                action: TargetAction::ReadOnly,
            },
        ],
        JUMP => vec![],
        JAL => vec![
            TargetWatch {
                target: WatchpointTarget::Register(Register::Ra),
                action: TargetAction::WriteOnly,
            }
        ],
        _ => vec![
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rs).unwrap()),
                action: TargetAction::ReadOnly,
            },
            TargetWatch {
                target: WatchpointTarget::Register(Register::from_u32(rt).unwrap()),
                action: TargetAction::WriteOnly,
            },
        ],
    }
}
