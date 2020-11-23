use std::{collections::HashMap, fmt, str::FromStr};
use crate::{Binary, error::MipsyResult, TEXT_BOT};
use serde::{Serialize, Deserialize};
use super::register::Register;
use crate::yaml::YamlFile;
use mipsy_parser::{
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
    I32,
    U32,
    Off32Rs,
    Off32Rt,
}

#[derive(Clone, Debug)]
pub struct InstMetadata {
    pub desc_short: Option<String>,
    pub desc_long:  Option<String>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Signature {
    Native(InstSignature),
    Pseudo(PseudoSignature),
}

#[derive(Debug, Clone)]
pub enum SignatureRef<'a> {
    Native(&'a InstSignature),
    Pseudo(&'a PseudoSignature),
}

impl InstSet {
    pub fn new(yaml: &YamlFile) -> MipsyResult<Self> {
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
    pub fn compile(&self, program: &Binary, args: Vec<&MPArgument>) -> MipsyResult<u32> {
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
                    ArgumentType::I16 => match arg {
                        MPArgument::Number(num) => match num {
                            MPNumber::Immediate(imm) => match imm {
                                &MPImmediate::I16(imm) => {
                                    imm as u16 as u32
                                }
                                &MPImmediate::U16(imm) => {
                                    imm as u32
                                }
                                MPImmediate::LabelReference(label) => {
                                    // must be relative
                                    let addr = program.get_label(label)?;

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
                    ArgumentType::U16 => match arg {
                        MPArgument::Number(num) => match num {
                            MPNumber::Immediate(imm) => match imm {
                                &MPImmediate::I16(imm) => {
                                    imm as u16 as u32
                                }
                                &MPImmediate::U16(imm) => {
                                    imm as u32
                                }
                                MPImmediate::LabelReference(label) => {
                                    // must be relative
                                    let addr = program.get_label(label)?;

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
                                    program.get_label(label)?
                                }
                                _ => unreachable!(),
                            }
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    },
                    ArgumentType::OffRs | ArgumentType::OffRt => match arg {
                        MPArgument::Register(reg) => match reg {
                            MPRegister::Offset(imm, reg) => match *imm {
                                MPImmediate::I16(imm) => {
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
                ArgumentType::I16    => inst |=  val & 0xFFFF,
                ArgumentType::U16    => inst |=  val & 0xFFFF,
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
        self.matches_args(inst.arguments().iter().map(|(arg, _, _)| arg).collect())
    }

    pub fn matches_args(&self, args: Vec<&MPArgument>) -> bool {
        if self.format.len() != args.len() {
            return false;
        }

        for (i, (my_arg, &their_arg)) in self.format.iter().zip(args.iter()).enumerate() {
            // labels are only relative as the final argument
            let relative_label = (i == args.len() - 1) && self.relative_label;
            if !my_arg.matches(their_arg, relative_label) {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for ArgumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgumentType::Rd      => write!(f, "$Rd"),
            ArgumentType::Rs      => write!(f, "$Rs"),
            ArgumentType::Rt      => write!(f, "$Rt"),
            ArgumentType::Shamt   => write!(f, "shift"),
            ArgumentType::I16     => write!(f, "i16"),
            ArgumentType::U16     => write!(f, "u16"),
            ArgumentType::J       => write!(f, "label"),
            ArgumentType::OffRs   => write!(f, "i16($Rs)"),
            ArgumentType::OffRt   => write!(f, "i16($Rt)"),
            ArgumentType::F32     => write!(f, "f32"),
            ArgumentType::F64     => write!(f, "f64"),
            ArgumentType::I32     => write!(f, "i32"),
            ArgumentType::U32     => write!(f, "u32"),
            ArgumentType::Off32Rs => write!(f, "i32($Rs)"),
            ArgumentType::Off32Rt => write!(f, "i32($Rt)"),
        }
    }
}

impl ArgumentType {
    fn matches(&self, arg: &MPArgument, relative_label: bool) -> bool {
        match arg {
            MPArgument::Register(register) => {
                match register {
                    MPRegister::Normal(_) => matches!(self, Self::Rd | Self::Rs | Self::Rt),
                    MPRegister::Offset(imm, _) => match imm {
                        MPImmediate::I16(_) => matches!(self, Self::OffRs | Self::OffRt | Self::Off32Rs | Self::Off32Rt),

                        MPImmediate::U16(_)
                        | MPImmediate::U32(_)
                        | MPImmediate::I32(_)
                        | MPImmediate::LabelReference(_) => matches!(self, Self::Off32Rs | Self::Off32Rt),
                    }
                }
            }
            MPArgument::Number(number) => {
                match number {
                    MPNumber::Immediate(immediate) => {
                        match immediate {
                            &MPImmediate::I16(num) => {
                                match self {
                                    Self::I16 | Self::I32 | Self::Off32Rs | Self::Off32Rt => true,
                                    Self::U16 | Self::U32 => num >= 0,
                                    Self::Shamt => num >= 0 && num < 32,
                                    _ => false,
                                }
                            }
                            MPImmediate::U16(_) => matches!(self, Self::U16 | Self::I32 | Self::U32 | Self::Off32Rs | Self::Off32Rt),
                            &MPImmediate::I32(num) => {
                                match self {
                                    Self::I32 | Self::J | Self::Off32Rs | Self::Off32Rt => true,
                                    Self::U32 => num >= 0,
                                    _ => false,
                                }
                            }
                            MPImmediate::U32(_) => matches!(self, Self::J | Self::U32 | Self::Off32Rs | Self::Off32Rt),
                            MPImmediate::LabelReference(_) => {
                                match self {
                                    Self::I32 | Self::U32 | Self::J | Self::Off32Rs | Self::Off32Rt => true,
                                    Self::I16 => relative_label,
                                    _ => false,
                                }
                            }
                        }
                    }
                    MPNumber::Char(_) => matches!(self, Self::I16 | Self::I32 | Self::U16 | Self::U32),
                    MPNumber::Float32(_) => matches!(self, Self::F32 | Self::F64),
                    MPNumber::Float64(_) => matches!(self, Self::F64),
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
    I16,
    U16,
    J,
    OffRs,
    OffRt,
    F32,
    F64,
    Off,

    // pseudo
    I32,
    U32,
    Off32,
}

impl PseudoVariable {
    fn name(&self) -> String {
        match self {
            Self::Rd    => "rd",
            Self::Rs    => "rs",
            Self::Rt    => "rt",
            Self::Shamt => "shamt",
            Self::I16   => "i16",
            Self::U16   => "u16",
            Self::J     => "j",
            Self::OffRs => "offrs",
            Self::OffRt => "offrt",
            Self::F32   => "f32",
            Self::F64   => "f64",
            Self::Off   => "off",
        
            // pseudo
            Self::I32   => "i32",
            Self::U32   => "u32",
            Self::Off32   => "off32",
        }.to_string()
    }

    fn from_arg_type(arg_type: ArgumentType) -> Self {
        match arg_type {
            ArgumentType::Rd    => Self::Rd,
            ArgumentType::Rs    => Self::Rs,
            ArgumentType::Rt    => Self::Rt,
            ArgumentType::Shamt => Self::Shamt,
            ArgumentType::I16   => Self::I16,
            ArgumentType::U16   => Self::U16,
            ArgumentType::J     => Self::J,
            ArgumentType::OffRs => Self::OffRs,
            ArgumentType::OffRt => Self::OffRt,
            ArgumentType::F32   => Self::F32,
            ArgumentType::F64   => Self::F64,
        
            // pseudo
            ArgumentType::I32 | ArgumentType::U32 | ArgumentType::Off32Rs | ArgumentType::Off32Rt => panic!("Bad arg type from mips.yaml"),
        }
    }
}

impl PseudoSignature {
    fn lower_upper(&self, program: &Binary, arg: &MPArgument, last: bool) -> MipsyResult<(u16, u16)> {
        let (lower, upper) = match arg {
            MPArgument::Register(reg) => match reg {
                MPRegister::Offset(imm, _) => self.lower_upper(program, &MPArgument::Number(MPNumber::Immediate(imm.clone())), last)?,
                _                          => unreachable!(),
            }
            MPArgument::Number(num) => match num {
                MPNumber::Immediate(imm) => match imm {
                    &MPImmediate::I16(imm) => {
                        (imm as u16, (imm as i32 >> 16) as u16)
                    }
                    &MPImmediate::U16(imm) => {
                        (imm, 0)
                    }
                    &MPImmediate::I32(imm) => {
                        ((imm & 0xFFFF) as u16, (imm >> 16) as u16)
                    }
                    &MPImmediate::U32(imm) => {
                        ((imm & 0xFFFF) as u16, (imm >> 16) as u16)
                    }
                    MPImmediate::LabelReference(ref label) => {
                        let mut addr = program.get_label(label)?;

                        if self.compile.relative_label && last {
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
        };

        Ok((lower, upper))
    }

    fn expand_32_var(var: &PseudoVariable, lower: u16, upper: u16) -> Vec<(String, MPArgument)> {
        vec![
            (format!("{}{}", var.name(), "ihi"), MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(upper as i16)))),
            (format!("{}{}", var.name(), "ilo"), MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(lower as i16)))),
            (format!("{}{}", var.name(), "uhi"), MPArgument::Number(MPNumber::Immediate(MPImmediate::U16(upper)))),
            (format!("{}{}", var.name(), "ulo"), MPArgument::Number(MPNumber::Immediate(MPImmediate::U16(lower)))),
        ]
    }

    fn new_variable(
        &self, 
        program: &Binary, 
        var_type: PseudoVariable, 
        value: MPArgument, 
        variables: &mut HashMap<String, MPArgument>, 
        used: &mut HashMap<String, usize>, last: bool
    ) -> MipsyResult<()> {

        let mappings: Vec<(String, MPArgument)> = match var_type {
            PseudoVariable::I32 | PseudoVariable::U32 | PseudoVariable::Off32 => {
                let (lower, upper) = self.lower_upper(program, &value, last)?;
                
                Self::expand_32_var(&var_type, lower, upper)
            }
            _ => {
                vec![(var_type.name(), value)]
            }
        };

        for (name, value) in mappings {
            if let Some(&amt) = used.get(&name) {
                used.insert(name.clone(), amt + 1);
    
                if amt == 1 {
                    let old = variables.remove(&name).unwrap();
                    variables.insert(format!("{}#{}", name, 1), old);
                    variables.insert(format!("{}#{}", name, 2), value);
                } else {
                    variables.insert(format!("{}#{}", name, amt + 1), value);
                }
            } else {
                used.insert(name.clone(), 1);
                variables.insert(name, value);
            }
    
        }


        Ok(())
    }

    fn get_variables(&self, program: &Binary, args: Vec<&MPArgument>) -> MipsyResult<HashMap<String, MPArgument>> {
        let mut variables: HashMap<String, MPArgument> = HashMap::new();
        let mut used: HashMap<String, usize> = HashMap::new();

        for (i, (arg_type, &arg)) in self.compile.format.iter().zip(&args).enumerate() {
            let last = i == args.len() - 1;

            match arg_type {
                ArgumentType::Rd | ArgumentType::Rs | ArgumentType::Rt | ArgumentType::Shamt | ArgumentType::J => {
                    self.new_variable(program, PseudoVariable::from_arg_type(*arg_type), arg.clone(), &mut variables, &mut used, last)?;
                }
                ArgumentType::I16 => {
                    let arg = match arg {
                        // Relative label
                        MPArgument::Number(MPNumber::Immediate(MPImmediate::LabelReference(label))) => {
                            let addr = program.get_label(label)?;

                            let current_inst_addr = (program.text.len() + self.expand.len() - 1) as u32 * 4 + TEXT_BOT;
                            let imm = ((addr.wrapping_sub(current_inst_addr)) / 4) as i16;

                            MPArgument::Number(MPNumber::Immediate(MPImmediate::I16(imm)))
                        }
                        _ => arg.clone(),
                    };

                    self.new_variable(program, PseudoVariable::I16, arg, &mut variables, &mut used, last)?;
                }
                ArgumentType::U16 => {
                    self.new_variable(program, PseudoVariable::U16, arg.clone(), &mut variables, &mut used, last)?;
                }
                ArgumentType::OffRs => {
                    self.new_variable(program, PseudoVariable::OffRs, arg.clone(), &mut variables, &mut used, last)?;

                    let (imm, reg) = match arg {
                        MPArgument::Register(reg) => match reg {
                            MPRegister::Offset(imm, reg) => (imm, reg),
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    };

                    self.new_variable(program, PseudoVariable::Off, MPArgument::Number(MPNumber::Immediate(imm.clone())), &mut variables, &mut used, last)?;
                    self.new_variable(program, PseudoVariable::Rs,  MPArgument::Register(MPRegister::Normal(reg.clone())), &mut variables, &mut used, last)?;
                }
                ArgumentType::OffRt => {
                    self.new_variable(program, PseudoVariable::OffRt, arg.clone(), &mut variables, &mut used, last)?;

                    let (imm, reg) = match arg {
                        MPArgument::Register(reg) => match reg {
                            MPRegister::Offset(imm, reg) => (imm, reg),
                            _ => unreachable!(),
                        }
                        _ => unreachable!(),
                    };

                    self.new_variable(program, PseudoVariable::Off, MPArgument::Number(MPNumber::Immediate(imm.clone())), &mut variables, &mut used, last)?;
                    self.new_variable(program, PseudoVariable::Rt,  MPArgument::Register(MPRegister::Normal(reg.clone())), &mut variables, &mut used, last)?;
                }
                ArgumentType::F32 => unimplemented!(),
                ArgumentType::F64 => unimplemented!(),
                ArgumentType::I32 => {
                    self.new_variable(program, PseudoVariable::I32, arg.clone(), &mut variables, &mut used, last)?;
                }
                ArgumentType::U32 => {
                    self.new_variable(program, PseudoVariable::U32, arg.clone(), &mut variables, &mut used, last)?;
                }
                ArgumentType::Off32Rs | ArgumentType::Off32Rt => {
                    let reg = match arg {
                        MPArgument::Register(MPRegister::Normal(id)) | MPArgument::Register(MPRegister::Offset(_, id)) => {
                            MPArgument::Register(MPRegister::Normal(id.clone()))
                        }
                        _ => MPArgument::Register(MPRegister::Normal(MPRegisterIdentifier::Numbered(0))),
                    };

                    let reg_type = match arg_type {
                        ArgumentType::Off32Rs => PseudoVariable::Rs,
                        ArgumentType::Off32Rt => PseudoVariable::Rt,
                        _ => unreachable!()
                    };

                    self.new_variable(program, reg_type, reg, &mut variables, &mut used, last)?;
                    self.new_variable(program, PseudoVariable::Off32, arg.clone(), &mut variables, &mut used, last)?;
                }
            }
        }

        Ok(variables)
    }

    fn pre_process(&self, program: &Binary, args: Vec<&MPArgument>) -> MipsyResult<Vec<(String, Vec<MPArgument>)>> {
        let variables = self.get_variables(program, args)?;

        let mut new_instns: Vec<(String, Vec<MPArgument>)> = vec![];

        for expand in self.expand.iter() {
            let (name, data) = (&expand.inst, &expand.data);

            let mut processed_args = vec![];

            for data in data.iter() {
                let mut data = data.clone();

                let mut index = 0;
                while index < data.len() {
                    let find = data[index..]
                        .find('$');

                    if find.is_none() { break }
                    let find = find.unwrap() + 1;

                    let end = {
                        let mut end = find;
                        
                        loop {
                            let char = data.chars().nth(end);
                            match char {
                                Some(char) => {
                                    if !char.is_ascii_alphanumeric() && char != '#' {
                                        break;
                                    }
                                }
                                None => break,
                            }

                            end += 1;
                        }

                        end
                    };

                    if Register::from_str(&data[find..end]).is_err() {
                        let arg = variables.get(&data[find..end].to_ascii_lowercase()).unwrap();
                        data.replace_range((find - 1)..end, &arg.to_string());
                    }

                    index = end;
                }

                let arg = parse_argument(data).unwrap();

                processed_args.push(arg);
            }

            new_instns.push((name.to_string(), processed_args));
        }

        Ok(new_instns)
    }

    pub fn compile(&self, iset: &InstSet, program: &Binary, args: Vec<&MPArgument>) -> MipsyResult<Vec<u32>> {
        let instns = self.pre_process(program, args)?;

        let mut ops = vec![];
        for (ref name, ref args) in instns {
            ops.push(iset.find_native_from_name(name).unwrap()
                         .compile(program, args.iter().collect())?);
        }

        Ok(ops)
    }
}

impl<'a> SignatureRef<'a> {
    pub fn name(&self) -> &str {
        match self {
            Self::Native(sig) => &sig.name,
            Self::Pseudo(sig) => &sig.name,
        }
    }

    pub fn compile_sig(&self) -> &CompileSignature {
        match self {
            Self::Native(sig) => &sig.compile,
            Self::Pseudo(sig) => &sig.compile,
        }
    }

    pub fn compile_ops(&self, binary: &Binary, iset: &InstSet, inst: &MPInstruction) -> MipsyResult<Vec<u32>> {
        Ok(
            match self {
                Self::Native(sig) => vec![sig.compile(binary, inst.arguments().iter().map(|(arg, _, _)| arg).collect())?],
                Self::Pseudo(sig) => sig.compile(iset, binary, inst.arguments().iter().map(|(arg, _, _)| arg).collect())?,
            }
        )
    }

    pub fn cloned(&self) -> Signature {
        match self {
            Self::Native(sig) => Signature::Native((*sig).clone()),
            Self::Pseudo(sig) => Signature::Pseudo((*sig).clone())
        }
    }
}

impl Signature {
    pub fn sigref(&self) -> SignatureRef<'_> {
        match self {
            Self::Native(native) => SignatureRef::Native(&native),
            Self::Pseudo(pseudo) => SignatureRef::Pseudo(&pseudo),
        }
    }
}

pub(crate) trait ToRegister {
    fn to_register(&self) -> MipsyResult<Register>;
}

impl ToRegister for MPRegisterIdentifier {
    fn to_register(&self) -> MipsyResult<Register> {
        Ok(
            match self {
                MPRegisterIdentifier::Named(name) => Register::from_str(name)?,
                MPRegisterIdentifier::Numbered(num) => Register::from_number(*num as i32)?,
            }
        )
    }
}
