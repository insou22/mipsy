use crate::inst::instruction::GenericSignature;

#[derive(Debug)]
pub enum CompileError {
    Str(&'static str),
    String(String),

    YamlMissingFunct(String),   // users should never see this
    YamlMissingOpcode(String),  // users should never see this
    MultipleMatchingInstructions(Vec<GenericSignature>), // users should never see this

    NumRegisterOutOfRange(i32),
    NamedRegisterOutOfRange { reg_name: char, reg_index: i32 },
    UnknownRegister(String),

    UnknownInstruction(String),
    InstructionBadFormat(String),

    UnresolvedLabel(String),
}
