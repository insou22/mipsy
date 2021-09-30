use serde::{Serialize, Deserialize};
use crate::error::{InternalError, MipsyInternalResult, compiler};
use std::str::FromStr;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Register {
    Zero,
    At,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    K0,
    K1,
    Gp,
    Sp,
    Fp,
    Ra,
}

pub const REGISTERS: [Register; 32] = [
    Register::Zero, Register::At, 
    Register::V0, Register::V1, 
    Register::A0, Register::A1, Register::A2, Register::A3, 
    Register::T0, Register::T1, Register::T2, Register::T3, 
    Register::T4, Register::T5, Register::T6, Register::T7, 
    Register::S0, Register::S1, Register::S2, Register::S3, 
    Register::S4, Register::S5, Register::S6, Register::S7, 
    Register::T8, Register::T9, 
    Register::K0, Register::K1, 
    Register::Gp, Register::Sp, Register::Fp, 
    Register::Ra,
];

impl FromStr for Register {
    type Err = InternalError;

    fn from_str(name: &str) -> MipsyInternalResult<Self> {
        // $num
        if let Ok(number) = name.parse::<i32>() {
            return Self::from_number(number);
        }

        // $name
        for reg in REGISTERS.iter() {
            if reg.to_str().eq_ignore_ascii_case(name) {
                return Ok(*reg);
            }
        }

        // better error reporting
        if name.starts_with('v') || name.starts_with('a') || 
           name.starts_with('t') || name.starts_with('s') || 
           name.starts_with('k') {
            if let Ok(num) = name[1..].parse::<i32>() {
                return Err(
                    InternalError::Compiler(
                        compiler::Error::NamedRegisterOutOfRange {
                            reg_name: name.chars().next().unwrap(),
                            reg_index: num
                        }
                    )
                );
            }
        }

        // who knows
        Err(
            InternalError::Compiler(
                compiler::Error::UnknownRegister {
                    reg_name: name.to_string(),
                }
            )
        )
    }
}

impl Register {
    pub fn all() -> [Register; 32] {
        REGISTERS
    }

    pub fn to_number(&self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::At   => 1,
            Self::V0   => 2,
            Self::V1   => 3,
            Self::A0   => 4,
            Self::A1   => 5,
            Self::A2   => 6,
            Self::A3   => 7,
            Self::T0   => 8,
            Self::T1   => 9,
            Self::T2   => 10,
            Self::T3   => 11,
            Self::T4   => 12,
            Self::T5   => 13,
            Self::T6   => 14,
            Self::T7   => 15,
            Self::S0   => 16,
            Self::S1   => 17,
            Self::S2   => 18,
            Self::S3   => 19,
            Self::S4   => 20,
            Self::S5   => 21,
            Self::S6   => 22,
            Self::S7   => 23,
            Self::T8   => 24,
            Self::T9   => 25,
            Self::K0   => 26,
            Self::K1   => 27,
            Self::Gp   => 28,
            Self::Sp   => 29,
            Self::Fp   => 30,
            Self::Ra   => 31,
        }
    }

    pub fn from_number(num: i32) -> MipsyInternalResult<Self> {
        match num {
            0  => Ok(Self::Zero),
            1  => Ok(Self::At),
            2  => Ok(Self::V0),
            3  => Ok(Self::V1),
            4  => Ok(Self::A0),
            5  => Ok(Self::A1),
            6  => Ok(Self::A2),
            7  => Ok(Self::A3),
            8  => Ok(Self::T0),
            9  => Ok(Self::T1),
            10 => Ok(Self::T2),
            11 => Ok(Self::T3),
            12 => Ok(Self::T4),
            13 => Ok(Self::T5),
            14 => Ok(Self::T6),
            15 => Ok(Self::T7),
            16 => Ok(Self::S0),
            17 => Ok(Self::S1),
            18 => Ok(Self::S2),
            19 => Ok(Self::S3),
            20 => Ok(Self::S4),
            21 => Ok(Self::S5),
            22 => Ok(Self::S6),
            23 => Ok(Self::S7),
            24 => Ok(Self::T8),
            25 => Ok(Self::T9),
            26 => Ok(Self::K0),
            27 => Ok(Self::K1),
            28 => Ok(Self::Gp),
            29 => Ok(Self::Sp),
            30 => Ok(Self::Fp),
            31 => Ok(Self::Ra),
            _  => Err(
                InternalError::Compiler(
                    compiler::Error::NumberedRegisterOutOfRange {
                        reg_num: num,
                    }
                )
            )
        }
    }

    pub fn from_u32(num: u32) -> MipsyInternalResult<Self> {
        Self::from_number(num as i32)
    }

    pub fn to_u32(&self) -> u32 {
        self.to_number() as u32
    }

    pub fn u32_to_str(num: u32) -> &'static str {
        Self::from_u32(num).unwrap().to_str()
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Zero => "ZERO",
            Self::At   => "AT",
            Self::V0   => "V0",
            Self::V1   => "V1",
            Self::A0   => "A0",
            Self::A1   => "A1",
            Self::A2   => "A2",
            Self::A3   => "A3",
            Self::T0   => "T0",
            Self::T1   => "T1",
            Self::T2   => "T2",
            Self::T3   => "T3",
            Self::T4   => "T4",
            Self::T5   => "T5",
            Self::T6   => "T6",
            Self::T7   => "T7",
            Self::S0   => "S0",
            Self::S1   => "S1",
            Self::S2   => "S2",
            Self::S3   => "S3",
            Self::S4   => "S4",
            Self::S5   => "S5",
            Self::S6   => "S6",
            Self::S7   => "S7",
            Self::T8   => "T8",
            Self::T9   => "T9",
            Self::K0   => "K0",
            Self::K1   => "K1",
            Self::Gp   => "GP",
            Self::Sp   => "SP",
            Self::Fp   => "FP",
            Self::Ra   => "RA",
        }
    }

    pub fn to_lower_str(&self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::At   => "at",
            Self::V0   => "v0",
            Self::V1   => "v1",
            Self::A0   => "a0",
            Self::A1   => "a1",
            Self::A2   => "a2",
            Self::A3   => "a3",
            Self::T0   => "t0",
            Self::T1   => "t1",
            Self::T2   => "t2",
            Self::T3   => "t3",
            Self::T4   => "t4",
            Self::T5   => "t5",
            Self::T6   => "t6",
            Self::T7   => "t7",
            Self::S0   => "s0",
            Self::S1   => "s1",
            Self::S2   => "s2",
            Self::S3   => "s3",
            Self::S4   => "s4",
            Self::S5   => "s5",
            Self::S6   => "s6",
            Self::S7   => "s7",
            Self::T8   => "t8",
            Self::T9   => "t9",
            Self::K0   => "k0",
            Self::K1   => "k1",
            Self::Gp   => "gp",
            Self::Sp   => "sp",
            Self::Fp   => "fp",
            Self::Ra   => "ra",
        }
    }
}