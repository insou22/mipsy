use crate::inst_sig;
use crate::i_format;

inst_sig!(SLL, RdRtSa);


i_format!(R0, R0);
i_format!(Rd, R1, RegType::Rd);
i_format!(Rs, R1, RegType::Rs);
i_format!(RdRs, R2, RegType::Rd, RegType::Rs);
i_format!(RsRt, R2, RegType::Rs, RegType::Rt);
i_format!(RdRsRt, R3, RegType::Rd, RegType::Rs, RegType::Rt);
i_format!(RdRtRs, R3, RegType::Rd, RegType::Rt, RegType::Rs);
i_format!(RdRtSa, R2Im, RegType::Rd, RegType::Rt, ImmType::Shamt);
i_format!(RsIm, R1Im, RegType::Rs, ImmType::Imm);
i_format!(RtIm, R1Im, RegType::Rt, ImmType::Imm);
i_format!(RsRtIm, R2Im, RegType::Rs, RegType::Rt, ImmType::Imm);
i_format!(RtRsIm, R2Im, RegType::Rt, RegType::Rs, ImmType::Imm);
i_format!(RtImRs, R1ImR1, RegType::Rt, ImmType::Imm, RegType::Rs);


pub struct InstructionSignature<'a> {
    name: &'a str,
    format: InstructionFormat,
    pseudo: bool,   
}

impl<'a> InstructionSignature<'a> {
    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn inst_format(&self) -> InstructionFormat {
        self.format
    }

    pub fn pseudo(&self) -> bool {
        self.pseudo
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InstructionFormat {
    R0,
    R1(RegType),
    R2(RegType, RegType),
    R3(RegType, RegType, RegType),
    Im(ImmType),
    R1Im(RegType, ImmType),
    R2Im(RegType, RegType, ImmType),
    R1ImR1(RegType, ImmType, RegType)
}

#[derive(Copy, Clone, Debug)]
pub enum RegType {
    Rd,
    Rs,
    Rt,
}

#[derive(Copy, Clone, Debug)]
pub enum ImmType {
    Imm,
    J,
    Shamt,
}