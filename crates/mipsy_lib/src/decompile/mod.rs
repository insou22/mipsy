use std::{collections::HashMap, rc::Rc};

use crate::inst::RuntimeMetadata;
use crate::{Binary, Safe};
use crate::inst::instruction::{InstSet, CompileSignature, ArgumentType, RuntimeSignature};
use crate::inst::register::Register;

pub struct Decompiled<'a> {
    pub opcode: u32,
    pub addr: u32,
    pub inst_sig: Option<&'a CompileSignature>,
    pub runtime_meta: Option<&'a RuntimeMetadata>,
    pub inst_name: Option<String>,
    pub arguments: Vec<String>,
    pub labels: Vec<String>,
    pub location: Option<(Rc<str>, u32)>,
}

#[derive(Debug)]
pub struct Uninit {
    pub addr: u32,
    pub labels: Vec<String>,
    pub location: Option<(Rc<str>, u32)>,
}

pub fn decompile(program: &Binary, iset: &InstSet) -> String {
    let mut text = String::new();
    let unknown_instruction = String::from("# Unknown instruction");

    let decompiled = decompile_into_parts(program, iset);

    let mut keys: Vec<u32> = decompiled.keys().copied().collect();
    keys.sort_unstable();

    for (addr, parts) in keys.into_iter().map(|addr| (addr, decompiled.get(&addr).unwrap())) {
        if let Err(parts) = parts {
            if !parts.labels.is_empty() {
                text.push('\n');
            }

            for label in parts.labels.iter() {
                text.push_str(&format!("{}: \n", label));
            }

            text.push_str(&format!("0x{:08x}: [uninitialised]\n", addr));
            continue;
        }

        let parts = parts.as_ref().expect("just checked Err case");

        if !parts.labels.is_empty() {
            text.push('\n');
        }

        for label in parts.labels.iter() {
            text.push_str(&format!("{}: \n", label));
        }

        text.push_str(
            &format!(
                "0x{:08x} [0x{:08x}]    {:6} {}\n",
                addr,
                parts.opcode,
                parts.inst_name.as_ref().unwrap_or(&unknown_instruction),
                parts.arguments.join(", ")
            )
        );
    }

    text
}

pub fn decompile_into_parts<'a>(program: &Binary, iset: &'a InstSet) -> HashMap<u32, Result<Decompiled<'a>, Uninit>> {
    let mut decompiled = HashMap::new();
    
    let mut text_addr = crate::TEXT_BOT;

    for word in program.text_words() {
        if let Safe::Valid(word) = word {
            let parts = decompile_inst_into_parts(program, iset, word, text_addr);

            decompiled.insert(text_addr, Ok(parts));
        } else {
            let mut labels = vec![];

            for (label, &addr) in program.labels.iter() {
                if addr == text_addr {
                    labels.push(label.to_string());
                }
            }

            decompiled.insert(text_addr, Err(Uninit {
                addr: text_addr,
                labels,
                location: program.line_numbers.get(&text_addr).cloned(),
            }));
        }

        text_addr += 4;
    }

    decompiled
}

pub fn decompile_inst_into_parts<'a>(program: &Binary, iset: &'a InstSet, inst: u32, text_addr: u32) -> Decompiled<'a> {
    let mut parts = Decompiled {
        opcode: inst,
        addr: text_addr,
        inst_sig: None,
        runtime_meta: None,
        inst_name: None,
        arguments: vec![],
        labels: vec![],
        location: program.line_numbers.get(&text_addr).cloned(),
    };

    for (label, &addr) in program.labels.iter() {
        if addr == text_addr {
            parts.labels.push(label.to_string());
        }
    }

    let opcode = inst >> 26;
    let rs =    (inst >> 21) & 0x1F;
    let rt =    (inst >> 16) & 0x1F;
    let rd =    (inst >> 11) & 0x1F;
    let shamt = (inst >> 6) & 0x1F;
    let funct =  inst & 0x3F;
    let imm =   (inst & 0xFFFF) as i16;
    let addr =   inst & 0x3FFFFFF;
    
    let mut inst = None;

    for native_inst in iset.native_set() {
        match native_inst.runtime_signature() {
            &RuntimeSignature::R { opcode: inst_opcode, funct: inst_funct } => {
                if inst_opcode as u32 != opcode || inst_funct as u32 != funct {
                    continue;
                }
            }

            &RuntimeSignature::I { opcode: inst_opcode, rt: inst_rt } => {
                if inst_opcode as u32 != opcode || inst_rt.is_some() && inst_rt.unwrap() as u32 != rt {
                    continue;
                }
            }

            &RuntimeSignature::J { opcode: inst_opcode, .. } => {
                if inst_opcode as u32 != opcode {
                    continue;
                }
            }
        }

        inst = Some(native_inst);
        parts.inst_sig = Some(native_inst.compile_signature());
        parts.runtime_meta = Some(native_inst.runtime_metadata());
        break;
    }

    if let Some(inst) = inst {
        if inst.name() == "sll" && rd == 0 && rt == 0 && shamt == 0 {
            parts.inst_name = Some("nop".to_string());
        } else {
            parts.inst_name = Some(inst.name().to_string());

            parts.arguments = inst.compile_signature().format().iter()
                .map(|arg| match arg {
                    ArgumentType::Rd     => format!("${}", Register::u32_to_str(rd)),
                    ArgumentType::Rt     => format!("${}", Register::u32_to_str(rt)),
                    ArgumentType::Rs     => format!("${}", Register::u32_to_str(rs)),
                    ArgumentType::Shamt  => format!("{}", shamt),
                    ArgumentType::OffRs  => format!("{}(${})", if imm != 0 { imm.to_string() } else { String::new() }, Register::u32_to_str(rs)),
                    ArgumentType::OffRt  => format!("{}(${})", if imm != 0 { imm.to_string() } else { String::new() }, Register::u32_to_str(rt)),
                    ArgumentType::I16    => {
                        let mut res = None;

                        if inst.compile_signature().relative_label() {

                            for (label, &addr) in program.labels.iter() {
                                if addr == text_addr.wrapping_add((imm as i32 * 4) as u32) {
                                    res = Some(label);
                                    break;
                                }
                            }

                        }
                        
                        if let Some(label) = res {
                            label.to_string()
                        } else {
                            imm.to_string()
                        }
                    }
                    ArgumentType::U16    => {
                        (imm as u16).to_string()
                    }
                    ArgumentType::J      => {
                        let j_addr = (text_addr + 4) & 0xF000_0000 | addr << 2;
                        let mut j_label = None;
                        for (label, &label_addr) in program.labels.iter() {
                            if label_addr == j_addr {
                                j_label = Some(label);
                                break;
                            }
                        }

                        j_label.map(|label| label.to_string()).unwrap_or(format!("{:08x}", j_addr))
                    }
                    _ => unreachable!(),
                }.to_ascii_lowercase())
                .collect::<Vec<String>>();
        }
    }

    parts
}
