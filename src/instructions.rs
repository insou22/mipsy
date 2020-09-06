use crate::runtime::CPU;
use crate::types::*;
use crate::util::TruncImm;

// =========== R Instructions ===========
// Shift
pub const SLL   : RInstruction = inst_r("sll",  0b000000, |&(_, rt, rd, shamt), cpu| cpu.registers[rd] = cpu.registers[rt] << shamt);
pub const SRL   : RInstruction = inst_r("srl",  0b000010, |&(_, rt, rd, shamt), cpu| cpu.registers[rd] = (cpu.registers[rt] as u32 >> shamt) as i32);
pub const SRA   : RInstruction = inst_r("sra",  0b000011, |&(_, rt, rd, shamt), cpu| cpu.registers[rd] = cpu.registers[rt] >> shamt);
pub const SLLV  : RInstruction = inst_r("sllv", 0b000100, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rt] << cpu.registers[rs]);
pub const SRLV  : RInstruction = inst_r("srlv", 0b000110, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = (cpu.registers[rt] as u32 >> cpu.registers[rs]) as i32);
pub const SRAV  : RInstruction = inst_r("srav", 0b000111, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rt] >> cpu.registers[rs]);

// Jump
pub const JR    : RInstruction = inst_r("jr",   0b001000, |&(rs, _, _, _), cpu| cpu.pc = cpu.registers[rs] as u32);
pub const JALR  : RInstruction = inst_r("jalr", 0b001001, |&(rs, _, _, _), cpu| { cpu.registers[31] = (cpu.pc + 4) as i32; cpu.pc = cpu.registers[rs] as u32 });

// Interrupt
pub const SYSCALL : RInstruction = inst_r("syscall", 0b001100, |_, cpu| cpu.syscall());
pub const BREAK   : RInstruction = inst_r("break",   0b001101, |_, cpu| cpu.r#break());

// HI/LO
pub const MFHI  : RInstruction = inst_r("mfhi", 0b010000, |&(_, _, rd, _), cpu| cpu.registers[rd] = cpu.hi);
pub const MTHI  : RInstruction = inst_r("mthi", 0b010001, |&(rs, _, _, _), cpu| cpu.hi = cpu.registers[rs]);
pub const MFLO  : RInstruction = inst_r("mflo", 0b010010, |&(_, _, rd, _), cpu| cpu.registers[rd] = cpu.lo);
pub const MTLO  : RInstruction = inst_r("mtlo", 0b010011, |&(rs, _, _, _), cpu| cpu.lo = cpu.registers[rs]);
pub const MULT  : RInstruction = inst_r("mult", 0b011000, |&(_rs, _rt, _, _), _cpu| unimplemented!());
pub const MULTU : RInstruction = inst_r("multu",0b011001, |&(_rs, _rt, _, _), _cpu| unimplemented!());
pub const DIV2  : RInstruction = inst_r("div",  0b011010, |&(rs, rt, _, _), cpu| { cpu.lo = cpu.registers[rs] / cpu.registers[rt]; cpu.hi = cpu.registers[rs] % cpu.registers[rt] });
pub const DIVU2 : RInstruction = inst_r("divu", 0b011011, |&(rs, rt, _, _), cpu| { cpu.lo = (cpu.registers[rs] as u32 / cpu.registers[rt] as u32) as i32; cpu.hi = (cpu.registers[rs] as u32 % cpu.registers[rt] as u32) as i32 });

// Arithmetic
pub const ADD   : RInstruction = inst_r("add",  0b100000, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] + cpu.registers[rt]);
pub const ADDU  : RInstruction = inst_r("addu", 0b100001, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] + cpu.registers[rt]);
pub const SUB   : RInstruction = inst_r("sub",  0b100010, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] - cpu.registers[rt]);
pub const SUBU  : RInstruction = inst_r("subu", 0b100011, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] - cpu.registers[rt]);

// Bitwise
pub const AND   : RInstruction = inst_r("and",  0b100100, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] & cpu.registers[rt]);
pub const OR    : RInstruction = inst_r("or",   0b100101, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] | cpu.registers[rt]);
pub const XOR   : RInstruction = inst_r("xor",  0b100110, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = cpu.registers[rs] ^ cpu.registers[rt]);
pub const NOR   : RInstruction = inst_r("nor",  0b100111, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = !(cpu.registers[rs] | cpu.registers[rt]));

// Set
pub const SLT   : RInstruction = inst_r("slt",  0b101010, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = (cpu.registers[rs] < cpu.registers[rt]) as i32);
pub const SLTU  : RInstruction = inst_r("sltu", 0b101011, |&(rs, rt, rd, _), cpu| cpu.registers[rd] = ((cpu.registers[rs] as u32) < (cpu.registers[rt] as u32)) as i32);


