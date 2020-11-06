use std::fmt::{
    Display,
    Formatter,
    Error,
    LowerHex,
};
use super::State;
use crate::{inst::register::Register, util::Safe};


impl Display for State {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> { 
        fmt.write_str("State {\n")?;
        fmt.write_str("    pages: {\n")?; // WIP
        
        let mut sorted: Vec<(&u32, &Box<[Safe<u8>]>)> = self.pages.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        let mut first = true;
        for (base_addr, page) in sorted {
            if first {
                first = false;
            } else {
                fmt.write_str("\n")?;
            }

            for x in 0..(4096/16) {
                let mut any_init = false;
                for y in 0..16 {
                    if matches!(page[x * 16 + y], Safe::Valid(_)) {
                        any_init = true;
                        break;
                    }
                }

                if any_init {
                    fmt.write_str(&format!("        0x{:08x}: [", base_addr + x as u32 * 16))?;
                    for y in 0..16 {
                        if y != 0 && y % 4 == 0 {
                            fmt.write_str("  ")?;
                        }

                        match page[x * 16 + y] {
                            Safe::Valid(b) => fmt.write_str(&format!("{:02x}", b))?,
                            Safe::Uninitialised => fmt.write_str("__")?,
                        }

                        if y != 15 {
                            fmt.write_str(", ")?;
                        }
                    }
                    fmt.write_str("]\n")?;
                }
            }
        }

        fmt.write_str("    },\n")?;
        fmt.write_str("    pc: ")?;
        fmt.write_str(&format!("0x{:08x}", self.pc))?;
        fmt.write_str(",\n")?;
        fmt.write_str("    registers: {\n")?;
        
        for (reg, &value) in self.registers.iter().enumerate() {
            match value {
                Safe::Valid(value) => {
                    fmt.write_str(&format!("        ${}: 0x{:08x}\n", Register::from_number(reg as i32).unwrap().to_str().to_ascii_lowercase(), value))?;
                },
                Safe::Uninitialised => {}
            }
        }
        
        fmt.write_str("    },\n")?;
        fmt.write_str("    hi: ")?;
        Display::fmt(&self.hi, fmt)?;
        fmt.write_str(",\n")?;
        fmt.write_str("    lo: ")?;
        Display::fmt(&self.lo, fmt)?;
        fmt.write_str(",\n")?;

        fmt.write_str("}\n")?;

        Ok(())
    }
}

impl<T> Display for Safe<T>
where T: Display {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> { 
        match self {
            Self::Valid(t) => t.fmt(fmt)?,
            Self::Uninitialised => fmt.write_str("Uninitialised")?,
        }

        Ok(())
    }
}

impl LowerHex for Safe<i32> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> { 
        match self {
            Self::Valid(t) => fmt.write_str(&format!("0x{:08x}", t))?,
            Self::Uninitialised => fmt.write_str("Uninitialised")?,
        }

        Ok(())
    }
}
