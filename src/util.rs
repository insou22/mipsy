pub fn ok<T: Default, E>() -> Result<T, E> {
    Ok(T::default())
}

pub trait TruncImm {
    fn trunc_imm(&self) -> i32;
}

impl TruncImm for i32 {
    fn trunc_imm(&self) -> i32 {
        *self as i16 as i32
    }
}
