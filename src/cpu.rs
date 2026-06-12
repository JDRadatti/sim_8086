use crate::isa;
use crate::memory::{Bus, MemoryError, Region};
use std::fmt;

// --------------------------------------------------------------------------------
// CPU errors
// --------------------------------------------------------------------------------

pub enum CpuError {
    InvalidOpcode(u16),
    Halted,
    Memory(MemoryError),
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuError::InvalidOpcode(addr) => write!(f, "failed to parse instruction {addr}"),
            CpuError::Halted => write!(f, "execution halted"),
            CpuError::Memory(err) => write!(f, "memory error: {err}"),
        }
    }
}

impl From<MemoryError> for CpuError {
    fn from(err: MemoryError) -> Self {
        CpuError::Memory(err)
    }
}

// --------------------------------------------------------------------------------
// CPU
// --------------------------------------------------------------------------------

pub struct Cpu {
    bus: Bus,
}

impl Cpu {
    pub fn new() -> Self {
        Self { bus: Bus::new() }
    }

    pub fn run_decode(&mut self) -> Result<(), CpuError> {
        loop {
            let instruction = self.step()?;
            println!("{instruction}");
        }
    }

    pub fn run_execute(&mut self) -> Result<(), CpuError> {
        loop {
            let instruction = self.step()?;
            self.execute(&instruction)?;
        }
    }

    pub fn step(&mut self) -> Result<isa::Instruction, CpuError> {
        let ip = self.bus.read_u16(&isa::Register::IP.region()).expect("register read failed");
        let buf = self.fetch_instruction_buffer()?;
        let (instruction, consumed) = isa::decode(buf).ok_or(CpuError::InvalidOpcode(ip))?;

        let next_ip = ip + consumed as u16;
        self.bus
            .write_u16(next_ip, &isa::Register::IP.region())
            .expect("register write failed");
        Ok(instruction)
    }

    fn execute(&mut self, instruction: &isa::Instruction) -> Result<(), CpuError> {
        match instruction.mnemonic {
            isa::InstructionMnemonic::Mov => {
                todo!("execute mov")
            }
        }
    }

    fn fetch_instruction_buffer(&self) -> Result<&[u8], CpuError> {
        let ip = self.bus.read_u16(&isa::Register::IP.region()).expect("register read failed");
        let cs = self.bus.read_u16(&isa::Register::CS.region()).expect("register read failed");
        let address = ((cs as usize) << 4) + (ip as usize);
        self.bus.read(&Region { address, size: 6 }).map_err(CpuError::from)
    }

    pub fn load_program(&mut self, bytes: &[u8], segment: u16) {
        let address = (segment as usize) << 4;
        self.bus
            .write(
                bytes,
                &Region {
                    address,
                    size: bytes.len(),
                },
            )
            .expect("load failed");
        self.bus
            .write_u16(segment, &isa::Register::CS.region())
            .expect("register write failed");
        self.bus
            .write_u16(0, &isa::Register::IP.region())
            .expect("register write failed");
        self.bus
            .write_u16(segment, &isa::Register::SS.region())
            .expect("register write failed");
        self.bus
            .write_u16(0x0400, &isa::Register::SP.region())
            .expect("register write failed");
        self.bus
            .write_u16(segment, &isa::Register::DS.region())
            .expect("register write failed");
    }
}
