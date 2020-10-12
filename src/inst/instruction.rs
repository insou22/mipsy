use crate::error::RSpimResult;
use crate::error::compile_error::CompileError;
use crate::cerr;
use serde::{Serialize, Deserialize};
use super::pseudo::PseudoInst;
use crate::yaml::parse::YamlFile;
use crate::yaml::parse::InstructionType;
use crate::compile::context::Token;

#[derive(Debug)]
pub struct InstSet {
    pub native_set: Vec<InstSignature>,
    pub pseudo_set: Vec<PseudoSignature>,
}

#[derive(Clone, Debug)]
pub struct InstSignature {
    pub name: String,
    pub compile: CompileSignature,
    pub runtime: RuntimeSignature,
    pub meta: InstMetadata,
}

#[derive(Clone, Debug)]
pub struct CompileSignature {
    pub format: InstFormat,
    pub relative_label: bool,
}

#[derive(Clone, Debug)]
pub enum RuntimeSignature {
    R { funct:  u8 },
    I { opcode: u8, rt: Option<u8> },
    J { opcode: u8 },
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InstFormat {
    R0,
    Rd,
    Rs,
    RdRs,
    RsRt,
    RdRsRt,
    RdRtRs,
    RdRtSa,
    J,
    Im,
    RsIm,
    RtIm,
    RsRtIm,
    RtRsIm,
    RtImRs,

    // Pseudo-Only
    RsIm1Im2,
}

#[derive(Copy, Clone, Debug)]
pub enum ArgType {
    Rd,
    Rs,
    Rt,
    Sa,
    Im,
    J,

    // Pseudo-only
    Im1,
    Im2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimpleArgType {
    Register,
    Immediate,
}

#[derive(Clone, Debug)]
pub struct InstMetadata {
    pub desc_short: Option<String>,
    pub desc_long:  Option<String>,
}

#[derive(Clone)]
pub enum PseudoExpansion {
    Simple(Vec<PseudoExpand>),
    Complex(Box<dyn PseudoInst>),
}

#[derive(Debug)]
pub enum GenericSignature {
    Native(InstSignature),
    Pseudo(PseudoSignature),
}

#[derive(Debug, Clone)]
pub struct PseudoSignature {
    pub name: String,
    pub compile: CompileSignature,
    pub expand: PseudoExpansion,
}

#[derive(Debug, Clone)]
pub struct PseudoExpand {
    pub inst: String,
    pub data: Vec<String>,
}

impl InstSet {
    pub fn new(yaml: &YamlFile) -> RSpimResult<Self> {
        let mut native_set = vec![];

        for inst in &yaml.instructions {
            let native_inst = InstSignature {
                name: inst.name.to_ascii_lowercase(),
                compile: CompileSignature {
                    format: inst.compile.format,
                    relative_label: inst.compile.relative_label,
                },
                runtime: match inst.runtime.inst_type {
                    InstructionType::R => {
                        if let Some(funct) = inst.runtime.funct {
                            RuntimeSignature::R { funct }
                        } else {
                            return cerr!(CompileError::YamlMissingFunct(inst.name.to_ascii_lowercase()));
                        }
                    }
                    InstructionType::I => {
                        if let Some(opcode) = inst.runtime.opcode {
                            RuntimeSignature::I { opcode, rt: inst.runtime.rt }
                        } else {
                            return cerr!(CompileError::YamlMissingOpcode(inst.name.to_ascii_lowercase()));
                        }
                    }
                    InstructionType::J => {
                        if let Some(opcode) = inst.runtime.opcode {
                            RuntimeSignature::J { opcode }
                        } else {
                            return cerr!(CompileError::YamlMissingOpcode(inst.name.to_ascii_lowercase()));
                        }
                    }
                },
                meta: InstMetadata {
                    desc_short: inst.desc_short.clone(),
                    desc_long: inst.desc_long.clone(),
                },
            };

            native_set.push(native_inst);
        }

        let mut pseudo_set = vec![];

        for inst in &yaml.pseudoinstructions {
            let pseudo_inst = PseudoSignature {
                name: inst.name.to_ascii_lowercase(),
                compile: CompileSignature {
                    format: inst.compile.format,
                    relative_label: inst.compile.relative_label,
                },
                expand: match &inst.expand {
                    Some(v) => PseudoExpansion::Simple(v.iter().map(|expand|
                        PseudoExpand {
                            inst: expand.inst.clone(),
                            data: expand.data.clone(),
                        }
                    ).collect()),
                    None => PseudoExpansion::Complex(super::pseudo::get_complex_pseudo(&inst.name.to_ascii_lowercase())?),
                }
            };

            pseudo_set.push(pseudo_inst);
        }

        Ok(
            InstSet {
                native_set,
                pseudo_set,
            }
        )
    }

