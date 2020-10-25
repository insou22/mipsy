use super::instruction::{ArgType, PseudoSignature};
use super::instruction::PseudoExpansion;
use super::instruction::InstSet;
use crate::error::RSpimResult;
use crate::cerr;
use crate::error::CompileError;
use crate::inst::register::Register;
use crate::compile::context::Context;
use std::collections::HashMap;

pub trait PseudoInst : PseudoInstClone {
    fn expand(&self, set: &InstSet, input: &[u32]) -> RSpimResult<Vec<u32>>;
    fn len(&self, context: &Context) -> usize;
}

pub fn get_complex_pseudo(name: &str) -> RSpimResult<Box<dyn PseudoInst>> {
    match name.to_ascii_lowercase() {
        _ => cerr!(CompileError::Unknown),
    }
}

impl PseudoInst for PseudoSignature {
    fn expand(&self, set: &InstSet, input: &[u32]) -> RSpimResult<Vec<u32>> {
        let mut insts = vec![];

        match &self.expand {
            PseudoExpansion::Simple(expands) => {
                let mut bindings: HashMap<&str, u32> = HashMap::new();

                for (&ty, &val) in self.compile.format.arg_formats().iter().zip(input) {
                    if ty == ArgType::Wd {
                        // words cannot be used in their full 32-bit form
                        // instead offer $Wdu and $Wdl for upper and lower 16 bits
                        bindings.insert("wdu", val >> 16);
                        bindings.insert("wdl", val & 0xFFFF);
                    } else {
                        bindings.insert(ty.to_string(), val);
                    }
                }

                for expand in expands {
                    let inst = set.find_instruction(&expand.inst, None)?;

                    let mut final_input: Vec<u32> = vec![];

                    for data in &expand.data {
                        if data.starts_with("$") {
                            let var = &data.to_lowercase()[1..];

                            if let Some(&binding) = bindings.get(var) {
                                final_input.push(binding);
                                continue;
                            }

                            match Register::from_str(var) {
                                Ok(reg) => {
                                    final_input.push(reg.to_number() as u32);
                                    continue;
                                }
                                Err(_) => {
                                    return cerr!(CompileError::PseudoUnknownVariable(data.to_string()))
                                }
                            }
                        }

                        match data.parse::<i32>() {
                            Ok(num) => {
                                final_input.push(num as u32);
                                continue;
                            }
                            Err(_) => {}
                        }

                        match data.parse::<u32>() {
                            Ok(num) => {
                                final_input.push(num);
                                continue;
                            }
                            Err(_) => return cerr!(CompileError::PseudoUnknownVariable(data.to_string()))
                        }
                    }

                    insts.push(inst.gen_op(&final_input)?);
                }
            }
            PseudoExpansion::Complex(complex) => {
                insts = complex.expand(set, input)?;
            }
        }

        Ok(insts)
    }

    fn len(&self, context: &Context) -> usize {
        match &self.expand {
            PseudoExpansion::Simple(expands) => expands.len(),
            PseudoExpansion::Complex(complex) => complex.len(context),
        }
    }
}


// sometimes the rust compiler is tons of funs! //
pub trait PseudoInstClone {
    fn clone_box(&self) -> Box<dyn PseudoInst>;
}

impl<T> PseudoInstClone for T
where
    T: 'static + PseudoInst + Clone,
{
    fn clone_box(&self) -> Box<dyn PseudoInst> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn PseudoInst> {
    fn clone(&self) -> Box<dyn PseudoInst> {
        self.clone_box()
    }
}
// sometimes the rust compiler is tons of funs! //