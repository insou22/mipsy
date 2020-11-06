use std::collections::HashMap;
use crate::{Binary, error::RSpimResult, TEXT_BOT};
use serde::{Serialize, Deserialize};
use super::register::Register;
use crate::yaml::YamlFile;
use rspim_parser::{
    MPInstruction,
    MPArgument,
    MPRegister,
    MPRegisterIdentifier,
    MPNumber,
    MPImmediate,
    parse_argument,
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
    OffRs,
    OffRt,
    F32,
    F64,

    // pseudo
    Imm32,
    Off32Rs,
    Off32Rt,
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

    pub fn find_native_from_name(&self, inst: &str) -> Option<&InstSignature> {
        let name = inst.to_ascii_lowercase();

        for native_inst in &self.native_set {
            if native_inst.name == name {
                return Some(native_inst);
            }
        }

        None
    }

    pub fn find_native(&self, inst: &MPInstruction) -> Option<&InstSignature> {
        let name = inst.name().to_ascii_lowercase();

        for native_inst in &self.native_set {
            if native_inst.name == name && native_inst.compile.matches(inst) {
                return Some(native_inst);
            }
        }

        None
    }

    pub fn find_pseudo(&self, inst: &MPInstruction) -> Option<&PseudoSignature> {
        let name = inst.name().to_ascii_lowercase();

        for pseudo_inst in &self.pseudo_set {
            if pseudo_inst.name == name && pseudo_inst.compile.matches(inst) {
                return Some(pseudo_inst);
            }
        }

        None
    }
}

impl InstSignature {
    pub fn compile(&self, program: &Binary, args: Vec<&MPArgument>) -> RSpimResult<u32> {
        let mut inst: u32 = 0;

        match self.runtime {
            RuntimeSignature::R { funct } => {
                inst |= (funct as u32) & 0x3F;
            }
            RuntimeSignature::I { opcode, rt } => {
                inst |= (opcode as u32 & 0x3F) << 26;

                if let Some(rt) = rt {
                    inst |= (rt as u32 & 0x1F) << 16;
                }
            }
            RuntimeSignature::J { opcode } => {
                inst |= (opcode as u32 & 0x3F) << 26;
            }
        }

        let mut arg_bits = vec![];

        for (&arg_type, &arg) in self.compile.format.iter().zip(args.iter()) {
            arg_bits.push(
                match arg_type {
                    ArgumentType::Rd | ArgumentType::Rs | ArgumentType::Rt => match arg {
                        MPArgument::Register(MPRegister::Normal(reg)) => {
                            reg.to_register()?.to_u32()
                        }
                        _ => unreachable!(),
                    },
                    ArgumentType::Shamt => match arg {
                        MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(num))) => {
                            (*num as u16 as u32) & 0x1F
                        }
                        _ => unreachable!()
                    },
                    ArgumentType::Imm => match arg {
                        MPArgument::Number(num) => match num {
                            MPNumber::Immediate(imm) => match imm {
                                &MPImmediate::I16(imm) => {
                                    imm as u16 as u32
                                }
                                MPImmediate::LabelReference(label) => {
                                    // must be relative
                                    let addr = program.labels.get(label).unwrap(); // TODO - error label

                                    let current_inst_addr = program.text.len() as u32 * 4 + TEXT_BOT;

                                    ((addr.wrapping_sub(current_inst_addr)) / 4) & 0xFFFF
                                }
                                _ => unreachable!()
                            }
                            &MPNumber::Char(chr) => {
                                chr as u8 as u32
                            }
                            _ => unreachable!()
                        }
                        _ => unreachable!()
                    },
                    ArgumentType::J => match arg {
                        MPArgument::Number(num) => match num {
                            MPNumber::Immediate(imm) => match imm {
                                MPImmediate::LabelReference(label) => {
                                    *program.labels.get(label).unwrap() // TODO - error label
                                }
                                _ => unreachable!(),
                            }
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    },
                    ArgumentType::OffRs | ArgumentType::OffRt => match arg {
                        MPArgument::Register(reg) => match reg {
                            MPRegister::Offset(imm, reg) => match imm {
                                &MPImmediate::I16(imm) => {
                                    let register = reg.to_register()?.to_u32();
                                    let imm = imm as u16 as u32;

                                    (register << 16) | imm
                                }
                                _ => unreachable!(),
                            }
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    },
                    ArgumentType::F32     => unimplemented!(),
                    ArgumentType::F64     => unimplemented!(),
                    _       => unreachable!(),
                }
            );
        }
        
        for (arg, val) in self.compile.format.iter().zip(arg_bits) {
            match arg {
                ArgumentType::Rs     => inst |= (val & 0x1F) << 21,
                ArgumentType::Rt     => inst |= (val & 0x1F) << 16,
                ArgumentType::Rd     => inst |= (val & 0x1F) << 11,
                ArgumentType::Shamt  => inst |= (val & 0x1F) << 6,
                ArgumentType::Imm    => inst |=  val & 0xFFFF,
                ArgumentType::J      => inst |= (val >> 2) & 0x03FFFFFF,
                ArgumentType::OffRs  => inst |= (val & 0x1F0000) << 5 | val & 0xFFFF,
                ArgumentType::OffRt  => inst |=  val & 0x1FFFFF,
                ArgumentType::F32    => unimplemented!(),
                ArgumentType::F64    => unimplemented!(),
                _                    => unreachable!()
            }
        }

        Ok(inst)
    }
}

