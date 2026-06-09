use crate::memory::Region;
use std::fmt;

// --------------------------------------------------------------------------------
// General purpose registers
// --------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub enum Register {
    AL,
    AX,
    CL,
    CX,
    DL,
    DX,
    BL,
    BX,
    AH,
    SP,
    CH,
    BP,
    DH,
    SI,
    BH,
    DI,
}

#[rustfmt::skip]
impl Register {
    pub fn region(self) -> Region {
        match self {
            Register::AL => Region { address: 0, size: 1, },
            Register::AX => Region { address: 0, size: 2, },
            Register::CL => Region { address: 2, size: 1, },
            Register::CX => Region { address: 2, size: 2, },
            Register::DL => Region { address: 4, size: 1, },
            Register::DX => Region { address: 4, size: 2, },
            Register::BL => Region { address: 6, size: 1, },
            Register::BX => Region { address: 6, size: 2, },
            Register::AH => Region { address: 1, size: 1, },
            Register::SP => Region { address: 8, size: 2, },
            Register::CH => Region { address: 3, size: 1, },
            Register::BP => Region { address: 10, size: 2, },
            Register::DH => Region { address: 5, size: 1, },
            Register::SI => Region { address: 12, size: 2, },
            Register::BH => Region { address: 7, size: 1, },
            Register::DI => Region { address: 14, size: 2, },
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::AL => write!(f, "al"),
            Register::AX => write!(f, "ax"),
            Register::CL => write!(f, "cl"),
            Register::CX => write!(f, "cx"),
            Register::DL => write!(f, "dl"),
            Register::DX => write!(f, "dx"),
            Register::BL => write!(f, "bl"),
            Register::BX => write!(f, "bx"),
            Register::AH => write!(f, "ah"),
            Register::SP => write!(f, "sp"),
            Register::CH => write!(f, "ch"),
            Register::BP => write!(f, "bp"),
            Register::DH => write!(f, "dh"),
            Register::SI => write!(f, "si"),
            Register::BH => write!(f, "bh"),
            Register::DI => write!(f, "di"),
        }
    }
}

// RM << 1 | W --> Register
pub const REG_ENCODING: [Register; 16] = [
    Register::AL,
    Register::AX, // 000
    Register::CL,
    Register::CX, // 001
    Register::DL,
    Register::DX, // 010
    Register::BL,
    Register::BX, // 011
    Register::AH,
    Register::SP, // 100
    Register::CH,
    Register::BP, // 101
    Register::DH,
    Register::SI, // 110
    Register::BH,
    Register::DI, // 111
];

// --------------------------------------------------------------------------------
// System registers
// --------------------------------------------------------------------------------

#[rustfmt::skip]
pub const CS: Region = Region { address: 16, size: 2, };
#[rustfmt::skip]
pub const DS: Region = Region { address: 18, size: 2, };
#[rustfmt::skip]
pub const SS: Region = Region { address: 20, size: 2, };
#[rustfmt::skip]
pub const ES: Region = Region { address: 22, size: 2, };
#[rustfmt::skip]
pub const IP: Region = Region { address: 24, size: 2, };
#[rustfmt::skip]
pub const FLAGS: Region = Region { address: 26, size: 2, };

// --------------------------------------------------------------------------------
// Operand
// --------------------------------------------------------------------------------

pub enum Operand {
    Register(Register),
    Memory(Region),
    Immediate(u16),
}

// --------------------------------------------------------------------------------
// Instructions
// --------------------------------------------------------------------------------
