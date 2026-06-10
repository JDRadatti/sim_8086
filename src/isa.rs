use crate::memory::Region;
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
// Instruction
// --------------------------------------------------------------------------------

pub enum Operand {
    Register(Register),
    Memory(Region),
    Immediate(u16),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(register) => write!(f, "{register}"),
            Operand::Memory(_region) => todo!("display memory operand"),
            Operand::Immediate(imm) => write!(f, "{imm}"),
        }
    }
}

pub enum Instruction {
    Mov(Operand, Operand),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Mov(op1, op2) => write!(f, "mov {op1}, {op2}"),
        }
    }
}

impl From<&ParseContext<'_>> for Instruction {
    fn from(_context: &ParseContext) -> Self {
        Instruction::Mov(
            Operand::Register(Register::AL),
            Operand::Register(Register::AL),
        )
    }
}

// --------------------------------------------------------------------------------
// Parser
// --------------------------------------------------------------------------------

#[derive(Copy, Clone)]
#[repr(usize)]
enum Identifier {
    Opcode,
    Mod,
    D,
    W,
    Rm,
    DispLo,
    DispHi,
    Reg,
}

type ParserFlag = u8;
const FLAG_DISP8: ParserFlag = 0b1;
const FLAG_DISP16: ParserFlag = 0b10;

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
    values: [Option<u8>; 7],
    flags: u8,
}

impl<'a> ParseContext<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        ParseContext {
            bytes: bytes,
            byte_index: 0,
            bit_offset: 0,
            flags: 0,
            values: [None; 7], // Size MUST match number of variants of Identifier Enum
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

// --------------------------------------------------------------------------------
// Instruction Encodings
// --------------------------------------------------------------------------------

#[rustfmt::skip]
const MOD_REG_OR_MEM_TO_OR_FROM_REG: &[Parser] = &[
    Parser::new(Identifier::Opcode, Some(0b100010), None, 6),
    PARSER_D,
    PARSER_W,
    PARSER_MOD,
    PARSER_REG,
    PARSER_RM,
    PARSER_DISPLO,
    PARSER_DISPHI,
];

const INSTRUCTION_TABLE: &[&[Parser]] = &[MOD_REG_OR_MEM_TO_OR_FROM_REG];

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

fn try_decode(encoding: &[Parser], bytes: &[u8]) -> Option<(Instruction, usize)> {
    let mut context = ParseContext::new(bytes);
    for parser in encoding {
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

    let instruction = Instruction::from(&context);
    Some((instruction, context.byte_index))
}
