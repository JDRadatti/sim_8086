use crate::memory::{REGISTER_BASE, Region};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Index, IndexMut};

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
    CS,
    DS,
    SS,
    ES,
    IP,
    FLAGS,
}

#[rustfmt::skip]
impl Register {
    pub fn region(self) -> Region {
        let (offset, size) = match self {
            Register::AL => (0, 1),
            Register::AX => (0, 2),
            Register::AH => (1, 1),
            Register::CL => (2, 1),
            Register::CX => (2, 2),
            Register::CH => (3, 1),
            Register::DL => (4, 1),
            Register::DX => (4, 2),
            Register::DH => (5, 1),
            Register::BL => (6, 1),
            Register::BX => (6, 2),
            Register::BH => (7, 1),
            Register::SP => (8, 2),
            Register::BP => (10, 2),
            Register::SI => (12, 2),
            Register::DI => (14, 2),
            Register::CS => (16, 2),
            Register::DS => (18, 2),
            Register::SS => (20, 2),
            Register::ES => (22, 2),
            Register::IP => (24, 2),
            Register::FLAGS => (26, 2),
        };
        Region { address: REGISTER_BASE + offset, size }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::AL => write!(f, "al"),
            Register::AX => write!(f, "ax"),
            Register::AH => write!(f, "ah"),
            Register::CL => write!(f, "cl"),
            Register::CX => write!(f, "cx"),
            Register::CH => write!(f, "ch"),
            Register::DL => write!(f, "dl"),
            Register::DX => write!(f, "dx"),
            Register::DH => write!(f, "dh"),
            Register::BL => write!(f, "bl"),
            Register::BX => write!(f, "bx"),
            Register::BH => write!(f, "bh"),
            Register::SP => write!(f, "sp"),
            Register::BP => write!(f, "bp"),
            Register::SI => write!(f, "si"),
            Register::DI => write!(f, "di"),
            Register::CS => write!(f, "cs"),
            Register::DS => write!(f, "ds"),
            Register::SS => write!(f, "ss"),
            Register::ES => write!(f, "es"),
            Register::IP => write!(f, "ip"),
            Register::FLAGS => write!(f, "flags"),
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
// Instruction
// --------------------------------------------------------------------------------

pub enum Operand {
    Register(Register),
    DirectAddress(Region),
    EffectiveAddress(Register, i16, usize),
    EffectiveAddress2(Register, Register, i16, usize),
    Immediate(u16),
    RelativeOffset(i8),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(register) => write!(f, "{register}"),
            Operand::DirectAddress(region) => write!(f, "[{}]", region.address),
            Operand::EffectiveAddress(register1, displacement, _) => match displacement.cmp(&0) {
                Ordering::Less => write!(f, "[{register1} - {}]", displacement.unsigned_abs()),
                Ordering::Greater => write!(f, "[{register1} + {displacement}]"),
                Ordering::Equal => write!(f, "[{register1}]"),
            },
            Operand::EffectiveAddress2(register1, register2, displacement, _) => {
                match displacement.cmp(&0) {
                    Ordering::Less => write!(
                        f,
                        "[{register1} + {register2} - {}]",
                        displacement.unsigned_abs()
                    ),
                    Ordering::Greater => write!(f, "[{register1} + {register2} + {displacement}]"),
                    Ordering::Equal => write!(f, "[{register1} + {register2}]"),
                }
            }
            Operand::Immediate(imm) => write!(f, "{imm}"),
            Operand::RelativeOffset(disp) => {
                let target = 2i16 + *disp as i16;
                if target > 0 {
                    write!(f, "$+{target}")
                } else if target == 0 {
                    write!(f, "$")
                } else {
                    write!(f, "${target}")
                }
            }
        }
    }
}

impl Operand {
    fn size(&self) -> Option<usize> {
        match self {
            Operand::DirectAddress(region) => Some(region.size),
            Operand::EffectiveAddress(_, _, size) => Some(*size),
            Operand::EffectiveAddress2(_, _, _, size) => Some(*size),
            _ => None,
        }
    }
}

