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
// FLAGS
// --------------------------------------------------------------------------------

type CpuFlag = u16;
const FLAG_CF: CpuFlag = 0b1;
const FLAG_PF: CpuFlag = 0b100;
const FLAG_AF: CpuFlag = 0b10000;
const FLAG_ZF: CpuFlag = 0b1000000;
const FLAG_SF: CpuFlag = 0b10000000;
const FLAG_TF: CpuFlag = 0b100000000;
const FLAG_IF: CpuFlag = 0b1000000000;
#[allow(dead_code)]
const FLAG_DF: CpuFlag = 0b10000000000;
const FLAG_OF: CpuFlag = 0b100000000000;

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
            isa::InstructionMnemonic::Add => {
                let src = instruction.operands.op2.as_ref().unwrap();
                let dst = instruction.operands.op1.as_ref().unwrap();
                let is_word = self.operand_is_word(dst);
                let mask: u16 = if is_word { 0xFFFF } else { 0x00FF };
                let src_val = self.read_operand(src)? & mask;
                let dst_val = self.read_operand(dst)? & mask;
                let (raw_result, word_overflow) = dst_val.overflowing_add(src_val);
                let carry = if is_word {
                    word_overflow
                } else {
                    raw_result > mask
                };
                let result = raw_result & mask;
                self.write_operand(dst, result)?;
                self.set_add_flags(result, src_val, dst_val, carry, is_word);
            }
            isa::InstructionMnemonic::Sub => {
                let src = instruction.operands.op2.as_ref().unwrap();
                let dst = instruction.operands.op1.as_ref().unwrap();
                let is_word = self.operand_is_word(dst);
                let mask: u16 = if is_word { 0xFFFF } else { 0x00FF };
                let src_val = self.read_operand(src)? & mask;
                let dst_val = self.read_operand(dst)? & mask;
                let (raw_result, word_borrow) = dst_val.overflowing_sub(src_val);
                let borrow = if is_word {
                    word_borrow
                } else {
                    dst_val < src_val
                };
                let result = raw_result & mask;
                self.write_operand(dst, result)?;
                self.set_sub_flags(result, src_val, dst_val, borrow, is_word);
            }
            isa::InstructionMnemonic::Cmp => {
                let src = instruction.operands.op2.as_ref().unwrap();
                let dst = instruction.operands.op1.as_ref().unwrap();
                let is_word = self.operand_is_word(dst);
                let mask: u16 = if is_word { 0xFFFF } else { 0x00FF };
                let src_val = self.read_operand(src)? & mask;
                let dst_val = self.read_operand(dst)? & mask;
                let (raw_result, word_borrow) = dst_val.overflowing_sub(src_val);
                let borrow = if is_word {
                    word_borrow
                } else {
                    dst_val < src_val
                };
                let result = raw_result & mask;
                // CMP sets flags like SUB but does NOT write the result
                self.set_sub_flags(result, src_val, dst_val, borrow, is_word);
            }
            isa::InstructionMnemonic::Jcc(cond) => {
                let disp = match instruction.operands.op1.as_ref().unwrap() {
                    isa::Operand::RelativeOffset(d) => *d as i16,
                    _ => unreachable!(),
                };
                let flags = self.read_register(isa::Register::FLAGS);
                let take = match cond {
                    isa::JumpCondition::Jo => flags & FLAG_OF != 0,
                    isa::JumpCondition::Jno => flags & FLAG_OF == 0,
                    isa::JumpCondition::Jb => flags & FLAG_CF != 0,
                    isa::JumpCondition::Jae => flags & FLAG_CF == 0,
                    isa::JumpCondition::Je => flags & FLAG_ZF != 0,
                    isa::JumpCondition::Jne => flags & FLAG_ZF == 0,
                    isa::JumpCondition::Jbe => flags & (FLAG_CF | FLAG_ZF) != 0,
                    isa::JumpCondition::Ja => flags & (FLAG_CF | FLAG_ZF) == 0,
                    isa::JumpCondition::Js => flags & FLAG_SF != 0,
                    isa::JumpCondition::Jns => flags & FLAG_SF == 0,
                    isa::JumpCondition::Jp => flags & FLAG_PF != 0,
                    isa::JumpCondition::Jnp => flags & FLAG_PF == 0,
                    isa::JumpCondition::Jl => flags & FLAG_SF != flags & FLAG_OF,
                    isa::JumpCondition::Jge => flags & FLAG_SF == flags & FLAG_OF,
                    isa::JumpCondition::Jle => {
                        flags & FLAG_ZF != 0 || flags & FLAG_SF != flags & FLAG_OF
                    }
                    isa::JumpCondition::Jg => {
                        flags & FLAG_ZF == 0 && flags & FLAG_SF == flags & FLAG_OF
                    }
                };
                if take {
                    let ip = self.read_register(isa::Register::IP);
                    let new_ip = ((ip as i16).wrapping_add(disp)) as u16;
                    self.bus.write_u16(new_ip, &isa::Register::IP.region())?;
                }
            }
            isa::InstructionMnemonic::Loopnz => {
                let disp = match instruction.operands.op1.as_ref().unwrap() {
                    isa::Operand::RelativeOffset(d) => *d as i16,
                    _ => unreachable!(),
                };
                let cx = self.read_register(isa::Register::CX);
                let new_cx = cx.wrapping_sub(1);
                self.bus.write_u16(new_cx, &isa::Register::CX.region())?;
                if new_cx != 0 {
                    let flags = self.read_register(isa::Register::FLAGS);
                    if flags & FLAG_ZF == 0 {
                        let ip = self.read_register(isa::Register::IP);
                        let new_ip = ((ip as i16).wrapping_add(disp)) as u16;
                        self.bus.write_u16(new_ip, &isa::Register::IP.region())?;
                    }
                }
            }
            isa::InstructionMnemonic::Loopz => {
                let disp = match instruction.operands.op1.as_ref().unwrap() {
                    isa::Operand::RelativeOffset(d) => *d as i16,
                    _ => unreachable!(),
                };
                let cx = self.read_register(isa::Register::CX);
                let new_cx = cx.wrapping_sub(1);
                self.bus.write_u16(new_cx, &isa::Register::CX.region())?;
                if new_cx != 0 {
                    let flags = self.read_register(isa::Register::FLAGS);
                    if flags & FLAG_ZF != 0 {
                        let ip = self.read_register(isa::Register::IP);
                        let new_ip = ((ip as i16).wrapping_add(disp)) as u16;
                        self.bus.write_u16(new_ip, &isa::Register::IP.region())?;
                    }
                }
            }
            isa::InstructionMnemonic::Loop => {
                let disp = match instruction.operands.op1.as_ref().unwrap() {
                    isa::Operand::RelativeOffset(d) => *d as i16,
                    _ => unreachable!(),
                };
                let cx = self.read_register(isa::Register::CX);
                let new_cx = cx.wrapping_sub(1);
                self.bus.write_u16(new_cx, &isa::Register::CX.region())?;
                if new_cx != 0 {
                    let ip = self.read_register(isa::Register::IP);
                    let new_ip = ((ip as i16).wrapping_add(disp)) as u16;
                    self.bus.write_u16(new_ip, &isa::Register::IP.region())?;
                }
            }
            isa::InstructionMnemonic::Jcxz => {
                let disp = match instruction.operands.op1.as_ref().unwrap() {
                    isa::Operand::RelativeOffset(d) => *d as i16,
                    _ => unreachable!(),
                };
                let cx = self.read_register(isa::Register::CX);
                if cx == 0 {
                    let ip = self.read_register(isa::Register::IP);
                    let new_ip = ((ip as i16).wrapping_add(disp)) as u16;
                    self.bus.write_u16(new_ip, &isa::Register::IP.region())?;
                }
            }
            isa::InstructionMnemonic::Push => {
                let src = instruction.operands.op1.as_ref().unwrap();
                let val = self.read_operand(src)?;
                let sp = self.read_register(isa::Register::SP);
                let new_sp = sp.wrapping_sub(2);
                let addr = self.compute_linear_address(new_sp, isa::Register::SS);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.bus.write_u16(
                    val,
                    &Region {
                        address: addr,
                        size: 2,
                    },
                )?;
            }
            isa::InstructionMnemonic::Pop => {
                let dst = instruction.operands.op1.as_ref().unwrap();
                let sp = self.read_register(isa::Register::SP);
                let addr = self.compute_linear_address(sp, isa::Register::SS);
                let val = self.bus.read_u16(&Region {
                    address: addr,
                    size: 2,
                })?;
                let new_sp = sp.wrapping_add(2);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.write_operand(dst, val)?;
            }
            isa::InstructionMnemonic::Call => {
                let ip = self.read_register(isa::Register::IP);
                let sp = self.read_register(isa::Register::SP);
                let new_sp = sp.wrapping_sub(2);
                let addr = self.compute_linear_address(new_sp, isa::Register::SS);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.bus.write_u16(
                    ip,
                    &Region {
                        address: addr,
                        size: 2,
                    },
                )?;
                let target = match instruction.operands.op1.as_ref().unwrap() {
                    isa::Operand::RelativeOffset16(disp) => {
                        ((ip as i32).wrapping_add(*disp as i32)) as u16
                    }
                    op => self.read_operand(op)?,
                };
                self.bus.write_u16(target, &isa::Register::IP.region())?;
            }
            isa::InstructionMnemonic::Ret => {
                let sp = self.read_register(isa::Register::SP);
                let addr = self.compute_linear_address(sp, isa::Register::SS);
                let ip = self.bus.read_u16(&Region {
                    address: addr,
                    size: 2,
                })?;
                let new_sp = sp.wrapping_add(2);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.bus.write_u16(ip, &isa::Register::IP.region())?;
            }
            isa::InstructionMnemonic::RetF => {
                let sp = self.read_register(isa::Register::SP);
                let addr = self.compute_linear_address(sp, isa::Register::SS);
                let ip = self.bus.read_u16(&Region {
                    address: addr,
                    size: 2,
                })?;
                let sp2 = sp.wrapping_add(2);
                let addr2 = self.compute_linear_address(sp2, isa::Register::SS);
                let cs = self.bus.read_u16(&Region {
                    address: addr2,
                    size: 2,
                })?;
                let new_sp = sp2.wrapping_add(2);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.bus.write_u16(ip, &isa::Register::IP.region())?;
                self.bus.write_u16(cs, &isa::Register::CS.region())?;
            }
            isa::InstructionMnemonic::RetImm => {
                let sp = self.read_register(isa::Register::SP);
                let addr = self.compute_linear_address(sp, isa::Register::SS);
                let ip = self.bus.read_u16(&Region {
                    address: addr,
                    size: 2,
                })?;
                let imm = instruction.operands.op1.as_ref().unwrap();
                let isa::Operand::Immediate(pop_val) = imm else {
                    unreachable!()
                };
                let new_sp = sp.wrapping_add(2).wrapping_add(*pop_val);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.bus.write_u16(ip, &isa::Register::IP.region())?;
            }
            isa::InstructionMnemonic::RetFImm => {
                let sp = self.read_register(isa::Register::SP);
                let addr = self.compute_linear_address(sp, isa::Register::SS);
                let ip = self.bus.read_u16(&Region {
                    address: addr,
                    size: 2,
                })?;
                let sp2 = sp.wrapping_add(2);
                let addr2 = self.compute_linear_address(sp2, isa::Register::SS);
                let cs = self.bus.read_u16(&Region {
                    address: addr2,
                    size: 2,
                })?;
                let imm = instruction.operands.op1.as_ref().unwrap();
                let isa::Operand::Immediate(pop_val) = imm else {
                    unreachable!()
                };
                let new_sp = sp2.wrapping_add(2).wrapping_add(*pop_val);
                self.bus.write_u16(new_sp, &isa::Register::SP.region())?;
                self.bus.write_u16(ip, &isa::Register::IP.region())?;
                self.bus.write_u16(cs, &isa::Register::CS.region())?;
            }
            isa::InstructionMnemonic::Hlt => return Err(CpuError::Halted),
        }
        Ok(())
    }

    //
    // Read/Write helpers
    //

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
            isa::Operand::RelativeOffset(_) | isa::Operand::RelativeOffset16(_) => {
                unreachable!()
            }
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
            isa::Operand::Immediate(_)
            | isa::Operand::RelativeOffset(_)
            | isa::Operand::RelativeOffset16(_) => {
                panic!("cannot write to immediate operand");
            }
            _ => {
                let region = self.operand_region(operand).unwrap();
                self.bus.write_u16(value, &region).map_err(CpuError::from)
            }
        }
    }

    //
    // Operand resolution
    //

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

    fn operand_is_word(&self, operand: &isa::Operand) -> bool {
        match operand {
            isa::Operand::Register(reg) => reg.region().size == 2,
            isa::Operand::DirectAddress(region) => region.size == 2,
            isa::Operand::EffectiveAddress(_, _, size) => *size == 2,
            isa::Operand::EffectiveAddress2(_, _, _, size) => *size == 2,
            isa::Operand::Immediate(_) => true,
            isa::Operand::RelativeOffset(_) | isa::Operand::RelativeOffset16(_) => {
                unreachable!()
            }
        }
    }

    //
    // Address calculation
    //

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

    //
    // Load/Fetch instructions
    //

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

    //
    // Flag helpers
    //

    fn set_add_flags(
        &mut self,
        result: u16,
        src_val: u16,
        dst_val: u16,
        carry: bool,
        is_word: bool,
    ) {
        let mask: u16 = if is_word { 0xFFFF } else { 0x00FF };
        let sign_bit: u16 = if is_word { 0x8000 } else { 0x0080 };
        let result = result & mask;

        if carry {
            self.set_flag(FLAG_CF);
        } else {
            self.clear_flag(FLAG_CF);
        }

        if result == 0 {
            self.set_flag(FLAG_ZF);
        } else {
            self.clear_flag(FLAG_ZF);
        }

        if result & sign_bit != 0 {
            self.set_flag(FLAG_SF);
        } else {
            self.clear_flag(FLAG_SF);
        }

        let src_sign = src_val & sign_bit != 0;
        let dst_sign = dst_val & sign_bit != 0;
        let res_sign = result & sign_bit != 0;
        if src_sign == dst_sign && res_sign != src_sign {
            self.set_flag(FLAG_OF);
        } else {
            self.clear_flag(FLAG_OF);
        }

        let parity = (result as u8).count_ones() % 2 == 0;
        if parity {
            self.set_flag(FLAG_PF);
        } else {
            self.clear_flag(FLAG_PF);
        }

        if (src_val ^ dst_val ^ result) & 0x10 != 0 {
            self.set_flag(FLAG_AF);
        } else {
            self.clear_flag(FLAG_AF);
        }
    }

    fn set_sub_flags(
        &mut self,
        result: u16,
        src_val: u16,
        dst_val: u16,
        borrow: bool,
        is_word: bool,
    ) {
        let mask: u16 = if is_word { 0xFFFF } else { 0x00FF };
        let sign_bit: u16 = if is_word { 0x8000 } else { 0x0080 };
        let result = result & mask;

        if borrow {
            self.set_flag(FLAG_CF);
        } else {
            self.clear_flag(FLAG_CF);
        }

        if result == 0 {
            self.set_flag(FLAG_ZF);
        } else {
            self.clear_flag(FLAG_ZF);
        }

        if result & sign_bit != 0 {
            self.set_flag(FLAG_SF);
        } else {
            self.clear_flag(FLAG_SF);
        }

        // OF for SUB: set when subtracting values of different signs
        // produces a result with the same sign as the subtrahend (src)
        let src_sign = src_val & sign_bit != 0;
        let dst_sign = dst_val & sign_bit != 0;
        let res_sign = result & sign_bit != 0;
        if src_sign != dst_sign && res_sign == src_sign {
            self.set_flag(FLAG_OF);
        } else {
            self.clear_flag(FLAG_OF);
        }

        let parity = (result as u8).count_ones() % 2 == 0;
        if parity {
            self.set_flag(FLAG_PF);
        } else {
            self.clear_flag(FLAG_PF);
        }

        if (src_val ^ dst_val ^ result) & 0x10 != 0 {
            self.set_flag(FLAG_AF);
        } else {
            self.clear_flag(FLAG_AF);
        }
    }

    fn set_flag(&mut self, flag: CpuFlag) {
        self.bus
            .or_u16(flag, &isa::Register::FLAGS.region())
            .expect("set flag failed");
    }

    fn clear_flag(&mut self, flag: CpuFlag) {
        self.bus
            .and_u16(!flag, &isa::Register::FLAGS.region())
            .expect("clear flag failed");
    }

    fn check_flag(&mut self, flag: CpuFlag) -> bool {
        let val = self
            .bus
            .read_u16(&isa::Register::FLAGS.region())
            .expect("read flag failed");
        val & flag != 0
    }
}
