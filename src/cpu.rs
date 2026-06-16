use crate::isa;
use crate::memory::{Bus, MemoryError, Region};
use std::fmt;

// --------------------------------------------------------------------------------
// CPU errors
// --------------------------------------------------------------------------------

#[derive(Debug)]
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

            if matches!(instruction.mnemonic, isa::InstructionMnemonic::Hlt) {
                break;
            }
        }
        Ok(())
    }

    pub fn run_execute(&mut self) -> Result<(), CpuError> {
        loop {
            let instruction = self.step()?;
            self.execute(&instruction)?;
        }
    }

    pub fn step(&mut self) -> Result<isa::Instruction, CpuError> {
        let ip = self
            .bus
            .read_u16(&isa::Register::IP.region())
            .expect("register read failed");
        let buf = self.fetch_instruction_buffer()?;
        let (instruction, consumed) = isa::decode(buf).ok_or(CpuError::InvalidOpcode(ip))?;

        let next_ip = ip + consumed as u16;
        self.bus
            .write_u16(next_ip, &isa::Register::IP.region())
            .expect("register write failed");
        Ok(instruction)
    }

    pub fn step_execute(&mut self) -> Result<(), CpuError> {
        let instruction = self.step()?;
        self.execute(&instruction)
    }

    fn execute(&mut self, instruction: &isa::Instruction) -> Result<(), CpuError> {
        match instruction.mnemonic {
            isa::InstructionMnemonic::Mov => {
                let src = instruction.operands.op2.as_ref().unwrap();
                let dst = instruction.operands.op1.as_ref().unwrap();
                let value = self.read_operand(src)?;
                self.write_operand(dst, value)?;
            }
            isa::InstructionMnemonic::Hlt => return Err(CpuError::Halted),
        }
        Ok(())
    }

    pub fn read_register(&self, reg: isa::Register) -> u16 {
        self.bus
            .read_u16(&reg.region())
            .expect("register read failed")
    }

    pub fn read_memory(&self, address: usize, size: usize) -> &[u8] {
        self.bus
            .read(&Region { address, size })
            .expect("memory read failed")
    }

    fn read_operand(&self, operand: &isa::Operand) -> Result<u16, CpuError> {
        match operand {
            isa::Operand::Register(reg) => self.bus.read_u16(&reg.region()).map_err(CpuError::from),
            isa::Operand::Immediate(val) => Ok(*val),
            _ => {
                let region = self.operand_region(operand).unwrap();
                self.bus.read_u16(&region).map_err(CpuError::from)
            }
        }
    }

    fn write_operand(&mut self, operand: &isa::Operand, value: u16) -> Result<(), CpuError> {
        match operand {
            isa::Operand::Register(reg) => self
                .bus
                .write_u16(value, &reg.region())
                .map_err(CpuError::from),
            isa::Operand::Immediate(_) => {
                panic!("cannot write to immediate operand");
            }
            _ => {
                let region = self.operand_region(operand).unwrap();
                self.bus.write_u16(value, &region).map_err(CpuError::from)
            }
        }
    }

    fn operand_region(&self, operand: &isa::Operand) -> Option<Region> {
        match operand {
            isa::Operand::DirectAddress(region) => {
                let linear = self.compute_linear_address(region.address as u16, isa::Register::DS);
                Some(Region {
                    address: linear,
                    size: region.size,
                })
            }
            isa::Operand::EffectiveAddress(base_reg, displacement, size) => {
                let offset = self.effective_address_offset(*base_reg, *displacement);
                let segment = self.effective_address_segment(*base_reg);
                let linear = self.compute_linear_address(offset, segment);
                Some(Region {
                    address: linear,
                    size: *size,
                })
            }
            isa::Operand::EffectiveAddress2(reg1, reg2, displacement, size) => {
                let offset = self.effective_address_offset2(*reg1, *reg2, *displacement);
                let segment = self.effective_address_segment2(*reg1, *reg2);
                let linear = self.compute_linear_address(offset, segment);
                Some(Region {
                    address: linear,
                    size: *size,
                })
            }
            _ => None,
        }
    }

    fn effective_address_offset(&self, base: isa::Register, displacement: i16) -> u16 {
        let base_val = self.bus.read_u16(&base.region()).unwrap();
        ((base_val as i32) + (displacement as i32)) as u16
    }

    fn effective_address_offset2(
        &self,
        reg1: isa::Register,
        reg2: isa::Register,
        displacement: i16,
    ) -> u16 {
        let val1 = self.bus.read_u16(&reg1.region()).unwrap();
        let val2 = self.bus.read_u16(&reg2.region()).unwrap();
        ((val1 as i32) + (val2 as i32) + (displacement as i32)) as u16
    }

    fn effective_address_segment(&self, base: isa::Register) -> isa::Register {
        if matches!(base, isa::Register::BP) {
            isa::Register::SS
        } else {
            isa::Register::DS
        }
    }

    fn effective_address_segment2(
        &self,
        reg1: isa::Register,
        reg2: isa::Register,
    ) -> isa::Register {
        if matches!(reg1, isa::Register::BP) || matches!(reg2, isa::Register::BP) {
            isa::Register::SS
        } else {
            isa::Register::DS
        }
    }

    fn compute_linear_address(&self, offset: u16, segment: isa::Register) -> usize {
        let seg_val = self.bus.read_u16(&segment.region()).unwrap();
        ((seg_val as usize) << 4) + (offset as usize)
    }

    fn fetch_instruction_buffer(&self) -> Result<&[u8], CpuError> {
        let ip = self
            .bus
            .read_u16(&isa::Register::IP.region())
            .expect("register read failed");
        let cs = self
            .bus
            .read_u16(&isa::Register::CS.region())
            .expect("register read failed");
        let address = ((cs as usize) << 4) + (ip as usize);
        self.bus
            .read(&Region { address, size: 6 })
            .map_err(CpuError::from)
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
