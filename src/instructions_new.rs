use crate::runtime::CPU;
use crate::types::*;

pub enum RType {
    R0,
    Rd,
    Rs,
    RdRs,
    RsRt,
    RdRsRt,
    RdRtRs,
    RdRtSa,
}

pub enum IType {
    RsIm,
    RtIm,
    RsRtIm,
    RtRsIm,
    RtImRs,
}

pub enum InstructionType {
    R(RType), 
    I(IType), 
    J(Box<dyn JInstruction>)
}

trait Instruction {
    fn inst_type(&self) -> InstructionType;
    fn name(&self) -> &'static str;
}

trait R0Instruction {
    fn exec(&self, cpu: &mut CPU);
}

trait RdInstruction {
    fn exec(&self, cpu: &mut CPU, rd: RegisterIndex);
}

trait RsInstruction {
    fn exec(&self, cpu: &mut CPU, rs: RegisterIndex);
}

trait RdRsInstruction {
    fn exec(&self, cpu: &mut CPU, rd: RegisterIndex, rs: RegisterIndex);
}

trait RsRtInstruction {
    fn exec(&self, cpu: &mut CPU, rs: RegisterIndex, rt: RegisterIndex);
}

trait RdRsRtInstruction {
    fn exec(&self, cpu: &mut CPU, rd: RegisterIndex, rs: RegisterIndex, rt: RegisterIndex);
}

trait RdRtRsInstruction {
    fn exec(&self, cpu: &mut CPU, rd: RegisterIndex, rt: RegisterIndex, rs: RegisterIndex);
}

trait RdRtSaInstruction {
    
}

trait IInstruction  {

}

trait JInstruction  {

}