    pub fn find_instruction<'a>(&'a self, name: &str, format: Option<Vec<SimpleArgType>>) -> RSpimResult<&'a InstSignature> {
        let mut matches = vec![];

        let name = name.to_lowercase();

        for inst in &self.native_set {
            if inst.name == name {
                if let Some(ref format) = format {
                    if inst.compile.format.matches(format) {
                        matches.push(inst)
                    }
                } else {
                    matches.push(inst)
                }
            }
        }

        if matches.is_empty() {
            return match format {
                Some(format) => cerr!(CompileError::UnknownInstructionSAT{ name: name.into(), format }),
                None => cerr!(CompileError::UnknownInstruction(name.into())),
            }
        }

        if matches.len() > 1 {
            return cerr!(CompileError::MultipleMatchingInstructions(
                matches.iter().map(|&i| GenericSignature::Native(i.clone())).collect())
            );
        }

        Ok(matches[0])
    }

    pub fn find_instruction_exact<'a>(&'a self, name: &str, format: InstFormat) -> RSpimResult<&'a InstSignature> {
        for inst in &self.native_set {
            if inst.name == name && inst.compile.format == format {
                return Ok(inst);
            }
        }

        cerr!(CompileError::UnknownInstructionExact { name: name.into(), format })
    }
}

impl InstFormat {
    pub fn tokens_match(&self, tokens: &mut Vec<&Token>) -> bool {
        let types: Vec<SimpleArgType> = 
            self.arg_formats().iter()
                .map(ArgType::simple)
                .collect();
        
        if types.len() != tokens.len() {
            if *self == InstFormat::RtImRs && types.len() == 3 && tokens.len() == 2 {
                if matches!(tokens[0], Token::Register(_)) && matches!(tokens[1], Token::Number(_) | Token::LabelReference(_)) {
                    // TODO - disgusting hack
                    tokens.push(Box::leak(Box::new(Token::OffsetRegister("zero".to_string()))));
                } else { 
                    return false; 
                }
            } else { 
                return false; 
            }
        }

        for (token, &simple_type) in tokens.iter().zip(types.iter()) {
            let token_type = match token {
                Token::Register(_) | Token::OffsetRegister(_) => SimpleArgType::Register,

                Token::Number(_) | Token::Float(_) | 
                  Token::LabelReference(_) | Token::ConstChar(_) => SimpleArgType::Immediate,

                other => panic!("tokens_match: {:?} - This should never happen", other),
            };
            
            if token_type != simple_type {
                return false;
            }
        }

        true
    }

    pub fn arg_formats(&self) -> Vec<ArgType> {
        match *self {
            InstFormat::R0     => vec![],
            InstFormat::Rd     => vec![ArgType::Rd],
            InstFormat::Rs     => vec![ArgType::Rs],
            InstFormat::RdRs   => vec![ArgType::Rd, ArgType::Rs],
            InstFormat::RsRt   => vec![ArgType::Rs, ArgType::Rt],
            InstFormat::RdRsRt => vec![ArgType::Rd, ArgType::Rs, ArgType::Rt],
            InstFormat::RdRtRs => vec![ArgType::Rd, ArgType::Rt, ArgType::Rs],
            InstFormat::RdRtSa => vec![ArgType::Rd, ArgType::Rt, ArgType::Sa],
            InstFormat::J      => vec![ArgType::J],
            InstFormat::Im     => vec![ArgType::Im],
            InstFormat::RsIm   => vec![ArgType::Rs, ArgType::Im],
            InstFormat::RtIm   => vec![ArgType::Rt, ArgType::Im],
            InstFormat::RsRtIm => vec![ArgType::Rs, ArgType::Rt, ArgType::Im],
            InstFormat::RtRsIm => vec![ArgType::Rt, ArgType::Rs, ArgType::Im],
            InstFormat::RtImRs => vec![ArgType::Rt, ArgType::Im, ArgType::Rs],

            // Pseudo-only
            InstFormat::RsIm1Im2 => vec![ArgType::Rs, ArgType::Im1, ArgType::Im2],
        }
    }

