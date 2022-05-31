use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct YamlFile {
    pub instructions: Vec<InstructionYaml>,
    pub pseudoinstructions: Vec<PseudoInstructionYaml>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InstructionYaml {
    pub name: String,
    pub desc_short: Option<String>,
    pub desc_long: Option<String>,
    pub compile: CompileYaml,
    pub runtime: RuntimeYaml,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CompileYaml {
    pub format: Vec<ArgumentType>,
    #[serde(default)]
    pub relative_label: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeYaml {
    #[serde(rename = "type")]
    pub inst_type: InstructionType,
    pub opcode: Option<u8>,
    pub funct: Option<u8>,
    pub shamt: Option<u8>,
    pub rs: Option<u8>,
    pub rt: Option<u8>,
    pub rd: Option<u8>,
    pub reads: Vec<ReadsRegisterType>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum InstructionType {
    R,
    I,
    J,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PseudoInstructionYaml {
    pub name: String,
    pub desc_short: Option<String>,
    pub desc_long: Option<String>,
    pub compile: CompileYaml,
    pub expand: Vec<InstructionExpansionYaml>,
    #[serde(default)]
    pub only_derive: bool,
    #[serde(default)]
    pub derives: Vec<DeriveStatementYaml>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum DeriveStatementYaml {
    Imm2Reg {
        register: String,
        imm_types: Vec<Imm2RegImmType>,
        #[serde(default)]
        sign_extend: bool,
        #[serde(default)]
        derives: Vec<DeriveStatementYaml>,
        imm_register: Option<String>,
    },
    DefaultValue {
        value: ArgumentType,
        default: String,
        #[serde(default)]
        derives: Vec<DeriveStatementYaml>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Imm2RegImmType {
    I16,
    U16,
    I32,
    U32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct InstructionExpansionYaml {
    pub inst: String,
    pub data: Vec<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgumentType {
    Rd,
    Rs,
    Rt,
    Shamt,
    I16,
    U16,
    J,
    OffRs,
    OffRt,
    F32,
    F64,

    // pseudo
    Rx,
    I32,
    U32,
    Off32Rs,
    Off32Rt,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadsRegisterType {
    Rs,
    Rt,
    OffRs,
    OffRt,
}

impl Display for ArgumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArgumentType::Rd => write!(f, "Rd"),
            ArgumentType::Rs => write!(f, "Rs"),
            ArgumentType::Rt => write!(f, "Rt"),
            ArgumentType::Shamt => write!(f, "Shamt"),
            ArgumentType::I16 => write!(f, "I16"),
            ArgumentType::U16 => write!(f, "U16"),
            ArgumentType::J => write!(f, "J"),
            ArgumentType::OffRs => write!(f, "OffRs"),
            ArgumentType::OffRt => write!(f, "OffRt"),
            ArgumentType::F32 => write!(f, "F32"),
            ArgumentType::F64 => write!(f, "F64"),
            ArgumentType::Rx => write!(f, "Rx"),
            ArgumentType::I32 => write!(f, "I32"),
            ArgumentType::U32 => write!(f, "U32"),
            ArgumentType::Off32Rs => write!(f, "Off32Rs"),
            ArgumentType::Off32Rt => write!(f, "Off32Rt"),
        }
    }
}

impl From<InstructionType> for super::base::InstructionType {
    fn from(x: InstructionType) -> super::base::InstructionType {
        match x {
            InstructionType::R => super::base::InstructionType::R,
            InstructionType::I => super::base::InstructionType::I,
            InstructionType::J => super::base::InstructionType::J,
        }
    }
}

impl From<ArgumentType> for super::base::ArgumentType {
    fn from(x: ArgumentType) -> super::base::ArgumentType {
        match x {
            ArgumentType::Rd      => super::base::ArgumentType::Rd,
            ArgumentType::Rs      => super::base::ArgumentType::Rs,
            ArgumentType::Rt      => super::base::ArgumentType::Rt,
            ArgumentType::Shamt   => super::base::ArgumentType::Shamt,
            ArgumentType::I16     => super::base::ArgumentType::I16,
            ArgumentType::U16     => super::base::ArgumentType::U16,
            ArgumentType::J       => super::base::ArgumentType::J,
            ArgumentType::OffRs   => super::base::ArgumentType::OffRs,
            ArgumentType::OffRt   => super::base::ArgumentType::OffRt,
            ArgumentType::F32     => super::base::ArgumentType::F32,
            ArgumentType::F64     => super::base::ArgumentType::F64,
            ArgumentType::I32     => super::base::ArgumentType::I32,
            ArgumentType::U32     => super::base::ArgumentType::U32,
            ArgumentType::Off32Rs => super::base::ArgumentType::Off32Rs,
            ArgumentType::Off32Rt => super::base::ArgumentType::Off32Rt,
            ArgumentType::Rx      => super::base::ArgumentType::Rx,
        }
    }
}

impl From<InstructionExpansionYaml> for super::base::InstructionExpansionYaml {
    fn from(x: InstructionExpansionYaml) -> super::base::InstructionExpansionYaml {
        super::base::InstructionExpansionYaml {
            inst: x.inst,
            data: x.data,
        }
    }
}

impl From<ReadsRegisterType> for super::base::ReadsRegisterType {
    fn from(x: ReadsRegisterType) -> super::base::ReadsRegisterType {
        match x {
            ReadsRegisterType::Rs => super::base::ReadsRegisterType::Rs,
            ReadsRegisterType::Rt => super::base::ReadsRegisterType::Rt,
            ReadsRegisterType::OffRs => super::base::ReadsRegisterType::OffRs,
            ReadsRegisterType::OffRt => super::base::ReadsRegisterType::OffRt,
        }
    }
}
