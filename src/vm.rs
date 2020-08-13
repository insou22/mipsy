use std::fmt;

pub struct CPU {}

pub struct Instruction {
    name: &'static str,
    invoke: fn(&Instruction, &mut CPU),
}

impl fmt::Debug for Instruction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}

fn foo() {
    let inst = Instruction {
        name: "add",
        invoke: add,
    };
}

fn add(inst: &Instruction, cpu: &mut CPU) {}
