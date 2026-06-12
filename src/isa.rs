use crate::memory::Region;
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
        match self {
            Register::AL => Region { address: 0, size: 1, },
            Register::AX => Region { address: 0, size: 2, },
            Register::AH => Region { address: 1, size: 1, },
            Register::CL => Region { address: 2, size: 1, },
            Register::CX => Region { address: 2, size: 2, },
            Register::CH => Region { address: 3, size: 1, },
            Register::DL => Region { address: 4, size: 1, },
            Register::DX => Region { address: 4, size: 2, },
            Register::DH => Region { address: 5, size: 1, },
            Register::BL => Region { address: 6, size: 1, },
            Register::BX => Region { address: 6, size: 2, },
            Register::BH => Region { address: 7, size: 1, },
            Register::SP => Region { address: 8, size: 2, },
            Register::BP => Region { address: 10, size: 2, },
            Register::SI => Region { address: 12, size: 2, },
            Register::DI => Region { address: 14, size: 2, },
            Register::CS => Region { address: 16, size: 2, },
            Register::DS => Region { address: 18, size: 2, },
            Register::SS => Region { address: 20, size: 2, },
            Register::ES => Region { address: 22, size: 2, },
            Register::IP => Region { address: 24, size: 2, },
            Register::FLAGS => Region { address: 26, size: 2, },
        }
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
    Memory(Region),
    DirectAddress(Region),
    EffectiveAddress(Register, i16),
    EffectiveAddress2(Register, Register, i16),
    Immediate(i16),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(register) => write!(f, "{register}"),
            Operand::Memory(_region) => todo!("display memory operand"),
            Operand::DirectAddress(region) => write!(f, "[{}]", region.address),
            Operand::EffectiveAddress(register1, displacement) => match displacement.cmp(&0) {
                Ordering::Less => write!(f, "[{register1} - {}]", displacement.unsigned_abs()),
                Ordering::Greater => write!(f, "[{register1} + {displacement}]"),
                Ordering::Equal => write!(f, "[{register1}]"),
            },
            Operand::EffectiveAddress2(register1, register2, displacement) => {
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
        }
    }
}

pub enum OperandForm {
    RegMemToFromReg,
    RegImm,
}

#[derive(Copy, Clone)]
pub enum InstructionMnemonic {
    Mov,
}

impl fmt::Display for InstructionMnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionMnemonic::Mov => write!(f, "mov"),
        }
    }
}

pub struct Instruction {
    pub mnemonic: InstructionMnemonic,
    operands: (Option<Operand>, Option<Operand>),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.mnemonic)?;
        if let Some(op1) = &self.operands.0 {
            write!(f, "{op1}")?;
        }
        if let Some(op2) = &self.operands.1 {
            write!(f, ", {op2}")?;
        }
        Ok(())
    }
}

