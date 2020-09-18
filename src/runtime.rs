use std::collections::HashMap;
use crate::types::*;
use crate::decode::*;
use crate::instructions::InstructionType;
use crate::instructions::StaticInstruction;

pub const PAGE_SIZE: usize = 0x1000; // 4096 bytes
pub const REGISTERS: usize = 32;

#[derive(Copy, Clone, Default)]
pub struct Byte {
    value: u8,
    initialized: bool,
}

pub struct CPU {
    pub pages: HashMap<Address, [Byte; PAGE_SIZE]>,
    pub registers: [Register; REGISTERS],
    pub fp_registers: [FRegister; REGISTERS],
    pub pc: Address,
    pub hi: Register,
    pub lo: Register,
}

impl CPU {
    pub fn step(&mut self) {
        let inst = self.get_word(self.pc) as u32;
        self.pc += 4;

        /*match inst_type(inst) {
            InstructionType::R(_) => decode_r(inst).exec(self),
            InstructionType::I(_) => decode_i(inst).exec(self),
            InstructionType::J(_) => decode_j(inst).exec(self),
        }*/
    }

    pub fn syscall(&mut self) {
        // temporary
        match self.registers[2] {
            1  => print!("{}", self.registers[4]),
            4  => {
                let mut addr = self.registers[4];
                let mut b = self.get_byte(addr as u32);

                while b != 0 {
                    print!("{}", b as u8 as char);

                    addr += 1;
                    b = self.get_byte(addr as u32);
                }
            },
            5  => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("pls input");
                self.registers[2] = input.trim().parse::<i32>().unwrap();
            },
            10 => {
                panic!("happy panic :)");
            },
            11 => {
                print!("{}", self.registers[4] as u8 as char);
            },
            other => println!("Unknown syscall: {}", other)
        }
    }

    pub fn r#break(&mut self) {
        unimplemented!()
    }

    pub fn page_baseaddr(addr: Address) -> Address {
        addr & 0xFFFFF000
    }

    pub fn page_offset(addr: Address) -> usize {
        (addr % PAGE_SIZE as u32) as usize
    }

    pub fn set_byte(&mut self, addr: Address, value: i32) {
        let base = CPU::page_baseaddr(addr);

        let page = self.pages.entry(base)
            .or_insert([Byte::default(); PAGE_SIZE]);

        let offset = CPU::page_offset(addr);
        page[offset].value = value as u8;
        page[offset].initialized = true;
    }

    pub fn set_half(&mut self, addr: Address, value: i32) {
        self.set_byte(addr,     value & 0x00FF);
        self.set_byte(addr + 1, value & 0xFF00);
    }

    pub fn set_word(&mut self, addr: Address, value: i32) {
        self.set_byte(addr,     value &  0x000000FF);
        self.set_byte(addr + 1, value &  0x0000FF00);
        self.set_byte(addr + 2, value &  0x00FF0000);
        self.set_byte(addr + 3, value &  0xFF000000 as u32 as i32);
    }

    pub fn get_byte(&self, addr: Address) -> i8 {
        let base = CPU::page_baseaddr(addr);

        if let Some(page) = self.pages.get(&base) {
            let offset = CPU::page_offset(addr);
            let byte = page[offset];

            if byte.initialized {
                byte.value as i8
            } else {
                panic!("insert error message here")
            }
        } else {
            panic!("insert error message here")
        }
    }

    pub fn get_half(&self, addr: Address) -> i16 {
        if addr % 2 != 0 {
            panic!("alignment error here");
        }

        let b1 = self.get_byte(addr);
        let b2 = self.get_byte(addr + 1);

        (((b2 as u16) << 8) | b1 as u16) as i16
    }

    pub fn get_word(&self, addr: Address) -> i32 {
        if addr % 4 != 0 {
            panic!("alignment error here");
        }

        let b1 = self.get_byte(addr)     as u32;
        let b2 = self.get_byte(addr + 1) as u32;
        let b3 = self.get_byte(addr + 2) as u32;
        let b4 = self.get_byte(addr + 3) as u32;

        (((b4 as u32) << 24) | ((b3 as u32) << 16) | ((b2 as u32) << 8) | b1 as u32) as i32
    }

    pub fn add_reg_address(addr: Register, offset: i32) -> Address {
        (addr as u32).wrapping_add(offset as u32)
    }

    pub fn add_pc(&mut self, offset: i32) {
        self.pc = self.pc.wrapping_add(offset as u32);
    }
}