pub struct Operands {
    pub op1: Option<Operand>,
    pub op2: Option<Operand>,
}

impl Operands {
    fn new(op1: Option<Operand>, op2: Option<Operand>) -> Self {
        Self { op1, op2 }
    }
}

impl fmt::Display for Operands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let has_register = self
            .op1
            .as_ref()
            .map_or(false, |op| matches!(op, Operand::Register(_)))
            || self
                .op2
                .as_ref()
                .map_or(false, |op| matches!(op, Operand::Register(_)));
        let has_offset = self
            .op1
            .as_ref()
            .map_or(false, |op| matches!(op, Operand::RelativeOffset(_)));
        if !has_register && !has_offset {
            let size = self.op1.as_ref().and_then(|op| op.size()).unwrap_or(1);
            let size_str = if size == 2 { "word" } else { "byte" };
            write!(f, "{size_str} ")?;
        }
        if let Some(op1) = &self.op1 {
            write!(f, "{op1}")?;
        }
        if let Some(op2) = &self.op2 {
            write!(f, ", {op2}")?;
        }
        Ok(())
    }
}

pub enum OperandForm {
    None,
    RegMemToFromReg,
    ImmToRegMem,
    RegImm,
    MemToAcc,
    AccToMem,
    SegToRegMem,
    RegMemToSeg,
    ImmToAcc,
    JmpRel8,
}

#[derive(Copy, Clone)]
pub enum JumpCondition {
    Jo, Jno, Jb, Jae, Je, Jne, Jbe, Ja,
    Js, Jns, Jp, Jnp, Jl, Jge, Jle, Jg,
}

#[derive(Copy, Clone)]
pub enum InstructionMnemonic {
    Mov,
    Add,
    Sub,
    Cmp,
    Jcc(JumpCondition),
    Hlt,
}

impl fmt::Display for InstructionMnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionMnemonic::Mov => write!(f, "mov"),
            InstructionMnemonic::Add => write!(f, "add"),
            InstructionMnemonic::Hlt => write!(f, "hlt"),
            InstructionMnemonic::Sub => write!(f, "sub"),
            InstructionMnemonic::Cmp => write!(f, "cmp"),
            InstructionMnemonic::Jcc(cond) => match cond {
                JumpCondition::Jo => write!(f, "jo"),
                JumpCondition::Jno => write!(f, "jno"),
                JumpCondition::Jb => write!(f, "jb"),
                JumpCondition::Jae => write!(f, "jae"),
                JumpCondition::Je => write!(f, "je"),
                JumpCondition::Jne => write!(f, "jne"),
                JumpCondition::Jbe => write!(f, "jbe"),
                JumpCondition::Ja => write!(f, "ja"),
                JumpCondition::Js => write!(f, "js"),
                JumpCondition::Jns => write!(f, "jns"),
                JumpCondition::Jp => write!(f, "jp"),
                JumpCondition::Jnp => write!(f, "jnp"),
                JumpCondition::Jl => write!(f, "jl"),
                JumpCondition::Jge => write!(f, "jge"),
                JumpCondition::Jle => write!(f, "jle"),
                JumpCondition::Jg => write!(f, "jg"),
            },
        }
    }
}

pub struct Instruction {
    pub mnemonic: InstructionMnemonic,
    pub operands: Operands,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mnemonic)?;
        if self.operands.op1.is_some() || self.operands.op2.is_some() {
            write!(f, " {}", self.operands)?;
        }
        Ok(())
    }
}

impl Instruction {
    fn from(mnemonic: InstructionMnemonic, operands: Operands) -> Self {
        Self { mnemonic, operands }
    }
}

// --------------------------------------------------------------------------------
// Parser
// --------------------------------------------------------------------------------

#[derive(Copy, Clone)]
#[repr(usize)]
enum Identifier {
    Literal,
    Opcode,
    Mod,
    D,
    W,
    S,
    Rm,
    DispLo,
    DispHi,
    Reg,
    DataLo,
    DataHi,
    AddrLo,
    AddrHi,
    Sr,
}

