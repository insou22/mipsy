#[derive(Debug)]
pub enum RuntimeError {
    Str(&'static str),
    String(String),

    PageNotExist(u32),
    UninitializedMemory(u32),
    UninitializedRegister(u32),
    UninitializedHi,
    UninitializedLo,

    IntegerOverflow,
}
