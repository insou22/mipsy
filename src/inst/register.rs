use crate::error::RSpimResult;
use crate::error::CompileError;
use crate::cerr;

#[derive(Copy, Clone)]
pub enum Register {
    ZERO,
    AT,
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
    GP,
    SP,
    FP,
    RA,
}

pub const REGISTERS: [Register; 32] = [
    Register::ZERO, Register::AT, 
    Register::V0, Register::V1, 
    Register::A0, Register::A1, Register::A2, Register::A3, 
    Register::T0, Register::T1, Register::T2, Register::T3, 
    Register::T4, Register::T5, Register::T6, Register::T7, 
    Register::S0, Register::S1, Register::S2, Register::S3, 
    Register::S4, Register::S5, Register::S6, Register::S7, 
    Register::T8, Register::T9, 
    Register::K0, Register::K1, 
    Register::GP, Register::SP, Register::FP, 
    Register::RA,
];

impl Register {
    pub fn to_number(&self) -> u8 {
        match self {
            Self::ZERO => 0,
            Self::AT   => 1,
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
            Self::GP   => 28,
            Self::SP   => 29,
            Self::FP   => 30,
            Self::RA   => 31,
        }
    }

    pub fn from_number(num: i32) -> RSpimResult<Self> {
        match num {
            0  => Ok(Self::ZERO),
            1  => Ok(Self::AT),
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
            28 => Ok(Self::GP),
            29 => Ok(Self::SP),
            30 => Ok(Self::FP),
            31 => Ok(Self::RA),
            _  => cerr!(CompileError::NumRegisterOutOfRange(num as i32))
        }
    }

    pub fn from_str(name: &str) -> RSpimResult<Self> {
        // too short
        if name.len() < 2 {
            return cerr!(CompileError::RegisterNameTooShort(name.into()));
        }

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
        if name.starts_with("v") || name.starts_with("a") || 
           name.starts_with("t") || name.starts_with("s") || 
           name.starts_with("k") {
            if let Ok(num) = name[1..].parse::<i32>() {
                return cerr!(CompileError::NamedRegisterOutOfRange { reg_name: name.chars().nth(0).unwrap(), reg_index: num });
            }
        }

        // who knows
        cerr!(CompileError::UnknownRegister(name.into()))
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::ZERO => "ZERO",
            Self::AT   => "AT",
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
            Self::GP   => "GP",
            Self::SP   => "SP",
            Self::FP   => "FP",
            Self::RA   => "RA",
        }
    }
}