use crate::inst::instruction::InstSignature;
use crate::inst::instruction::InstFormat;
use crate::inst::instruction::SimpleArgType;
use crate::compile::context::Token;
use crate::compile::context::Segment;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug)]
pub enum CompileError {
    Unknown,

    And(Box<CompileError>, Box<CompileError>),

    Str(&'static str),
    String(String),

    YamlMissingFunct(String),
    YamlMissingOpcode(String),

    LexExpectedChar(char),

    CompilerAsciiExpectedString { line: usize, got_instead: Token  },
    CompilerAlignExpectedNum    { line: usize, got_instead: Token  },
    CompilerAlignExpectedPos    { line: usize, got_instead: i32    },
    CompilerSpaceExpectedNum    { line: usize, got_instead: Token  },
    CompilerSpaceExpectedPos    { line: usize, got_instead: i32    },
    CompilerUnknownDirective    { line: usize, got_instead: String },
    CompilerIncorrectSegment    { line: usize, current_segment: Segment, needed_segment: Segment },
    

    RegisterNameTooShort(String),
    NumRegisterOutOfRange(i32),
    NamedRegisterOutOfRange { reg_name: char, reg_index: i32 },
    UnknownRegister(String),

    UnknownInstruction(String),
    UnknownInstructionExact { name: String, format: InstFormat },
    UnknownInstructionSAT { name: String, format: Vec<SimpleArgType> },
    MultipleMatchingInstructions(Vec<InstSignature>), // (name, format)
}