type ParserFlag = u8;
const FLAG_DISP8: ParserFlag = 0b1;
const FLAG_DISP16: ParserFlag = 0b10;
const FLAG_W16: ParserFlag = 0b100;

struct Parser {
    identifier: Identifier,
    literal: Option<u8>,
    flag: Option<ParserFlag>,
    bit_count: usize,
}

impl Parser {
    const fn new(
        identifier: Identifier,
        literal: Option<u8>,
        flag: Option<ParserFlag>,
        bit_count: usize,
    ) -> Self {
        Parser {
            identifier,
            literal,
            flag,
            bit_count,
        }
    }

    fn parse(&self, context: &ParseContext) -> Option<u8> {
        if context.bit_offset + self.bit_count > 8 {
            return None;
        }
        let bits_remaining = 8 - (context.bit_offset + self.bit_count);
        let value = (context.current_byte() >> bits_remaining) & self.mask();
        if let Some(literal) = self.literal
            && literal != value
        {
            return None;
        }
        Some(value)
    }

    fn mask(&self) -> u8 {
        match self.bit_count {
            1 => 0b1,
            2 => 0b11,
            3 => 0b111,
            4 => 0b1111,
            5 => 0b11111,
            6 => 0b111111,
            7 => 0b1111111,
            8 => 0b11111111,
            _ => panic!("invalid bit count"),
        }
    }
}

struct ParseContext<'a> {
    bytes: &'a [u8],
    byte_index: usize,
    bit_offset: usize,
    values: [Option<u8>; 15], // Size MUST match number of variants of Identifier Enum
    flags: u8,
}

impl<'a> ParseContext<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        ParseContext {
            bytes: bytes,
            byte_index: 0,
            bit_offset: 0,
            flags: 0,
            values: [None; 15],
        }
    }

    fn advance(&mut self, bit_count: usize) {
        self.bit_offset += bit_count;
        self.byte_index += self.bit_offset / 8;
        self.bit_offset %= 8;
    }

    fn current_byte(&self) -> u8 {
        self.bytes[self.byte_index]
    }

    fn set_flag(&mut self, flag: ParserFlag) {
        self.flags |= flag;
    }

    fn check_flag(&self, flag: ParserFlag) -> bool {
        (self.flags & flag) != 0
    }

    fn update_flags(&mut self) {
        let mod_val = self[Identifier::Mod];
        let rm_val = self[Identifier::Rm];
        match mod_val {
            Some(0b01) => self.set_flag(FLAG_DISP8),
            Some(0b10) => self.set_flag(FLAG_DISP16),
            Some(0b00) if rm_val == Some(0b110) => self.set_flag(FLAG_DISP16),
            _ => {}
        }
        if self[Identifier::W] == Some(1) && self[Identifier::S] != Some(1) {
            self.set_flag(FLAG_W16);
        }
    }
}

impl Index<Identifier> for ParseContext<'_> {
    type Output = Option<u8>;
    fn index(&self, id: Identifier) -> &Self::Output {
        &self.values[id as usize]
    }
}

impl IndexMut<Identifier> for ParseContext<'_> {
    fn index_mut(&mut self, id: Identifier) -> &mut Self::Output {
        &mut self.values[id as usize]
    }
}

// --------------------------------------------------------------------------------
// Reusable Parsers
// --------------------------------------------------------------------------------

const PARSER_D: Parser = Parser::new(Identifier::D, None, None, 1);
const PARSER_W: Parser = Parser::new(Identifier::W, None, None, 1);
const PARSER_S: Parser = Parser::new(Identifier::S, None, None, 1);
const PARSER_MOD: Parser = Parser::new(Identifier::Mod, None, None, 2);
const PARSER_REG: Parser = Parser::new(Identifier::Reg, None, None, 3);
const PARSER_RM: Parser = Parser::new(Identifier::Rm, None, None, 3);
const PARSER_DISPLO: Parser =
    Parser::new(Identifier::DispLo, None, Some(FLAG_DISP8 | FLAG_DISP16), 8);
