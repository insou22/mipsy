#[derive(Debug)]
pub enum RuntimeError {
    NoInstruction(u32),
    Uninitialised(Uninitialised),

    IntegerOverflow,
    DivisionByZero,
    SbrkNegative,
}

#[derive(Debug)]
pub enum Uninitialised {
    Byte(u32),
    Half(u32),
    Word(u32),
    Register(u32),
    Lo,
    Hi,
}