// =========== I Instructions ===========
// Branch
// rt = 0b00000
pub const BLTZ  : IInstruction = inst_i("bltz", 0b000001, |&(rs, _, imm), cpu| { if cpu.registers[rs] < 0 { cpu.add_pc(imm * 4) } });
// rt = 0b00001
pub const BGEZ  : IInstruction = inst_i("bgez", 0b000001, |&(rs, _, imm), cpu| { if cpu.registers[rs] >= 0 { cpu.add_pc(imm * 4) } });
pub const BEQ   : IInstruction = inst_i("beq",  0b000100, |&(rs, rt, imm), cpu| { if cpu.registers[rs] == cpu.registers[rt] { cpu.add_pc(imm * 4) } });
pub const BNE   : IInstruction = inst_i("bne",  0b000101, |&(rs, rt, imm), cpu| { if cpu.registers[rs] != cpu.registers[rt] { cpu.add_pc(imm * 4) } });
pub const BLEZ  : IInstruction = inst_i("blez", 0b000110, |&(rs, _, imm), cpu| { if cpu.registers[rs] <= 0 { cpu.add_pc(imm * 4) } });
pub const BGTZ  : IInstruction = inst_i("bgtz", 0b000111, |&(rs, _, imm), cpu| { if cpu.registers[rs] > 0 { cpu.add_pc(imm * 4) } });

// Arithmetic
pub const ADDI  : IInstruction = inst_i("addi", 0b001000, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.registers[rs] + imm);
pub const ADDIU : IInstruction = inst_i("addiu",0b001001, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.registers[rs] + imm);

// Set
pub const SLTI  : IInstruction = inst_i("slti", 0b001010, |&(rs, rt, imm), cpu| cpu.registers[rt] = (cpu.registers[rs] < imm) as i32);
pub const SLTIU : IInstruction = inst_i("sltiu",0b001011, |&(rs, rt, imm), cpu| cpu.registers[rt] = ((cpu.registers[rs] as u32) < imm as u32) as i32);

// Bitwise
pub const ANDI  : IInstruction = inst_i("andi", 0b001100, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.registers[rs] & imm);
pub const ORI   : IInstruction = inst_i("ori",  0b001101, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.registers[rs] | imm);
pub const XORI  : IInstruction = inst_i("xori", 0b001110, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.registers[rs] ^ imm);
pub const LUI   : IInstruction = inst_i("lui",  0b001111, |&(_, rt, imm), cpu| cpu.registers[rt] = (imm << 16) as i32);

// Coprocessors?

// Memory
pub const LB    : IInstruction = inst_i("lb",   0b100000, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.get_byte(CPU::add_reg_address(cpu.registers[rs], imm)) as i32);
pub const LH    : IInstruction = inst_i("lh",   0b100001, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.get_half(CPU::add_reg_address(cpu.registers[rs], imm)) as i32);
pub const LW    : IInstruction = inst_i("lw",   0b100011, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.get_word(CPU::add_reg_address(cpu.registers[rs], imm)));
pub const LBU   : IInstruction = inst_i("lbu",  0b100100, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.get_byte(CPU::add_reg_address(cpu.registers[rs], imm)) as u32 as i32);
pub const LHU   : IInstruction = inst_i("lhu",  0b100101, |&(rs, rt, imm), cpu| cpu.registers[rt] = cpu.get_half(CPU::add_reg_address(cpu.registers[rs], imm)) as u32 as i32);
pub const SB    : IInstruction = inst_i("sb",   0b101000, |&(rs, rt, imm), cpu| cpu.set_byte(CPU::add_reg_address(cpu.registers[rs], imm), cpu.registers[rt]));
pub const SH    : IInstruction = inst_i("sh",   0b101001, |&(rs, rt, imm), cpu| cpu.set_half(CPU::add_reg_address(cpu.registers[rs], imm), cpu.registers[rt]));
pub const SW    : IInstruction = inst_i("sw",   0b101011, |&(rs, rt, imm), cpu| cpu.set_word(CPU::add_reg_address(cpu.registers[rs], imm), cpu.registers[rt]));
pub const LWC1  : IInstruction = inst_i("lwc1", 0b110001, |&(_rs, _rt, _imm), _cpu| unimplemented!());
pub const SWC1  : IInstruction = inst_i("swc1", 0b111001, |&(_rs, _rt, _imm), _cpu| unimplemented!());


