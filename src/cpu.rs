use crate::isa::{REG_ENCODING, Register};
use crate::memory::{Memory, MemoryError, Region};

struct CPU {
    memory: Memory,
    register_file: Memory,
}

impl CPU {
    fn new() -> Self {
        Self {
            memory: Memory::new(1_048_576),
            register_file: Memory::new(28),
        }
    }

    fn decode_register(&self, rm: u8, w: u8) -> Register {
        REG_ENCODING[(rm << 1 | w) as usize]
    }
}