impl CompileSignature {
    pub fn matches(&self, inst: &MPInstruction) -> bool {
        self.matches_args(inst.arguments())
    }

    pub fn matches_args(&self, args: Vec<&MPArgument>) -> bool {
        if self.format.len() != args.len() {
            return false;
        }

        let mut i = 0;
        for (my_arg, &their_arg) in self.format.iter().zip(args.iter()) {
            // labels are only relative as the final argument
            let relative_label = (i == args.len() - 1) && self.relative_label;
            if !my_arg.matches(their_arg, relative_label) {
                return false;
            }

            i += 1;
        }

        true
    }


}

impl ArgumentType {
    fn matches(&self, arg: &MPArgument, relative_label: bool) -> bool {
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
                                Self::OffRs | Self::OffRt | Self::Off32Rs | Self::Off32Rt => true,
                                _ => false,
                            }
                        },
                        MPImmediate::I32(_) | MPImmediate::LabelReference(_) => {
                            match self {
                                Self::Off32Rs | Self::Off32Rt => true,
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
                            &MPImmediate::I16(num) => {
                                match self {
                                    Self::Imm | Self::Imm32 | Self::Off32Rs | Self::Off32Rt => true,
                                    Self::Shamt => num >= 0 && num < 32,
                                    _ => false,
                                }
                            }
                            MPImmediate::I32(_) => {
                                match self {
                                    Self::Imm32 | Self::J | Self::Off32Rs | Self::Off32Rt => true,
                                    _ => false,
                                }
                            }
                            MPImmediate::LabelReference(_) => {
                                match self {
                                    Self::Imm32 | Self::J | Self::Off32Rs | Self::Off32Rt => true,
                                    Self::Imm => relative_label,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum PseudoVariable {
    Rd,
    Rs,
    Rt,
    Shamt,
    Imm,
    J,
    OffRs,
    OffRt,
    F32,
    F64,
    Off,

    // pseudo
    Imm32Lo,
    Imm32Hi,
    Off32Lo,
    Off32Hi,
}

impl PseudoVariable {
    fn name(&self) -> String {
        match self {
            Self::Rd    => "rd",
            Self::Rs    => "rs",
            Self::Rt    => "rt",
            Self::Shamt => "shamt",
            Self::Imm   => "imm",
            Self::J     => "j",
            Self::OffRs => "offrs",
            Self::OffRt => "offrt",
            Self::F32   => "f32",
            Self::F64   => "f64",
            Self::Off   => "off",
        
            // pseudo
            Self::Imm32Lo   => "imm32lo",
            Self::Imm32Hi   => "imm32hi",
            Self::Off32Lo   => "off32lo",
            Self::Off32Hi   => "off32hi",
        }.to_string()
    }

    fn from_arg_type(arg_type: ArgumentType) -> Self {
        match arg_type {
            ArgumentType::Rd    => Self::Rd,
            ArgumentType::Rs    => Self::Rs,
            ArgumentType::Rt    => Self::Rt,
            ArgumentType::Shamt => Self::Shamt,
            ArgumentType::Imm   => Self::Imm,
            ArgumentType::J     => Self::J,
            ArgumentType::OffRs => Self::OffRs,
            ArgumentType::OffRt => Self::OffRt,
            ArgumentType::F32   => Self::F32,
            ArgumentType::F64   => Self::F64,
        
            // pseudo
            ArgumentType::Imm32 | ArgumentType::Off32Rs | ArgumentType::Off32Rt => panic!("Bad arg type from mips.yaml"),        
        }
    }
}

impl PseudoSignature {
    fn new_variable(var_type: PseudoVariable, value: MPArgument, variables: &mut HashMap<String, MPArgument>, used: &mut HashMap<PseudoVariable, usize>) {
        if let Some(&amt) = used.get(&var_type) {
            used.insert(var_type, amt + 1);

            if amt == 1 {
                let old = variables.remove(&var_type.name()).unwrap();
                variables.insert(format!("{}{}", var_type.name(), 1), old);
                variables.insert(format!("{}{}", var_type.name(), 2), value);
            } else {
                variables.insert(format!("{}{}", var_type.name(), amt + 1), value);
            }
        } else {
            used.insert(var_type, 1);
            variables.insert(var_type.name(), value);
        }
    }

    fn get_variables(&self, program: &Binary, args: Vec<&MPArgument>) -> RSpimResult<HashMap<String, MPArgument>> {
        let mut variables: HashMap<String, MPArgument> = HashMap::new();
        let mut used: HashMap<PseudoVariable, usize> = HashMap::new();

        let mut i = 0;
        for (arg_type, &arg) in self.compile.format.iter().zip(&args) {
            match arg_type {
                ArgumentType::Rd | ArgumentType::Rs | ArgumentType::Rt | ArgumentType::Shamt | ArgumentType::J => {
                    Self::new_variable(PseudoVariable::from_arg_type(*arg_type), arg.clone(), &mut variables, &mut used);
                },
                ArgumentType::Imm => {
                    let arg = match arg {
                        // Relative label
                        MPArgument::Number(MPNumber::Immediate(MPImmediate::LabelReference(label))) => {
                            let addr = *program.labels.get(label).unwrap(); // TODO - error label

                            let current_inst_addr = (program.text.len() + self.expand.len() - 1) as u32 * 4 + TEXT_BOT;
                            let imm = ((addr.wrapping_sub(current_inst_addr)) / 4) as i16;

                            MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(imm)))
                        }
                        _ => arg.clone(),
                    };

                    Self::new_variable(PseudoVariable::Imm, arg, &mut variables, &mut used);
                }
                ArgumentType::OffRs => {
                    Self::new_variable(PseudoVariable::OffRs, arg.clone(), &mut variables, &mut used);

                    let (imm, reg) = match arg {
                        MPArgument::Register(reg) => match reg {
                            MPRegister::Offset(imm, reg) => (imm, reg),
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    };

                    Self::new_variable(PseudoVariable::Off, MPArgument::Number(MPNumber::Immediate(imm.clone())), &mut variables, &mut used);
                    Self::new_variable(PseudoVariable::Rs,  MPArgument::Register(MPRegister::Normal(reg.clone())), &mut variables, &mut used);
                }
                ArgumentType::OffRt => {
                    Self::new_variable(PseudoVariable::OffRt, arg.clone(), &mut variables, &mut used);

                    let (imm, reg) = match arg {
                        MPArgument::Register(reg) => match reg {
                            MPRegister::Offset(imm, reg) => (imm, reg),
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    };

                    Self::new_variable(PseudoVariable::Off, MPArgument::Number(MPNumber::Immediate(imm.clone())), &mut variables, &mut used);
                    Self::new_variable(PseudoVariable::Rt,  MPArgument::Register(MPRegister::Normal(reg.clone())), &mut variables, &mut used);
                }
                ArgumentType::F32 => unimplemented!(),
                ArgumentType::F64 => unimplemented!(),
                ArgumentType::Imm32 => {
                    let (lower, upper) = match arg {
                        MPArgument::Number(num) => match num {
                            MPNumber::Immediate(imm) => match imm {
                                &MPImmediate::I16(imm) => {
                                    (imm as u16, 0 as u16)
                                }
                                &MPImmediate::I32(imm) => {
                                    ((imm & 0xFFFF) as u16, (imm >> 16) as u16)
                                }
                                MPImmediate::LabelReference(ref label) => {
                                    let mut addr = *program.labels.get(label).unwrap(); // TODO - error label

                                    if self.compile.relative_label && (i == args.len() - 1) {
                                        let current_inst_addr = (program.text.len() + self.expand.len() - 1) as u32 * 4 + TEXT_BOT;
                                        addr = (addr.wrapping_sub(current_inst_addr)) / 4;
                                    }

                                    ((addr & 0xFFFF) as u16, (addr >> 16) as u16)
                                }
                            }
                            &MPNumber::Char(chr) => {
                                (chr as u16, 0 as u16)
                            }
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    };

                    Self::new_variable(PseudoVariable::Imm32Lo, MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(lower as i16))), &mut variables, &mut used);
                    Self::new_variable(PseudoVariable::Imm32Hi, MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(upper as i16))), &mut variables, &mut used);
                }
                ArgumentType::Off32Rs | ArgumentType::Off32Rt => {
                    let (off32lower, off32upper, reg) = match arg {
                        MPArgument::Register(MPRegister::Offset(imm, reg)) => {
                            let (off32lower, off32upper) = match imm {
                                &MPImmediate::I16(imm) => (imm, 0),
                                &MPImmediate::I32(imm) => (imm as i16, (imm >> 16) as i16),
                                MPImmediate::LabelReference(label) => {
                                    let addr = *program.labels.get(label).unwrap(); // TODO - Error label doesnt exist

                                    (addr as i16, (addr >> 16) as i16)
                                },
                            };

                            let reg = MPArgument::Register(MPRegister::Normal(reg.clone()));

                            (off32lower, off32upper, reg)
                        }
                        MPArgument::Number(MPNumber::Immediate(imm)) => match imm {
                            &MPImmediate::I16(imm) => {
                                (imm, 0 as i16, MPArgument::Register(MPRegister::Normal(MPRegisterIdentifier::Numbered(0))))
                            }
                            &MPImmediate::I32(imm) => {
                                ((imm & 0xFFFF) as i16, (imm >> 16) as i16, MPArgument::Register(MPRegister::Normal(MPRegisterIdentifier::Numbered(0))))
                            }
                            MPImmediate::LabelReference(label) => {
                                let mut addr = *program.labels.get(label).unwrap(); // TODO - error label

                                if self.compile.relative_label && (i == args.len() - 1) {
                                    let current_inst_addr = (program.text.len() + self.expand.len() - 1) as u32 * 4 + TEXT_BOT;
                                    addr = (addr.wrapping_sub(current_inst_addr)) / 4;
                                }

                                ((addr & 0xFFFF) as i16, (addr >> 16) as i16, MPArgument::Register(MPRegister::Normal(MPRegisterIdentifier::Numbered(0))))

                            }
                        }
                        _ => unreachable!(),
                    };

                    let reg_type = match arg_type {
                        ArgumentType::Off32Rs => PseudoVariable::Rs,
                        ArgumentType::Off32Rt => PseudoVariable::Rt,
                        _ => unreachable!()
                    };

                    Self::new_variable(reg_type, reg, &mut variables, &mut used);

                    Self::new_variable(PseudoVariable::Off32Lo, MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(off32lower))), &mut variables, &mut used);
                    Self::new_variable(PseudoVariable::Off32Hi, MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(off32upper))), &mut variables, &mut used);
                }
            }

            i += 1;
        }

        Ok(variables)
    }

    fn pre_process(&self, program: &Binary, args: Vec<&MPArgument>) -> RSpimResult<Vec<(String, Vec<MPArgument>)>> {
        let variables = self.get_variables(program, args)?;

        let mut new_instns: Vec<(String, Vec<MPArgument>)> = vec![];

        for expand in self.expand.iter() {
            let (name, data) = (&expand.inst, &expand.data);

            let mut processed_args = vec![];

            for data in data.iter() {
                let data = &data.to_ascii_lowercase();

                if data.starts_with("$") && Register::from_str(&data[1..]).is_err() {
                    // assume pseudoinstructions are sane
                    let arg = variables.get(&data[1..]).unwrap();

                    processed_args.push(arg.clone());
                } else {
                    let (leftover, arg) = parse_argument(data.as_bytes()).unwrap();

                    if !leftover.is_empty() {
                        // TODO: warning
                        println!("Dodgy parse? leftover={}", String::from_utf8_lossy(leftover).to_string());
                    }

                    processed_args.push(arg);
                }
            }

            new_instns.push((name.to_string(), processed_args));
        }

        Ok(new_instns)
    }

    pub fn compile(&self, iset: &InstSet, program: &Binary, args: Vec<&MPArgument>) -> RSpimResult<Vec<u32>> {
        let instns = self.pre_process(program, args)?;

        let mut ops = vec![];
        for (ref name, ref args) in instns {
            ops.push(iset.find_native_from_name(name).unwrap()
                         .compile(program, args.iter().collect())?);
        }

        Ok(ops)
    }
}

trait ToRegister {
    fn to_register(&self) -> RSpimResult<Register>;
}

impl ToRegister for MPRegisterIdentifier {
    fn to_register(&self) -> RSpimResult<Register> {
        Ok(
            match self {
                MPRegisterIdentifier::Named(name) => Register::from_str(name)?,
                MPRegisterIdentifier::Numbered(num) => Register::from_number(*num as i32)?,
            }
        )
    }
}