    pub fn matches(&self, other: &Vec<SimpleArgType>) -> bool {
        let format = self.arg_formats();

        if format.len() != other.len() {
            return false;
        }

        for (arg_type, simple_arg_type) in format.iter().zip(other.iter()) {
            match simple_arg_type {
                SimpleArgType::Register => {
                    if arg_type.is_immediate() {
                        return false;
                    }
                }
                SimpleArgType::Immediate => { 
                    if arg_type.is_register() {
                        return false;
                    }
                }
            }
        }

        true
    }
}

impl ArgType {
    pub fn is_register(&self) -> bool {
        match *self {
            Self::Rd => true,
            Self::Rs => true,
            Self::Rt => true,
            Self::Sa => false,
            Self::Im => false,
            Self::J  => false,

            // Pseudo-only
            Self::Im1 => false,
            Self::Im2 => false,
        }
    }

    pub fn is_immediate(&self) -> bool {
        match *self {
            Self::Rd => false,
            Self::Rs => false,
            Self::Rt => false,
            Self::Sa => true,
            Self::Im => true,
            Self::J  => true,

            // Pseudo-only
            Self::Im1 => true,
            Self::Im2 => true,
        }
    }

    pub fn simple(&self) -> SimpleArgType {
        if self.is_register() {
            SimpleArgType::Register
        } else {
            SimpleArgType::Immediate
        }
    }

    pub fn to_string(&self) -> &'static str {
        match *self {
            Self::Rd => "rd",
            Self::Rs => "rs",
            Self::Rt => "rt",
            Self::Sa => "sa",
            Self::Im => "im",
            Self::J  => "j",

            // Pseudo-only
            Self::Im1 => "im1",
            Self::Im2 => "im2",
        }
    }
}

impl InstSignature {
    pub fn gen_op(&self, args: &[u32]) -> RSpimResult<u32> {
        let format = self.compile.format.arg_formats();

        if args.len() != format.len() {
            println!("{} {:?}", self.name, args);
            cerr!(CompileError::Str("SHOULDN'T HAPPEN - InstSignature::gen_op"))
        } else {
            let mut op: u32 = 0;

            match self.runtime {
                RuntimeSignature::R { funct } => {
                    op |= funct as u32;
                }
                RuntimeSignature::I { opcode, rt } => {
                    op |=  (opcode as u32) << 26;

                    if let Some(rt) = rt {
                        op |= (rt as u32) << 16;
                    }
                }
                RuntimeSignature::J { opcode } => {
                    op |=  (opcode as u32) << 26;
                }
            }
            
            for (&arg, val) in format.iter().zip(args) {
                match arg {
                    ArgType::Rs => op |= (val & 0x0000001F) << 21,
                    ArgType::Rt => op |= (val & 0x0000001F) << 16,
                    ArgType::Rd => op |= (val & 0x0000001F) << 11,
                    ArgType::Sa => op |= (val & 0x0000001F) << 6,
                    ArgType::Im => op |= (val & 0x0000FFFF) << 0,
                    ArgType::J  => op |= (val & 0x03FFFFFF) << 0,
                    _           => unreachable!(),
                }
            }

            Ok(op)
        }
    }
}

impl std::fmt::Debug for PseudoExpansion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Simple(v) => {
                write!(formatter, "PseudoExpansion::Simple(")?;
                v.fmt(formatter)?;
                write!(formatter, ")")
            }
            Self::Complex(_) => write!(formatter, "PseudoExpansion::Complex(fn expand(&self, set: &InstSet, input: Vec<u32>) -> RSpimResult<Vec<u32>>)"),
        }
    }
}