const PARSER_DISPHI: Parser = Parser::new(Identifier::DispHi, None, Some(FLAG_DISP16), 8);
const PARSER_DATA_LO: Parser = Parser::new(Identifier::DataLo, None, None, 8);
const PARSER_DATA_HI: Parser = Parser::new(Identifier::DataHi, None, Some(FLAG_W16), 8);
const PARSER_ADDR_LO: Parser = Parser::new(Identifier::AddrLo, None, None, 8);
const PARSER_ADDR_HI: Parser = Parser::new(Identifier::AddrHi, None, None, 8);
const PARSER_SR: Parser = Parser::new(Identifier::Sr, None, None, 2);

const fn opcode(opcode: u8, size: usize) -> Parser {
    Parser::new(Identifier::Opcode, Some(opcode), None, size)
}

const fn literal(literal: u8, size: usize) -> Parser {
    Parser::new(Identifier::Literal, Some(literal), None, size)
}

// --------------------------------------------------------------------------------
// Instruction Encodings
// --------------------------------------------------------------------------------

struct Encoding {
    form: OperandForm,
    mnemonic: InstructionMnemonic,
    parsers: &'static [Parser],
}

// MOV Instructions

const MOV_REG_MEM: Encoding = Encoding {
    form: OperandForm::RegMemToFromReg,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b100010, 6),
        PARSER_D,
        PARSER_W,
        PARSER_MOD,
        PARSER_REG,
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
    ],
};

const MOV_IMM_REGMEM: Encoding = Encoding {
    form: OperandForm::ImmToRegMem,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b1100011, 7),
        PARSER_W,
        PARSER_MOD,
        literal(0b000, 3),
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

const MOV_REG_IMM: Encoding = Encoding {
    form: OperandForm::RegImm,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b1011, 4),
        PARSER_W,
        PARSER_REG,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

const MOV_MEM_ACC: Encoding = Encoding {
    form: OperandForm::MemToAcc,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b1010000, 7),
        PARSER_W,
        PARSER_ADDR_LO,
        PARSER_ADDR_HI,
    ],
};

const MOV_ACC_MEM: Encoding = Encoding {
    form: OperandForm::AccToMem,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b1010001, 7),
        PARSER_W,
        PARSER_ADDR_LO,
        PARSER_ADDR_HI,
    ],
};

const SEG_TO_REGMEM: Encoding = Encoding {
    form: OperandForm::RegMemToSeg,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b10001110, 8),
        PARSER_MOD,
        literal(0b0, 1),
        PARSER_SR,
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
    ],
};

const REGMEM_TO_SEG: Encoding = Encoding {
    form: OperandForm::SegToRegMem,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[
        opcode(0b10001100, 8),
        PARSER_MOD,
        literal(0b0, 1),
        PARSER_SR,
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
    ],
};

// ADD Instructions

const ADD_REGMEM_REG: Encoding = Encoding {
    form: OperandForm::RegMemToFromReg,
    mnemonic: InstructionMnemonic::Add,
    parsers: &[
        opcode(0b000000, 6),
        PARSER_D,
        PARSER_W,
        PARSER_MOD,
        PARSER_REG,
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
    ],
};

