use crate::error::RSpimResult;
use serde::{Serialize, Deserialize};
use super::register::Register;
use crate::yaml::parse::YamlFile;
use rspim_parser::{
    MPInstruction,
    MPArgument,
    MPRegister,
    MPRegisterIdentifier,
    MPNumber,
    MPImmediate,
};

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
    pub format: Vec<ArgumentType>,
    pub relative_label: bool,
}

#[derive(Clone, Debug)]
pub enum RuntimeSignature {
    R { funct:  u8 },
    I { opcode: u8, rt: Option<u8> },
    J { opcode: u8 },
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArgumentType {
    Rd,
    Rs,
    Rt,
    Shamt,
    Imm,
    J,
    AddrRs,
    AddrRt,
    F32,
    F64,

    // pseudo
    Imm32,
    Addr32Rs,
    Addr32Rt,
}

#[derive(Clone, Debug)]
pub struct InstMetadata {
    pub desc_short: Option<String>,
    pub desc_long:  Option<String>,
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
    pub expand: Vec<PseudoExpand>,
}

#[derive(Debug, Clone)]
pub struct PseudoExpand {
    pub inst: String,
    pub data: Vec<String>,
}

impl InstSet {
    pub fn new(yaml: &YamlFile) -> RSpimResult<Self> {
        super::yaml::from_yaml(yaml)
    }

    pub fn find_native(&self, inst: &MPInstruction) -> Option<&InstSignature> {
        let name = inst.name().to_ascii_lowercase();

        let matches = vec![];
        
        for native_inst in &self.native_set {
            if native_inst.name == name && native_inst.compile.matches(inst) {
                return Some(native_inst);
            }
        }

        None
    }

    pub fn find_pseudo(&self, inst: &MPInstruction) -> Option<&PseudoSignature> {
        let name = inst.name().to_ascii_lowercase();

        let matches = vec![];
        
        for pseudo_inst in &self.pseudo_set {
            if pseudo_inst.name == name && pseudo_inst.compile.matches(inst) {
                return Some(pseudo_inst);
            }
        }

        None
    }
}

impl CompileSignature {
    pub fn matches(&self, inst: &MPInstruction) -> bool {
        if self.format.len() != inst.arguments().len() {
            return false;
        }

        if !self.matches_args(inst.arguments()) {
            return false;
        }

        true
    }

    pub fn matches_args(&self, args: Vec<&MPArgument>) -> bool {
        for (my_arg, &their_arg) in self.format.iter().zip(args.iter()) {
            if !my_arg.matches(their_arg) {
                return false;
            }
        }

        true
    }
}

impl ArgumentType {
    fn matches(&self, arg: &MPArgument) -> bool {
        match arg {
            MPArgument::Register(register) => {
                match register {
                    MPRegister::Normal(_) => {
                        match self {
                            Self::Rd | Self::Rs | Self::Rt => true,
                            _ => false,
                        }
                    },
                    MPRegister::Offset(imm, _) => match imm {
                        MPImmediate::I16(_) => {
                            match self {
                                Self::AddrRs | Self::AddrRt | Self::Addr32Rs | Self::Addr32Rt => true,
                                _ => false,
                            }
                        },
                        MPImmediate::I32(_) | MPImmediate::LabelReference(_) => {
                            match self {
                                Self::Addr32Rs | Self::Addr32Rt => true,
                                _ => false,
                            }
                        },
                    }
                }
            }
            MPArgument::Number(number) => {
                match number {
                    MPNumber::Immediate(immediate) => {
                        match immediate {
                            MPImmediate::I16(num) => {
                                match self {
                                    Self::Imm | Self::Imm32 => true,
                                    Self::Shamt => *num >= 0 && *num < 32,
                                    _ => false,
                                }
                            }
                            MPImmediate::I32(_) | MPImmediate::LabelReference(_) => {
                                match self {
                                    Self::Imm32 | Self::J => true,
                                    _ => false,
                                }
                            }
                        }
                    }
                    MPNumber::Char(_) => {
                        match self {
                            Self::Imm | Self::Imm32 => true,
                            _ => false,
                        }
                    }
                    MPNumber::Float32(_) => {
                        match self {
                            Self::F32 | Self::F64 => true,
                            _ => false,
                        }
                    }
                    MPNumber::Float64(_) => {
                        match self {
                            Self::F64 => true,
                            _ => false,
                        }
                    }
                }
            }
        }
    }
}

trait ToRegister {
    fn to_register(&self) -> RSpimResult<Register>;
}

impl ToRegister for MPRegisterIdentifier {
    fn to_register(&self) -> RSpimResult<Register> {
        Ok(
            match *self {
                MPRegisterIdentifier::Named(name) => Register::from_str(&name)?,
                MPRegisterIdentifier::Numbered(num) => Register::from_number(num as i32)?,
            }
        )
    }
}



    // pub fn gen_op(&self, args: &[u32]) -> RSpimResult<u32> {
    //     let format = self.compile.format.arg_formats();

    //     if args.len() != format.len() {
    //         println!("{} {:?}", self.name, args);
    //         cerr!(CompileError::Str("SHOULDN'T HAPPEN - InstSignature::gen_op"))
    //     } else {
    //         let mut op: u32 = 0;

    //         match self.runtime {
    //             RuntimeSignature::R { funct } => {
    //                 op |= funct as u32;
    //             }
    //             RuntimeSignature::I { opcode, rt } => {
    //                 op |=  (opcode as u32) << 26;

    //                 if let Some(rt) = rt {
    //                     op |= (rt as u32) << 16;
    //                 }
    //             }
    //             RuntimeSignature::J { opcode } => {
    //                 op |=  (opcode as u32) << 26;
    //             }
    //         }
            
    //         for (&arg, val) in format.iter().zip(args) {
    //             match arg {
    //                 ArgType::Rs => op |= (val & 0x0000001F) << 21,
    //                 ArgType::Rt => op |= (val & 0x0000001F) << 16,
    //                 ArgType::Rd => op |= (val & 0x0000001F) << 11,
    //                 ArgType::Sa => op |= (val & 0x0000001F) << 6,
    //                 ArgType::Im => op |= (val & 0x0000FFFF) << 0,
    //                 ArgType::J  => op |=  val >> 2 & 0x03FFFFFF,
    //                 _           => unreachable!(),
    //             }
    //         }

    //         Ok(op)
    //     }
    // }
