pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for char {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl ToBytes for u8 {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl ToBytes for u16 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToBytes for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToBytes for f32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToBytes for f64 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
