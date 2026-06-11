use crate::isa;
use crate::memory::{Memory, Region};
use std::fmt;

// --------------------------------------------------------------------------------
// CPU errors
// --------------------------------------------------------------------------------

pub enum CpuError {
    InvalidOpcode(u16),
    Halted,
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuError::InvalidOpcode(addr) => write!(f, "failed to parse instruction {addr}"),
            CpuError::Halted => write!(f, "execution halted"),
        }
    }
}

// --------------------------------------------------------------------------------
// CPU
// --------------------------------------------------------------------------------

pub struct Cpu {
    memory: Memory,
    register_file: Memory,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(1_048_576),
            register_file: Memory::new(28),
        }
    }

    pub fn run(&mut self) -> Result<(), CpuError> {
        loop {
            let ip = self.read_register(&isa::IP);
            let buf = self.fetch_instruction_buffer();
            let (instruction, consumed) = isa::decode(buf).ok_or(CpuError::InvalidOpcode(ip))?;

            let next_ip = ip + consumed as u16;
            self.write_register(&isa::IP, next_ip);

            println!("{instruction} ({consumed} bytes) [{:#06X}]", ip);
            self.execute(&instruction)?;
        }
    }

    fn execute(&mut self, instruction: &isa::Instruction) -> Result<(), CpuError> {
        match instruction {
            isa::Instruction::Mov(_, _) => {
                todo!("execute mov")
            }
        }
    }

    fn fetch_instruction_buffer(&self) -> &[u8] {
        let ip = self.read_register(&isa::IP);
        let cs = self.read_register(&isa::CS);
        let address = ((cs as usize) << 4) + (ip as usize);

        // NOTE: possible edge case exists where the instruction segment
        // is at 100% capacity and the last instruction will fail.
        self.memory
            .read(&Region { address, size: 6 })
            .expect("instruction fetch failed")
    }

    fn read_register(&self, region: &Region) -> u16 {
        let bytes = self
            .register_file
            .read(region)
            .expect("register read failed");
        let mut buf = [0u8; 2];
        buf[..bytes.len()].copy_from_slice(bytes);
        u16::from_le_bytes(buf)
    }

    fn write_register(&mut self, region: &Region, value: u16) {
        self.register_file
            .write(&value.to_le_bytes()[..region.size], region)
            .expect("register write failed");
    }

    fn decode_register(&self, rm: u8, w: u8) -> isa::Register {
        isa::REG_ENCODING[(rm << 1 | w) as usize]
    }

    pub fn load_program(&mut self, bytes: &[u8], segment: u16) {
        let address = (segment as usize) << 4;
        self.memory
            .write(
                bytes,
                &Region {
                    address,
                    size: bytes.len(),
                },
            )
            .expect("load failed");
        self.write_register(&isa::CS, segment);
        self.write_register(&isa::IP, 0);
        self.write_register(&isa::SS, segment);
        self.write_register(&isa::SP, 0x0400);
        self.write_register(&isa::DS, segment);
    }
}