const ADD_IMM_REGMEM: Encoding = Encoding {
    form: OperandForm::ImmToRegMem,
    mnemonic: InstructionMnemonic::Add,
    parsers: &[
        opcode(0b100000, 6),
        PARSER_S,
        PARSER_W,
        PARSER_MOD,
        literal(0b000, 3),
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

const ADD_IMM_ACC: Encoding = Encoding {
    form: OperandForm::ImmToAcc,
    mnemonic: InstructionMnemonic::Add,
    parsers: &[
        opcode(0b0000010, 7),
        PARSER_W,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

// SUB Instructions

const SUB_REGMEM_REG: Encoding = Encoding {
    form: OperandForm::RegMemToFromReg,
    mnemonic: InstructionMnemonic::Sub,
    parsers: &[
        opcode(0b001010, 6),
        PARSER_D,
        PARSER_W,
        PARSER_MOD,
        PARSER_REG,
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
    ],
};

const SUB_IMM_REGMEM: Encoding = Encoding {
    form: OperandForm::ImmToRegMem,
    mnemonic: InstructionMnemonic::Sub,
    parsers: &[
        opcode(0b100000, 6),
        PARSER_S,
        PARSER_W,
        PARSER_MOD,
        literal(0b101, 3),
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

const SUB_IMM_ACC: Encoding = Encoding {
    form: OperandForm::ImmToAcc,
    mnemonic: InstructionMnemonic::Sub,
    parsers: &[
        opcode(0b0010110, 7),
        PARSER_W,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

// CMP Instructions

const CMP_REGMEM_REG: Encoding = Encoding {
    form: OperandForm::RegMemToFromReg,
    mnemonic: InstructionMnemonic::Cmp,
    parsers: &[
        opcode(0b001110, 6),
        PARSER_D,
        PARSER_W,
        PARSER_MOD,
        PARSER_REG,
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
    ],
};

const CMP_IMM_REGMEM: Encoding = Encoding {
    form: OperandForm::ImmToRegMem,
    mnemonic: InstructionMnemonic::Cmp,
    parsers: &[
        opcode(0b100000, 6),
        PARSER_S,
        PARSER_W,
        PARSER_MOD,
        literal(0b111, 3),
        PARSER_RM,
        PARSER_DISPLO,
        PARSER_DISPHI,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

const CMP_IMM_ACC: Encoding = Encoding {
    form: OperandForm::ImmToAcc,
    mnemonic: InstructionMnemonic::Cmp,
    parsers: &[
        opcode(0b0011110, 7),
        PARSER_W,
        PARSER_DATA_LO,
        PARSER_DATA_HI,
    ],
};

// Conditional Jump Instructions

macro_rules! jump_encoding {
    ($opcode:expr, $cond:ident) => {
        Encoding {
            form: OperandForm::JmpRel8,
            mnemonic: InstructionMnemonic::Jcc(JumpCondition::$cond),
            parsers: &[opcode($opcode, 8), PARSER_DATA_LO],
        }
    };
}

const JO_INST: Encoding = jump_encoding!(0b01110000, Jo);
const JNO_INST: Encoding = jump_encoding!(0b01110001, Jno);
const JB_INST: Encoding = jump_encoding!(0b01110010, Jb);
const JAE_INST: Encoding = jump_encoding!(0b01110011, Jae);
const JE_INST: Encoding = jump_encoding!(0b01110100, Je);
const JNE_INST: Encoding = jump_encoding!(0b01110101, Jne);
const JBE_INST: Encoding = jump_encoding!(0b01110110, Jbe);
const JA_INST: Encoding = jump_encoding!(0b01110111, Ja);
const JS_INST: Encoding = jump_encoding!(0b01111000, Js);
const JNS_INST: Encoding = jump_encoding!(0b01111001, Jns);
const JP_INST: Encoding = jump_encoding!(0b01111010, Jp);
const JNP_INST: Encoding = jump_encoding!(0b01111011, Jnp);
const JL_INST: Encoding = jump_encoding!(0b01111100, Jl);
const JGE_INST: Encoding = jump_encoding!(0b01111101, Jge);
const JLE_INST: Encoding = jump_encoding!(0b01111110, Jle);
const JG_INST: Encoding = jump_encoding!(0b01111111, Jg);

// HLT Instruction

const HLT_INST: Encoding = Encoding {
    form: OperandForm::None,
    mnemonic: InstructionMnemonic::Hlt,
    parsers: &[opcode(0b11110100, 8)],
};

const INSTRUCTION_TABLE: &[Encoding] = &[
    MOV_REG_MEM,
    MOV_IMM_REGMEM,
    MOV_REG_IMM,
    MOV_MEM_ACC,
    MOV_ACC_MEM,
    SEG_TO_REGMEM,
    REGMEM_TO_SEG,
    HLT_INST,
    ADD_IMM_REGMEM,
    ADD_REGMEM_REG,
    ADD_IMM_ACC,
    SUB_IMM_REGMEM,
    SUB_REGMEM_REG,
    SUB_IMM_ACC,
    CMP_REGMEM_REG,
    CMP_IMM_REGMEM,
    CMP_IMM_ACC,
    JO_INST,
    JNO_INST,
    JB_INST,
    JAE_INST,
    JE_INST,
    JNE_INST,
    JBE_INST,
    JA_INST,
    JS_INST,
    JNS_INST,
    JP_INST,
    JNP_INST,
    JL_INST,
    JGE_INST,
    JLE_INST,
    JG_INST,
];

// --------------------------------------------------------------------------------
// Decode
// --------------------------------------------------------------------------------

pub fn decode(bytes: &[u8]) -> Option<(Instruction, usize)> {
    for encoding in INSTRUCTION_TABLE {
        if let Some((instruction, bytes_consumed)) = try_decode(encoding, bytes) {
            return Some((instruction, bytes_consumed));
        }
    }
    None
}

fn try_decode(encoding: &Encoding, bytes: &[u8]) -> Option<(Instruction, usize)> {
    let mut context = ParseContext::new(bytes);
    for parser in encoding.parsers {
        if let Some(flag) = parser.flag
            && !context.check_flag(flag)
        {
            continue;
        }
        if let Some(value) = parser.parse(&context) {
            context[parser.identifier] = Some(value);
        } else {
            return None;
        }
        context.advance(parser.bit_count);
        context.update_flags();
    }

    let operands = decode_operands(&context, &encoding.form);
    let instruction = Instruction::from(encoding.mnemonic, operands);
    Some((instruction, context.byte_index))
}

fn decode_operands(context: &ParseContext, form: &OperandForm) -> Operands {
    match form {
        OperandForm::None => Operands::new(None, None),

        OperandForm::RegMemToFromReg => {
            let d = context[Identifier::D].unwrap();
            let reg_op = Operand::Register(reg_register(context));
            let rm_op = rm_field_encoding(context);

            if d == 1 {
                Operands::new(Some(reg_op), Some(rm_op))
            } else {
                Operands::new(Some(rm_op), Some(reg_op))
            }
        }

        OperandForm::ImmToRegMem => {
            let w = context[Identifier::W].unwrap();
            let s = context[Identifier::S].unwrap_or(0);
            let imm = if s == 1 && w == 1 {
                context[Identifier::DataLo].unwrap() as i8 as i16 as u16
            } else {
                combine_bytes_u16(context[Identifier::DataLo], context[Identifier::DataHi])
            };
            let rm_op = rm_field_encoding(context);
            Operands::new(Some(rm_op), Some(Operand::Immediate(imm)))
        }

        OperandForm::RegImm => {
            let reg = reg_register(&context);
            let imm = combine_bytes_u16(context[Identifier::DataLo], context[Identifier::DataHi]);
            Operands::new(Some(Operand::Register(reg)), Some(Operand::Immediate(imm)))
        }

        OperandForm::AccToMem => {
            let address =
                combine_bytes_u16(context[Identifier::AddrLo], context[Identifier::AddrHi]) as u16
                    as usize;
            let w = context[Identifier::W].unwrap();
            let acc = if w == 0 { Register::AL } else { Register::AX };
            let region = Region {
                address,
                size: 1 + w as usize,
            };
            Operands::new(
                Some(Operand::DirectAddress(region)),
                Some(Operand::Register(acc)),
            )
        }
        OperandForm::MemToAcc => {
            let address =
                combine_bytes_u16(context[Identifier::AddrLo], context[Identifier::AddrHi]) as u16
                    as usize;
            let w = context[Identifier::W].unwrap();
            let acc = if w == 0 { Register::AL } else { Register::AX };
            let region = Region {
                address,
                size: 1 + w as usize,
            };
            Operands::new(
                Some(Operand::Register(acc)),
                Some(Operand::DirectAddress(region)),
            )
        }

        OperandForm::SegToRegMem => {
            let seg_op = seg_field_encoding(context);
            let rm_op = rm_field_encoding(context);
            Operands::new(Some(rm_op), Some(seg_op))
        }
        OperandForm::RegMemToSeg => {
            let seg_op = seg_field_encoding(context);
            let rm_op = rm_field_encoding(context);
            Operands::new(Some(seg_op), Some(rm_op))
        }

        OperandForm::ImmToAcc => {
            let w = context[Identifier::W].unwrap();
            let acc = if w == 0 { Register::AL } else { Register::AX };
            let imm = combine_bytes_u16(context[Identifier::DataLo], context[Identifier::DataHi]);
            Operands::new(Some(Operand::Register(acc)), Some(Operand::Immediate(imm)))
        }

        OperandForm::JmpRel8 => {
            let disp = context[Identifier::DataLo].unwrap() as i8;
            Operands::new(Some(Operand::RelativeOffset(disp)), None)
        }
    }
}

// Table 4-11; Row SR
fn seg_field_encoding(context: &ParseContext) -> Operand {
    match context[Identifier::Sr].unwrap() {
        0b00 => Operand::Register(Register::ES),
        0b01 => Operand::Register(Register::CS),
        0b10 => Operand::Register(Register::SS),
        0b11 => Operand::Register(Register::DS),
        _ => unreachable!(),
    }
}

// Table 4-10
fn rm_field_encoding(context: &ParseContext) -> Operand {
    let mod_val = context[Identifier::Mod].unwrap();
    let rm = context[Identifier::Rm].unwrap();
    let w = context[Identifier::W].unwrap_or(1);
    let size = 1 + w as usize;
    let disp_lo = context[Identifier::DispLo];
    let disp_hi = context[Identifier::DispHi];
    let displacement = match mod_val {
        0b01 => combine_bytes_i16(disp_lo, None) as i8 as i16,
        0b10 => combine_bytes_i16(disp_lo, disp_hi),
        _ => 0,
    };
    match (mod_val, rm) {
        (0b11, _) => Operand::Register(decode_register(rm, w)),

        (0b00, 0b110) => Operand::DirectAddress(Region {
            address: combine_bytes_i16(disp_lo, disp_hi) as usize,
            size: size,
        }),

        (_, 0b000) => Operand::EffectiveAddress2(Register::BX, Register::SI, displacement, size),
        (_, 0b001) => Operand::EffectiveAddress2(Register::BX, Register::DI, displacement, size),
        (_, 0b010) => Operand::EffectiveAddress2(Register::BP, Register::SI, displacement, size),
        (_, 0b011) => Operand::EffectiveAddress2(Register::BP, Register::DI, displacement, size),
        (_, 0b100) => Operand::EffectiveAddress(Register::SI, displacement, size),
        (_, 0b101) => Operand::EffectiveAddress(Register::DI, displacement, size),
        (_, 0b110) => Operand::EffectiveAddress(Register::BP, displacement, size),
        (_, 0b111) => Operand::EffectiveAddress(Register::BX, displacement, size),

        _ => todo!("effective address for rm={rm}, mod={mod_val}"),
    }
}

fn decode_register(reg: u8, w: u8) -> Register {
    REG_ENCODING[(reg << 1 | w) as usize]
}

fn reg_register(context: &ParseContext) -> Register {
    let reg = context[Identifier::Reg].unwrap();
    let w = context[Identifier::W].unwrap();
    decode_register(reg, w)
}

fn combine_bytes_u16(lo: Option<u8>, hi: Option<u8>) -> u16 {
    lo.unwrap_or(0) as u16 | (hi.unwrap_or(0) as u16) << 8
}

fn combine_bytes_i16(lo: Option<u8>, hi: Option<u8>) -> i16 {
    combine_bytes_u16(lo, hi) as i16
}
