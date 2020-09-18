use crate::instructions::*;
use crate::types::*;

/*pub fn inst_type(inst: RawInstruction) -> InstructionType {
    let opcode = inst >> 26;

    match opcode {
        // R-Type
        0b000000 => InstructionType::R(),
        // J-Type
        0b000010 | 0b000011 => InstructionType::J,
        // I-Type
        _ => InstructionType::I,
    }
}

pub fn decode_r(inst: RawInstruction) -> RStaticInstruction<'static> {
    let funct = inst & 0x3F;

    let rs    = (inst >> 21 & 0x1F) as usize;
    let rt    = (inst >> 16 & 0x1F) as usize;
    let rd    = (inst >> 11 & 0x1F) as usize;
    let shamt = (inst >> 6  & 0x1F) as usize;

    let params = (rs, rt, rd, shamt);

    match funct {
        0b100000 => &ADD,
        0b100001 => &ADDU,
        0b100100 => &AND,
        0b001101 => &BREAK,
        0b011010 => &DIV2,
        0b011011 => &DIVU2,
        0b001001 => &JALR,
        0b001000 => &JR,
        0b010000 => &MFHI,
        0b010010 => &MFLO,
        0b010001 => &MTHI,
        0b010011 => &MTLO,
        0b011000 => &MULT,
        0b011001 => &MULTU,
        0b100111 => &NOR,
        0b100101 => &OR,
        0b000000 => &SLL,
        0b000100 => &SLLV,
        0b101010 => &SLT,
        0b101011 => &SLTU,
        0b000011 => &SRA,
        0b000111 => &SRAV,
        0b000010 => &SRL,
        0b000110 => &SRLV,
        0b100010 => &SUB,
        0b100011 => &SUBU,
        0b001100 => &SYSCALL,
        0b100110 => &XOR,
        other => panic!("Failed to decode R-Type instruction with funct {}", other),
    }.fill(params)
}

pub fn decode_i(inst: RawInstruction) -> IStaticInstruction<'static> {
    let opcode = inst >> 26;

    let rs  = (inst >> 21 & 0x1F)   as usize;
    let rt  = (inst >> 16 & 0x1F)   as usize;
    let imm = (inst       & 0xFFFF) as i32;

    let params = (rs, rt, imm);

    match opcode {
        0b001000 => &ADDI,
        0b001001 => &ADDIU,
        0b001100 => &ANDI,
        0b000100 => &BEQ,
        0b000001 => {
            match rt {
                0b000000 => &BLTZ,
                0b000001 => &BGEZ,
                other => panic!("Failed to decode I-Type BLTZ / BGEZ instruction with opcode {} and rt {}", opcode, other),
            }
        }
        0b000111 => &BGTZ,
        0b000110 => &BLEZ,
        0b000101 => &BNE,
        0b100000 => &LB,
        0b100100 => &LBU,
        0b100001 => &LH,
        0b100101 => &LHU,
        0b001111 => &LUI,
        0b100011 => &LW,
        0b001101 => &ORI,
        0b101000 => &SB,
        0b001010 => &SLTI,
        0b001011 => &SLTIU,
        0b101001 => &SH,
        0b101011 => &SW,
        0b001110 => &XORI,
        other => panic!("Failed to decode I-Type instruction with opcode {}", other),
    }.fill(params)
}

pub fn decode_j(inst: RawInstruction) -> JStaticInstruction<'static> {
    let opcode = inst >> 26;

    let target = (inst & 0x03FFFFFF) as u32;
    let params = target;

    match opcode {
        0b000010 => &J,
        0b100011 => &JAL,
        _ => unreachable!(),
    }.fill(params)
}*/