impl Instruction {
    fn from(mnemonic: InstructionMnemonic, operands: (Option<Operand>, Option<Operand>)) -> Self {
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
    Rm,
    DispLo,
    DispHi,
    Reg,
    DataLo,
    DataHi,
    AddrLo,
    AddrHi,
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
    values: [Option<u8>; 13], // Size MUST match number of variants of Identifier Enum
    flags: u8,
}

impl<'a> ParseContext<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        ParseContext {
            bytes: bytes,
            byte_index: 0,
            bit_offset: 0,
            flags: 0,
            values: [None; 13],
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
        let mod_val = self.values[Identifier::Mod as usize];
        let rm_val = self.values[Identifier::Rm as usize];
        match mod_val {
            Some(0b01) => self.set_flag(FLAG_DISP8),
            Some(0b10) => self.set_flag(FLAG_DISP16),
            Some(0b00) if rm_val == Some(0b110) => self.set_flag(FLAG_DISP16),
            _ => {}
        }
        if self.values[Identifier::W as usize] == Some(1) {
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
const PARSER_MOD: Parser = Parser::new(Identifier::Mod, None, None, 2);
const PARSER_REG: Parser = Parser::new(Identifier::Reg, None, None, 3);
const PARSER_RM: Parser = Parser::new(Identifier::Rm, None, None, 3);
const PARSER_DISPLO: Parser =
    Parser::new(Identifier::DispLo, None, Some(FLAG_DISP8 | FLAG_DISP16), 8);
const PARSER_DISPHI: Parser = Parser::new(Identifier::DispHi, None, Some(FLAG_DISP16), 8);
const PARSER_DATA_LO: Parser = Parser::new(Identifier::DataLo, None, None, 8);
const PARSER_DATA_HI: Parser = Parser::new(Identifier::DataHi, None, Some(FLAG_W16), 8);
const ADDR_LO: Parser = Parser::new(Identifier::AddrLo, None, None, 8);
const ADDR_HI: Parser = Parser::new(Identifier::AddrHi, None, Some(FLAG_W16), 8);

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
    form: OperandForm::RegMemToFromReg,
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
    form: OperandForm::RegMemToFromReg,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[opcode(0b1010000, 7), PARSER_W, ADDR_LO, ADDR_HI],
};

const MOV_ACC_MEM: Encoding = Encoding {
    form: OperandForm::RegMemToFromReg,
    mnemonic: InstructionMnemonic::Mov,
    parsers: &[opcode(0b1010001, 7), PARSER_W, ADDR_LO, ADDR_HI],
};

const INSTRUCTION_TABLE: &[Encoding] = &[
    MOV_REG_MEM,
    MOV_IMM_REGMEM,
    MOV_REG_IMM,
    MOV_MEM_ACC,
    MOV_ACC_MEM,
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

fn decode_operands(
    context: &ParseContext,
    form: &OperandForm,
) -> (Option<Operand>, Option<Operand>) {
    match form {
        OperandForm::RegMemToFromReg => {
            let d = context.values[Identifier::D as usize].unwrap();
            let reg_op = Operand::Register(reg_register(context));
            let rm_op = rm_field_encoding(context);

            if d == 1 {
                (Some(reg_op), Some(rm_op))
            } else {
                (Some(rm_op), Some(reg_op))
            }
        }

        OperandForm::RegImm => {
            let reg = reg_register(&context);
            let imm = combine_bytes(context[Identifier::DataLo], context[Identifier::DataHi]);
            (Some(Operand::Register(reg)), Some(Operand::Immediate(imm)))
        }
    }
}

// Table 4-10
fn rm_field_encoding(context: &ParseContext) -> Operand {
    let mod_val = context.values[Identifier::Mod as usize].unwrap();
    let rm = context.values[Identifier::Rm as usize].unwrap();
    let w = context.values[Identifier::W as usize].unwrap();
    let disp_lo = context.values[Identifier::DispLo as usize];
    let disp_hi = context.values[Identifier::DispHi as usize];
    let combined = combine_bytes(disp_lo, disp_hi);
    let displacement = match mod_val {
        0b01 => combined as i8 as i16,
        0b10 => combined,
        _ => 0,
    };
    match (mod_val, rm) {
        (0b11, _) => Operand::Register(decode_register(rm, w)),

        (0b00, 0b110) => Operand::DirectAddress(Region {
            address: combined as usize,
            size: 1 + w as usize,
        }),

        (_, 0b000) => Operand::EffectiveAddress2(Register::BX, Register::SI, displacement),
        (_, 0b001) => Operand::EffectiveAddress2(Register::BX, Register::DI, displacement),
        (_, 0b010) => Operand::EffectiveAddress2(Register::BP, Register::SI, displacement),
        (_, 0b011) => Operand::EffectiveAddress2(Register::BP, Register::DI, displacement),
        (_, 0b100) => Operand::EffectiveAddress(Register::SI, displacement),
        (_, 0b101) => Operand::EffectiveAddress(Register::DI, displacement),
        (_, 0b110) => Operand::EffectiveAddress(Register::BP, displacement),
        (_, 0b111) => Operand::EffectiveAddress(Register::BX, displacement),

        _ => todo!("effective address for rm={rm}, mod={mod_val}"),
    }
}

fn decode_register(reg: u8, w: u8) -> Register {
    REG_ENCODING[(reg << 1 | w) as usize]
}

fn reg_register(context: &ParseContext) -> Register {
    let reg = context.values[Identifier::Reg as usize].unwrap();
    let w = context.values[Identifier::W as usize].unwrap();
    decode_register(reg, w)
}

fn combine_bytes(lo: Option<u8>, hi: Option<u8>) -> i16 {
    lo.unwrap_or(0) as i16 | (hi.unwrap_or(0) as i16) << 8
}
