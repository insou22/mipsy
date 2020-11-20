use crate::inst::instruction::Signature;
use mipsy_parser::MPInstruction;
use crate::inst::instruction::GenericSignature;

#[derive(Debug, Clone)]
pub enum CompileError {
    YamlMissingFunct(String),   // users should never see this
    YamlMissingOpcode(String),  // users should never see this
    MultipleMatchingInstructions(Vec<GenericSignature>), // users should never see this

    ParseFailure { line: u32, col: usize },

    NumRegisterOutOfRange(i32),
    NamedRegisterOutOfRange { reg_name: char, reg_index: i32 },
    UnknownRegister(String),

    UnknownInstruction(MPInstruction),
    InstructionBadFormat(MPInstruction, Vec<Signature>),
    InstructionSimName(MPInstruction, Vec<Signature>),

    UnresolvedLabel(String, Vec<String>),
}