// =========== J Instructions ===========
pub const J     : JInstruction = inst_j("j",    0b000010, |&j, cpu| cpu.pc = (cpu.pc & 0xF0000000) | ((j & 0x03FFFFFF) << 2));
pub const JAL   : JInstruction = inst_j("jal",  0b000011, |&j, cpu| { cpu.registers[31] = (cpu.pc + 4) as i32; cpu.pc = (cpu.pc & 0xF0000000) | ((j & 0x03FFFFFF) << 2) });


// =========== Pseudo Instructions ===========
pub static NOP  : RPseudoInstruction = inst_psuedo_r("nop", |_| vec![
    wrap_r(&SLL, (0, 0, 0, 0)),
]);

pub static MOVE : RPseudoInstruction = inst_psuedo_r("move", |&(rs, _, rd, _)| vec![
    wrap_r(&ADDU, (rd, rs, 0, 0)),
]);

pub static B    : IPseudoInstruction = inst_psuedo_i("b", |&(_, _, imm)| vec![
    wrap_i(&BEQ, (0, 0, imm.trunc_imm())),
]);

pub static BEQZ : IPseudoInstruction = inst_psuedo_i("beqz", |&(rs, _, imm)| vec![
    wrap_i(&BEQ, (rs, 0, imm.trunc_imm())),
]);

pub static BGE  : IPseudoInstruction = inst_psuedo_i("bge", |&(rs, rt, imm)| vec![
    wrap_r(&SLT, (rs, rt, 1, 0)),
    wrap_i(&BEQ, (1, 0, imm.trunc_imm())),
]);

pub static BGEU : IPseudoInstruction = inst_psuedo_i("bgeu", |&(rs, rt, imm)| vec![
    wrap_r(&SLTU, (rs, rt, 1, 0)),
    wrap_i(&BEQ, (1, 0, imm.trunc_imm())),
]);

pub static BLT  : IPseudoInstruction = inst_psuedo_i("blt", |&(rs, rt, imm)| vec![
    wrap_r(&SLT, (rs, rt, 1, 0)),
    wrap_i(&BNE, (1, 0, imm.trunc_imm())),
]);

pub static BLTU : IPseudoInstruction = inst_psuedo_i("bltu", |&(rs, rt, imm)| vec![
    wrap_r(&SLTU, (rs, rt, 1, 0)),
    wrap_i(&BNE, (1, 0, imm.trunc_imm())),
]);

pub static BLEU : IPseudoInstruction = inst_psuedo_i("bleu", |&(rs, rt, imm)| vec![
    wrap_r(&SLTU, (rt, rs, 1, 0)),
    wrap_i(&BEQ, (1, 0, imm.trunc_imm())),
]);

pub static BLE  : IPseudoInstruction = inst_psuedo_i("ble", |&(rs, rt, imm)| vec![
    wrap_r(&SLT, (rt, rs, 1, 0)),
    wrap_i(&BEQ, (1, 0, imm.trunc_imm())),
]);

pub static BGT  : IPseudoInstruction = inst_psuedo_i("bgt", |&(rs, rt, imm)| vec![
    wrap_r(&SLT, (rt, rs, 1, 0)),
    wrap_i(&BNE, (1, 0, imm.trunc_imm())),
]);

pub static BGTU : IPseudoInstruction = inst_psuedo_i("bgtu", |&(rs, rt, imm)| vec![
    wrap_r(&SLTU, (rt, rs, 1, 0)),
    wrap_i(&BNE, (1, 0, imm.trunc_imm())),
]);

pub static MUL  : RPseudoInstruction = inst_psuedo_r("mul", |&(rd, rs, rt, _)| vec![
    wrap_r(&MULTU, (rs, rt, 0, 0)),
    wrap_r(&MFLO, (0, 0, rd, 0)),
]);

pub static DIV3 : RPseudoInstruction = inst_psuedo_r("div", |&(rd, rs, rt, _)| vec![
    wrap_r(&DIV2, (rs, rt, 0, 0)),
    wrap_r(&MFLO, (0, 0, rd, 0)),
]);

pub static DIVU3: RPseudoInstruction = inst_psuedo_r("divu", |&(rd, rs, rt, _)| vec![
    wrap_r(&DIVU2, (rs, rt, 0, 0)),
    wrap_r(&MFLO,  (0, 0, rd, 0)),
]);

pub static REM : RPseudoInstruction = inst_psuedo_r("rem", |&(rd, rs, rt, _)| vec![
    wrap_r(&DIV2, (rs, rt, 0, 0)),
    wrap_r(&MFHI, (0, 0, rd, 0)),
]);

pub static REMU: RPseudoInstruction = inst_psuedo_r("remu", |&(rd, rs, rt, _)| vec![
    wrap_r(&DIVU2, (rs, rt, 0, 0)),
    wrap_r(&MFHI,  (0, 0, rd, 0)),
]);

