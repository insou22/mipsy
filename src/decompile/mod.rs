use crate::compile::context::Program;
use crate::inst::instruction::{InstSet, InstFormat, RuntimeSignature};
use crate::inst::register::Register;

pub fn decompile(program: &Program, iset: &InstSet) -> String {
    let mut text = String::new();
    
    let mut text_addr = crate::compile::compiler::TEXT_BOT;

    for &word in &program.text {
        for (label, &addr) in program.labels.iter() {
            if addr == text_addr {
                text.push_str(&format!("\n{}: \n", label));
            }
        }

        let opcode = word >> 26;
        let rs =    (word >> 21) & 0x1F;
        let rt =    (word >> 16) & 0x1F;
        let rd =    (word >> 11) & 0x1F;
        let shamt = (word >> 6) & 0x1F;
        let funct =  word & 0x3F;
        let imm =    (word & 0xFFFF) as i16;
        let addr =   word & 0x3FFFFFF;
        
        let mut inst = None;

        for native_inst in &iset.native_set {
            match &native_inst.runtime {
                RuntimeSignature::R { funct: inst_funct } => {
                    if opcode != 0 || *inst_funct as u32 != funct {
                        continue;
                    }
                }

                RuntimeSignature::I { opcode: inst_opcode, .. } | 
                RuntimeSignature::J { opcode: inst_opcode, .. } => {
                    if *inst_opcode as u32 != opcode {
                        continue;
                    }
                }
            }

            inst = Some(native_inst);
            break;
        }

        text.push_str(&format!("0x{:08x} [0x{:08x}]    ", text_addr, word));

        if let Some(inst) = inst {

            if inst.name == "sll" && rd == 0 && rt == 0 && shamt == 0 {
                text.push_str("nop");
            } else {
                text.push_str(&format!("{}\t", inst.name));

                text.push_str(
                    &match inst.compile.format {
                        InstFormat::R0 =>     format!(""),
                        InstFormat::Rd =>     format!(" ${}", Register::u32_to_str(rd)),
                        InstFormat::Rs =>     format!(" ${}", Register::u32_to_str(rs)),
                        InstFormat::RdRs =>   format!(" ${}, ${}", Register::u32_to_str(rd), Register::u32_to_str(rs)),
                        InstFormat::RsRt =>   format!(" ${}, ${}", Register::u32_to_str(rs), Register::u32_to_str(rt)),
                        InstFormat::RdRsRt => format!(" ${}, ${}, ${}", Register::u32_to_str(rd), Register::u32_to_str(rs), Register::u32_to_str(rt)),
                        InstFormat::RdRtRs => format!(" ${}, ${}, ${}", Register::u32_to_str(rd), Register::u32_to_str(rt), Register::u32_to_str(rs)),
                        InstFormat::RdRtSa => format!(" ${}, ${}, {}", Register::u32_to_str(rd), Register::u32_to_str(rt), shamt),
                        InstFormat::J =>      format!(" 0x{:x}", addr),
                        InstFormat::Im =>     format!(" {}", imm),
                        InstFormat::RsIm =>   format!(" ${}, {}", Register::u32_to_str(rs), imm),
                        InstFormat::RtIm =>   format!(" ${}, {}", Register::u32_to_str(rt), imm),
                        InstFormat::RsRtIm => format!(" ${}, ${}, {}", Register::u32_to_str(rs), Register::u32_to_str(rt), imm),
                        InstFormat::RtRsIm => format!(" ${}, ${}, {}", Register::u32_to_str(rt), Register::u32_to_str(rs), imm),
                        InstFormat::RtImRs => format!(" ${}, {}(${})", Register::u32_to_str(rt), imm, Register::u32_to_str(rs)),
                    
                        _ => unreachable!(),
                    }.to_ascii_lowercase()
                );
            }
        } else {
            text.push_str(&format!("# Unknown instruction: opcode={} funct={}", opcode, funct));
        }

        text.push('\n');
        text_addr += 4;
    }

    text
}
