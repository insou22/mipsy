use mipsy_lib::inst::{InstSignature, CompileSignature, RuntimeMetadata, RuntimeSignature, InstMetadata, PseudoSignature, PseudoExpand};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

impl Into<InstSignature> for InstructionYaml {
    fn into(self) -> InstSignature {
        InstSignature::new(
            self.name.to_ascii_lowercase(),
            self.compile.into(),
            self.runtime.clone().into(),
            self.runtime.into(),
            InstMetadata::new(self.desc_short, self.desc_long),
        )
    }
}

impl Into<CompileSignature> for CompileYaml {
    fn into(self) -> CompileSignature {
        CompileSignature::new(
            self.format.into_iter().map(Into::into).collect(),
            self.relative_label,
        )
    }
}

impl Into<RuntimeSignature> for RuntimeYaml {
    fn into(self) -> RuntimeSignature {
        match self.inst_type {
            InstructionType::R => RuntimeSignature::R {
                opcode: self.opcode.unwrap_or(0),
                funct: self.funct.unwrap_or(0),
                shamt: self.shamt, rs: self.rs, rt: self.rt, rd: self.rd
            },
            InstructionType::I => RuntimeSignature::I {
                opcode: self.opcode.expect("I-type requires opcode"), rt: self.rt
            },
            InstructionType::J => RuntimeSignature::J {
                opcode: self.opcode.expect("J-type requires opcode")
            },
        }
    }
}

impl Into<RuntimeMetadata> for RuntimeYaml {
    fn into(self) -> RuntimeMetadata {
        RuntimeMetadata::new(self.reads.into_iter().map(Into::into).collect())
    }
}

impl Into<mipsy_lib::ArgumentType> for ArgumentType {
    fn into(self) -> mipsy_lib::ArgumentType {
        match self {
            ArgumentType::Rd      => mipsy_lib::ArgumentType::Rd,
            ArgumentType::Rs      => mipsy_lib::ArgumentType::Rs,
            ArgumentType::Rt      => mipsy_lib::ArgumentType::Rt,
            ArgumentType::Shamt   => mipsy_lib::ArgumentType::Shamt,
            ArgumentType::I16     => mipsy_lib::ArgumentType::I16,
            ArgumentType::U16     => mipsy_lib::ArgumentType::U16,
            ArgumentType::J       => mipsy_lib::ArgumentType::J,
            ArgumentType::OffRs   => mipsy_lib::ArgumentType::OffRs,
            ArgumentType::OffRt   => mipsy_lib::ArgumentType::OffRt,
            ArgumentType::F32     => mipsy_lib::ArgumentType::F32,
            ArgumentType::F64     => mipsy_lib::ArgumentType::F64,
            ArgumentType::Rx      => panic!("Rx is not a real register -- it must be macroed away"),
            ArgumentType::I32     => mipsy_lib::ArgumentType::I32,
            ArgumentType::U32     => mipsy_lib::ArgumentType::U32,
            ArgumentType::Off32Rs => mipsy_lib::ArgumentType::Off32Rs,
            ArgumentType::Off32Rt => mipsy_lib::ArgumentType::Off32Rt,
        }
    }
}

impl Into<mipsy_lib::inst::ReadsRegisterType> for ReadsRegisterType {
    fn into(self) -> mipsy_lib::inst::ReadsRegisterType {
        match self {
            ReadsRegisterType::Rs    => mipsy_lib::inst::ReadsRegisterType::Rs,
            ReadsRegisterType::Rt    => mipsy_lib::inst::ReadsRegisterType::Rt,
            ReadsRegisterType::OffRs => mipsy_lib::inst::ReadsRegisterType::OffRs,
            ReadsRegisterType::OffRt => mipsy_lib::inst::ReadsRegisterType::OffRt,
        }
    }
}

impl Into<PseudoSignature> for PseudoInstructionYaml {
    fn into(self) -> PseudoSignature {
        PseudoSignature::new(
            self.name.to_ascii_lowercase(),
            self.compile.into(),
            self.expand.into_iter().map(Into::into).collect(),
        )
    }
}

impl Into<PseudoExpand> for InstructionExpansionYaml {
    fn into(self) -> PseudoExpand {
        PseudoExpand::new(
            self.inst,
            self.data,
        )
    }
}
