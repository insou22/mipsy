use std::collections::HashMap;

use crate::Binary;
use crate::inst::instruction::{InstSet, ArgumentType, RuntimeSignature};
use crate::inst::register::Register;

pub struct Decompiled {
    pub opcode: u32,
    pub addr: u32,
    pub inst_name: Option<String>,
    pub arguments: Vec<String>,
    pub labels: Vec<String>,
}

pub fn decompile(program: &Binary, iset: &InstSet) -> String {
    let mut text = String::new();
    let unknown_instruction = String::from("# Unknown instruction");

    let decompiled = decompile_into_parts(program, iset);

    let mut keys: Vec<u32> = decompiled.keys().copied().collect();
    keys.sort();

    for (addr, parts) in keys.into_iter().map(|addr| (addr, decompiled.get(&addr).unwrap())) {
        if !parts.labels.is_empty() {
            text.push_str("\n");
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

pub fn decompile_into_parts(program: &Binary, iset: &InstSet) -> HashMap<u32, Decompiled> {
    let mut decompiled = HashMap::new();
    
    let mut text_addr = crate::TEXT_BOT;

    for &word in &program.text {
        let parts = decompile_inst_into_parts(program, iset, word, text_addr);

        decompiled.insert(text_addr, parts);
        text_addr += 4;
    }

    decompiled
}

pub fn decompile_inst_into_parts(program: &Binary, iset: &InstSet, inst: u32, text_addr: u32) -> Decompiled {
    let mut parts = Decompiled {
        opcode: inst,
        addr: text_addr,
        inst_name: None,
        arguments: vec![],
        labels: vec![],
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

    for native_inst in &iset.native_set {
        match &native_inst.runtime {
            RuntimeSignature::R { funct: inst_funct } => {
                if opcode != 0 || *inst_funct as u32 != funct {
                    continue;
                }
            }

            RuntimeSignature::I { opcode: inst_opcode, rt: inst_rt } => {
                if *inst_opcode as u32 != opcode || inst_rt.is_some() && inst_rt.unwrap() as u32 != rt {
                    continue;
                }
            }

            RuntimeSignature::J { opcode: inst_opcode, .. } => {
                if *inst_opcode as u32 != opcode {
                    continue;
                }
            }
        }

        inst = Some(native_inst);
        break;
    }

    if let Some(inst) = inst {
        if inst.name == "sll" && rd == 0 && rt == 0 && shamt == 0 {
            parts.inst_name = Some("nop".to_string());
        } else {
            parts.inst_name = Some(inst.name.to_string());

            parts.arguments = inst.compile.format.iter()
                .map(|arg| match arg {
                    ArgumentType::Rd     => format!("${}", Register::u32_to_str(rd)),
                    ArgumentType::Rt     => format!("${}", Register::u32_to_str(rt)),
                    ArgumentType::Rs     => format!("${}", Register::u32_to_str(rs)),
                    ArgumentType::Shamt  => format!("{}", shamt),
                    ArgumentType::OffRs  => format!("{}(${})", imm, Register::u32_to_str(rs)),
                    ArgumentType::OffRt  => format!("{}(${})", imm, Register::u32_to_str(rt)),
                    ArgumentType::Imm    => {
                        let mut res = None;

                        if inst.compile.relative_label {

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
                            format!("{}", imm)
                        }
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