pub static NEG: RPseudoInstruction = inst_psuedo_r("neg", |&(rd, rs, _, _)| vec![
    wrap_r(&SUB, (0, rs, rd, 0)),
]);

pub static NOT: RPseudoInstruction = inst_psuedo_r("not", |&(rd, rs, _, _)| vec![
    wrap_r(&NOR, (rs, 0, rd, 0)),
]);

pub static LI   : IPseudoInstruction = inst_psuedo_i("li", |&(_, rt, imm)| {
    if imm as u32 & 0xFFFF0000 == imm as u32 & 0x8000 {
        vec![wrap_i(&ADDIU, (0, rt, imm.trunc_imm()))]
    } else if imm & 0xFFFF == 0 {
        vec![wrap_i(&LUI, (0, rt, imm >> 16))]
    } else {
        vec![
            wrap_i(&LUI, (0, rt, imm >> 16)),
            wrap_i(&ADDIU, (0, rt, imm.trunc_imm())),
        ]
    }
});

pub static LA   : IPseudoInstruction = inst_psuedo_i("la", LI.expand);

////////////////////////////////////////////////////////////////////////////////


// =========== Traits ===========
pub trait Instruction {
    type Param;

    fn info(&self) -> InstructionInfo;
    fn exec(&self, param: &Self::Param, cpu: &mut CPU);
}

pub trait StaticInstruction<'a> {
    type Param;

    fn inst(&self) -> &'a dyn Instruction<Param = Self::Param>;
    fn param(&'a self) -> &'a Self::Param;
    fn exec(&self, cpu: &mut CPU);
}

pub enum PseudoExpand<'a> {
    R(Box<dyn StaticInstruction<'a, Param=RParam>>),
    I(Box<dyn StaticInstruction<'a, Param=IParam>>),
    J(Box<dyn StaticInstruction<'a, Param=JParam>>),
}

pub trait PseudoInstruction<'a> {
    type Param;

    fn name(&self) -> &'static str;
    fn size(&self, param: &Self::Param) -> usize;
    fn expand(&self, param: &Self::Param) -> Vec<PseudoExpand<'a>>;
}


// =========== Structs ===========

#[derive(Copy, Clone)]
pub struct InstructionInfo {
    opcode: Opcode,
    name: &'static str,
}

pub struct RInstruction {
    info: InstructionInfo,
    exec: fn(&RParam, &mut CPU),
}

pub struct RStaticInstruction<'a> {
    inst: &'a RInstruction,
    param: RParam,
}

pub struct IInstruction {
    info: InstructionInfo,
    exec: fn(&IParam, &mut CPU),
}

pub struct IStaticInstruction<'a> {
    inst: &'a IInstruction,
    param: IParam,
}

pub struct JInstruction {
    info: InstructionInfo,
    exec: fn(&JParam, &mut CPU),
}

pub struct JStaticInstruction<'a> {
    inst: &'a JInstruction,
    param: JParam,
}

pub struct RPseudoInstruction<'a> {
    name: &'static str,
    pub expand: fn(&RParam) -> Vec<PseudoExpand<'a>>,
}

pub struct IPseudoInstruction<'a> {
    name: &'static str,
    pub expand: fn(&IParam) -> Vec<PseudoExpand<'a>>,
}

pub struct JPseudoInstruction<'a> {
    name: &'static str,
    pub expand: fn(&JParam) -> Vec<PseudoExpand<'a>>,
}


// =========== Impl's ===========

impl Instruction for RInstruction {
    type Param = RParam;

    fn exec(&self, param: &Self::Param, cpu: &mut CPU) {
        (self.exec)(param, cpu);
    }

    fn info(&self) -> InstructionInfo {
        self.info
    }
}

impl<'a> StaticInstruction<'a> for RStaticInstruction<'a> {
    type Param = RParam;

    fn inst(&self) -> &'a dyn Instruction<Param = Self::Param> {
        self.inst
    }

    fn param(&'a self) -> &'a Self::Param {
        &self.param
    }

    fn exec(&self, cpu: &mut CPU) {
        (self.inst.exec)(&self.param, cpu);
    }
}

impl Instruction for IInstruction {
    type Param = IParam;

    fn exec(&self, param: &Self::Param, cpu: &mut CPU) {
        (self.exec)(param, cpu);
    }

    fn info(&self) -> InstructionInfo {
        self.info
    }
}

impl<'a> StaticInstruction<'a> for IStaticInstruction<'a> {
    type Param = IParam;

    fn inst(&self) -> &'a dyn Instruction<Param = Self::Param> {
        self.inst
    }

    fn param(&'a self) -> &'a Self::Param {
        &self.param
    }

    fn exec(&self, cpu: &mut CPU) {
        (self.inst.exec)(&self.param, cpu);
    }
}

impl Instruction for JInstruction {
    type Param = JParam;

    fn exec(&self, param: &Self::Param, cpu: &mut CPU) {
        (self.exec)(param, cpu);
    }

    fn info(&self) -> InstructionInfo {
        self.info
    }
}

impl<'a> StaticInstruction<'a> for JStaticInstruction<'a> {
    type Param = JParam;

    fn inst(&self) -> &'a dyn Instruction<Param = Self::Param> {
        self.inst
    }

    fn param(&'a self) -> &'a Self::Param {
        &self.param
    }

    fn exec(&self, cpu: &mut CPU) {
        (self.inst.exec)(&self.param, cpu);
    }
}

impl<'a> PseudoInstruction<'a> for RPseudoInstruction<'a> {
    type Param = RParam;

    fn name(&self) -> &'static str {
        self.name
    }

    fn size(&self, param: &Self::Param) -> usize {
        self.expand(param).len()
    }

    fn expand(&self, param: &Self::Param) -> Vec<PseudoExpand<'a>> {
        (self.expand)(param)
    }
}

impl<'a> PseudoInstruction<'a> for IPseudoInstruction<'a> {
    type Param = IParam;

    fn name(&self) -> &'static str {
        self.name
    }

    fn size(&self, param: &Self::Param) -> usize {
        self.expand(param).len()
    }

    fn expand(&self, param: &Self::Param) -> Vec<PseudoExpand<'a>> {
        (self.expand)(param)
    }
}

impl<'a> PseudoInstruction<'a> for JPseudoInstruction<'a> {
    type Param = JParam;

    fn name(&self) -> &'static str {
        self.name
    }

    fn size(&self, param: &Self::Param) -> usize {
        self.expand(param).len()
    }

    fn expand(&self, param: &Self::Param) -> Vec<PseudoExpand<'a>> {
        (self.expand)(param)
    }
}

const fn inst_r(name: &'static str, opcode: Opcode, exec: fn(&RParam, &mut CPU)) -> RInstruction {
    RInstruction {
        info: InstructionInfo {
            opcode,
            name,
        },
        exec
    }
}

const fn inst_i(name: &'static str, opcode: Opcode, exec: fn(&IParam, &mut CPU)) -> IInstruction {
    IInstruction {
        info: InstructionInfo {
            opcode,
            name,
        },
        exec
    }
}

const fn inst_j(name: &'static str, opcode: Opcode, exec: fn(&JParam, &mut CPU)) -> JInstruction {
    JInstruction {
        info: InstructionInfo {
            opcode,
            name,
        },
        exec
    }
}

const fn inst_static_r(inst: &RInstruction, param: RParam) -> RStaticInstruction<'_> {
    RStaticInstruction {
        inst,
        param,
    }
}

const fn inst_static_i(inst: &IInstruction, param: IParam) -> IStaticInstruction<'_> {
    IStaticInstruction {
        inst,
        param,
    }
}

const fn inst_static_j(inst: &JInstruction, param: JParam) -> JStaticInstruction<'_> {
    JStaticInstruction {
        inst,
        param,
    }
}

fn wrap_r(inst: &'static RInstruction, param: RParam) -> PseudoExpand<'_> {
    PseudoExpand::R(Box::new(inst_static_r(inst, param)))
}

fn wrap_i(inst: &'static IInstruction, param: IParam) -> PseudoExpand<'_> {
    PseudoExpand::I(Box::new(inst_static_i(inst, param)))
}

fn wrap_j(inst: &'static JInstruction, param: JParam) -> PseudoExpand<'_> {
    PseudoExpand::J(Box::new(inst_static_j(inst, param)))
}

const fn inst_psuedo_r<'a>(name: &'static str, expand: fn(&RParam) -> Vec<PseudoExpand<'a>>) -> RPseudoInstruction<'a> {
    RPseudoInstruction {
        name,
        expand,
    }
}

const fn inst_psuedo_i<'a>(name: &'static str, expand: fn(&IParam) -> Vec<PseudoExpand<'a>>) -> IPseudoInstruction<'a> {
    IPseudoInstruction {
        name,
        expand,
    }
}

const fn inst_psuedo_j<'a>(name: &'static str, expand: fn(&JParam) -> Vec<PseudoExpand<'a>>) -> JPseudoInstruction<'a> {
    JPseudoInstruction {
        name,
        expand,
    }
}
