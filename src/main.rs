// --------------------------------------------------------------------------------
// Notes
// --------------------------------------------------------------------------------
//
// http://data.matthieu.benoit.free.fr/cross/data_sheet2/8086_family_Users_Manual.pdf
//
// Registers:
// - 16-bits
//  AX  | word multiply, word divide, word IO
//  AL  | byte multiply, byte divide, byte IO, translate, decimal arithmatic
//  AH  | byte multiply, byte divide
//  BX  | translate
//  CX  | string operations, loops
//  CL  | variable shift and rotate
//  DX  | word multiply, word divide, indirect IO
//  SP  | stack pointer
//  SI  | string operations
//  DI  | string operations
//
//  Segment registers:
//  - 1M memory split into four regions.
//   CS  | code segment start (64 bytes)
//   DS  | data segment start (128 bytes)
//   SS  | current stack segment start (64 bytes)
//   ES  | extra segment start
//
//  Instruction pointer:
//   IP  | offset from CS to next instruction to fetch
//
//  Control flags:
//   TF  | trap
//   DF  | direction
//   IF  | interupt enable
//  Status flags:
//   OF  | overflow
//   SF  | sign
//   ZF  | zero
//   AF  | auxilary carry
//   PF  | parity
//   CF  | carry
//
//  Reserved memory:
//   0H - 7FH (128 bytes)
//   FFFF0H - FFFFFH (16 bytes)
//
//  Notes:
//  - data should be word aligned
//  - instructions are fetched 1 byte at a time
//
//  Interupts:
//  - up to 256 different interupts
//  -
//
//  ISA:
//   --- General purpose data transfers ---
//   MOV source, destination
//   - transfers byte or word from source to destination
//   PUSH source
//   - decrements SP by two
//   - transfers source operand to top of stack
//   POP destination
//   - transfers word at top of the stack (pointed to by SP) to destination operand
//   - increments SP by two
//   XCHG destination, source
//   - switches the content of the source and destination registers
//   XLAT translate-table
//   - replace byte in the AL register with byte from 256-byte user-coded translate table
//   - translate table expected in BX
//   - byte in AL is used as index into translate table ([BX + AL] = new AL value)
//   IN accumulator, port
//   - transfers byte or a word from input port to AL or AX register
//   - port can be immediate (0-255) or the number in DX
//   OUT accumulator, port
//   - transfers byte or a word from AL or AX register to an output port
//   - port can be immediate (0-255) or the number in DX
//   --- Address object transfers ---
//   LEA destination, source
//   - transfers offset of source operand to the destination operand
//   - source operand must be a memory operand
//   - destination must be 16-bit general purpose register
//   LDS destination, source
//   - transfers 32-bit pointer from source operand to destination operand and register DS
//   - offset word of pointer transfered to destination operand, and 16-bit general purpose register
//   - segment word of pointer transfered to register DS
//   - source operand must be a memory operand
//   LES destination, source
//   - transfers 32-bit pointer from source operand to destination operand and register ES
//   - offset word of pointer transfered to destination operand, and 16-bit general purpose register
//   - segment word of pointer transfered to register ES
//   - source operand must be a memory operand
//   --- Flag transfers ---
//   LAHF
//   - copies SF, ZF, AF, PF, and CF flags into bits 7, 6, 4, 2, and 0 of register AH, respectively
//   SAHF
//   - copies bits 7, 6, 4, 2, and 0 of register AH into SF, ZF, AF, PF, and CF flags, respectively
//   PUSHF
//   - decrements SP by two
//   - transfers all flags to the word at the top of the stack
//   - flags are not affected
//   POPF
//   - transfers bits from word at top of stack to the flag registers, replacing previous flags
//   - increment SP by two
//   --- Addition Arithmetic instructions ---
//   ADD destination, source
//   - Add byte or word
//   - Replaces the destination operand
//   ADC destination, source
//   - Add byte or word with carry
//   - Replaces the destination operand
//   - Adds 1 if CF is set
//   INC destination
//   - Increment byte or word by 1
//   AAA
//   - ASCII adjust for addition
//   DAA
//   - Decimal adjust for addition
//   --- Subtraction Arithmetic instructions ---
//   SUB destination, source
//   - Subtract byte or word
//   - Replaces the destination operand
//   SBB destination, source
//   - Subtract byte or word with borrow
//   - Subtracts 1 if CF is set
//   - Replaces the destination operand
//   DEC destination
//   - Decrement byte or word by 1
//   NEG destination
//   - Negate byte or word
//   CMP destination, source
//   - Compare byte or word
//   - Operands are unchanged, but flags are updated
//   AAS
//   - ASCII adjust for subtraction
//   DAS
//   - Decimal adjust for subtraction
//   --- Multiplication Arithmetic instructions ---
//   MUL source
//   - Multiply word or byte unsigned
//   IMUL
//   - Integer multiply word or byte
//   AAM
//   - ASCII adjust for multiply
//   --- Division Arithmetic instructions ---
//   DIV
//   - Divide byte or word unsigned
//   IDIV
//   - Integer divide byte or word
//   AAD
//   - ASCII adjust for division
//   CBW
//   - Convert byte to word
//   CWD
//   - Convert word to doubleword
//   --- Logical instructions ---
//   NOT destination
//   - Inverts bits of the byte or word operand
//   AND destination, source
//   - Overwrites destination with result of logical AND operation
//   - Bit is set if 1 in both destination and source, otherwise bit is cleared
//   OR destination, source
//   - Overwrites destination with result of logical OR operation
//   - Bit is set if 1 in either destination and source, otherwise bit is cleared
//   XOR destination, source
//   - Overwrites destination with result of logical XOR operation
//   - Bit is set if corresponding bit in destination and source are opposite, otherwise bit is cleared
//   TEST destination, source
//   - Updates flags after performing logical AND operation
//   - Destination unchanged
//   --- Shifts ---
//   SHL/SAL destination, count
//   - Shifts destination by count to the left
//   - If sign bit unchanged, clear OF
//   SAR destination, count
//   - Shifts destination by count to the right
//   - New slots on left filled by 0
//   - If sign bit unchanged, clear OF
//   SHR destination, count
//   - Shifts destination by count to the right
//   - New slots on left filled by sign bit
//   --- Rotate instructions ---
//   ...
//   --- String instructions ---
//   ...
//   --- Program transfer instructions ---
//   CALL procedure-name
//   - Intrasegment direct CALL: SP decremented by two, IP pushed to stack. Displacement added to IP
//   - Intrasegment indirect CALL: SP decremented by two, IP pushed to stack. Offset obtained from
//   word or register referenced in instruction, and replaces IP
//   - Intersegment direct CALL: SP decremented by two, CS pushed to stack. CS replaced with second word of
//   instruction memory pointer.  SP decremented by two. IP pushed to stack. Offset word replaces IP
//   - Intersegment indirect CALL: SP decremented by two, CS pushed to stack. CS replaced with content of the
//   second word of instruction memory pointer.  SP decremented by two. IP pushed to stack. Content
//   of offset word of instruction replaces IP
//   - IP automatically adjusted to next instruction to be executed before saving to stack
//   RET optional-pop-value
//   - Transfers control back to the caller procedure
//   - Pops word at top of stack, SP, into the IP
//   - Increments SP by two
//   - If intersegment, next word at top of stack popped into CS register and SP incremented by two
//   - If optional-pop-value, add it to SP
//   JMP target
//   - Transfers control to the target location
//   - Target obtained from instruction (direct) or content of memory/register (indirect)
//   - Intersegment JMP replaces IP (first word) and CS (second word) with values contained in instruction
//   --- Conditional transfer instructions ---
//   LOOP short-label
//   - Decrements CX by 1 and transfers control to target operand if CX is not 0
//   - Otherwise, instructions after LOOP is executed
//   LOOPE/LOOPZ short-label
//   - Decrements CX by 1 and transfers control to target operand if CX is not 0 and if ZF is set
//   - Otherwise, instructions after LOOPE/LOOPZ is executed
//   LOOPNE/LOOPNZ short-label
//   - Decrements CX by 1 and transfers control to target operand if CX is not 0 and if ZF is clear
//   - Otherwise, instructions after LOOPNE/LOOPNZ is executed
//   JCXZ short-label
//   - Transfers control to target operand if CX is 0
//   JA/JNBE
//   - Condition: (CF or ZF) = 0
//   - Jump if "above/not below nor equal"
//   JAE/JNB
//   - Condition: CF = 0
//   - Jump if "above or equal/not below"
//   JB/JNAE
//   - Condition: CF = 1
//   - Jump if "below/not equal or above"
//   JBE/JNA
//   - Condition: (CF or ZF) = 1
//   - Jump if "below or equal/not above"
//   JC
//   - Condition: CF = 1
//   - Jump if "carry"
//   JZ/JZ
//   - Condition: ZF = 1
//   - Jump if "equal/zero"
//   JG/JNLE
//   - ((SF xor OF) or ZF)=0
//   - Jump if "greater/not less nor equal"
//   JGE/JNL
//   - (SF xor OF) = 0
//   - Jump if "greater or equal/ not less"
//   JL/JNGE
//   - ((SF xor OF) or ZF) = 1
//   - Jump if "less or equal, not greater"
//   JNC
//   - CF = 0
//   - Jump if "not carry"
//   JNE/JNZ
//   - ZF = 0
//   - Jump if "not zero/not equal"
//   JNO
//   - OF = 0
//   - Jump if "not overflow"
//   JNP/JPO
//   - PF = 0
//   - Jump if "not parity/parity odd"
//   JNS
//   - SF = 0
//   - Jump if "not sign"
//   JO
//   - OF = 1
//   - Jump if "overflow"
//   JP/JPE
//   - PF = 1
//   - Jump if "parity/parity equal"
//   JS
//   - SF = 1
//   - Jump if "sign"
//   --- Interupt instructions ---
//   ...
//   --- Processor control instructions ---
//   ...
//   --- No-operation instructions ---
//   NOP
//   - Causes the CPU to do nothing
//
// Instruction coding format key
// Destination
// - Register or memory location containing data operated on by the instruction
// - Replaced by the result of the operation
// Source
// - Register or memory location containing data operated on by the instruction
// - Not altered
// Source-table
// - used in XLAT
// - Name of memory-translation table addressed by BX
// Target
// - used by JMP/CALL
// - A label to which control will be transfered
// - OR a register or memory location whose content is the address of location to which control is
// to be transfered
// Short-label
// - A label to which control will be transfered
// - Must be -128 to 127 bytes of the first byte of the next instruction
// Accumulator
// - Used in IN/OUT
// - Register AX for word transfers, AL for bytes
// Port
// - Used in IN/OUT
// - IO port number. Immediate (0-255) or register DX (0-64k)
// Count
// - Used in shifts/rotates
// - Immediate value 1 (?) or register CL (0-255)
// Interupt-type
// - Used in INT
// - Immediate value of 0-255 identifying interupt pointer number
// Optional-pop-value
// - Number of bytes (0-64k) to pop from stack
// ...
// String and ESC not noted for now
//
//
// Flag Effect Key
// blank  | Not altered
// 0  | cleared to 0
// 1  | set to 1
// X  | set or cleared according to result
// U  | undefined
// R  | restored from previously saved value
//
// Operand type key
// Register  | an 8-bit or 16-bit general register
// Reg16  | a 16-bit general register
// Seg-reg  | a segment register
// Accumulator  | register AX or AL
// Immediate  | constant in range 0 - FFFFH
// Imm8 | constant in range 0 - FFH
// Memory | 8-bit or 16-bit memory location
// Mem8  | 8-bit memory location
// Mem16  | 16-bit memory location
//
// Single-bit field encoding
// S
// - 0: No sign extension
// - 1: Sign extend 8-bit immediate data to 16 bits if W = 1
// W
// - 0: Instruction operates on byte data
// - 1: Instruction operates on word data
// D
// - 0: Instruction source is specified in REG field
// - 1: Instruction destination is specified in REG field
// V
// - 0: Shift/rotate count is one
// - 1: Shift/rotate count is specified in CL register
// Z
// - 0: Repeate/loop while zero flag is clear
// - 1: Repeate/loop while zero flag is set
//
//
// Register encodings
// 000
// - W=0: AL
// - W=1: AX
// 001
// - W=0: CL
// - W=1: CX
// 010
// - W=0: DL
// - W=1: DX
// 011
// - W=0: BL
// - W=1: BX
// 100
// - W=0: AH
// - W=1: SP
// 101
// - W=0: CH
// - W=1: BP
// 110
// - W=0: DH
// - W=1: SI
// 111
// - W=0: BH
// - W=1: DI
//
//
// MOD field encoding
// 00  | Memory mode, not displacement follows (except when R/M = 110, then 16-bit displacement
//    follows
// 01  | Memory mode, 8-bit displacement follows
// 10  | Memory mode, 16-bit displacement follows
// 11  | Register mode, no displacement
//
// R/M field encoding
// 000
//  MOD = 11
//   - W=0: AL
//   - W=1: AX
//  MOD == 00 (Effective address calculation): (BX) + (SI)
//  MOD == 01 (Effective address calculation): (BX) + (SI) + D8
//  MOD == 10 (Effective address calculation): (BX) + (SI) + D16
// 001
//  MOD = 11
//   - W=0: CL
//   - W=1: CX
//  MOD == 00 (Effective address calculation): (BX) + (DI)
//  MOD == 01 (Effective address calculation): (BX) + (DI) + D8
//  MOD == 10 (Effective address calculation): (BX) + (DI) + D16
// 010
//  MOD = 11
//   - W=0: DL
//   - W=1: DX
//  MOD == 00 (Effective address calculation): (BP) + (SI)
//  MOD == 01 (Effective address calculation): (BP) + (SI) + D8
//  MOD == 10 (Effective address calculation): (BP) + (SI) + D16
// 011
//  MOD = 11
//   - W=0: BL
//   - W=1: BX
//  MOD == 00 (Effective address calculation): (BP) + (DI)
//  MOD == 01 (Effective address calculation): (BP) + (DI) + D8
//  MOD == 10 (Effective address calculation): (BP) + (DI) + D16
// 100
//  MOD = 11
//   - W=0: AH
//   - W=1: SP
//  MOD == 00 (Effective address calculation): (SI)
//  MOD == 01 (Effective address calculation): (SI) + D8
//  MOD == 10 (Effective address calculation): (SI) + D16
// 101
//  MOD = 11
//   - W=0: CH
//   - W=1: BP
//  MOD == 00 (Effective address calculation): (DI)
//  MOD == 01 (Effective address calculation): (DI) + D8
//  MOD == 10 (Effective address calculation): (DI) + D16
// 110
//  MOD = 11
//   - W=0: DH
//   - W=1: SI
//  MOD == 00 (Effective address calculation): Direct address
//  MOD == 01 (Effective address calculation): (BP) + D8
//  MOD == 10 (Effective address calculation): (BP) + D16
// 111
//  MOD = 11
//   - W=0: BH
//   - W=1: DI
//  MOD == 00 (Effective address calculation): (BX)
//  MOD == 01 (Effective address calculation): (BX) + D8
//  MOD == 10 (Effective address calculation): (BX) + D16
//
//  Unsigned
//  - 8 bits (0 to 255) or 16 bits (0 to 65,535)
//  Signed
//  - 8 bits (-128 to 127) or 16 bits (-32,768 to 32, 767)
//  - Leftmost (high-order) bit is the sign. 0 = positive, 1 = negative
//  - Addition and subtraction are performed with the unsigned binary numbers
//
//  CF
//  - Set if addition carries out of high-order bit
//  - Set if subtraction borrows into the high-order bit
//  - Otherwise, cleared
//  AF
//  - Set if addition carries out of low-order half byte
//  - Set if subtraction borrows into the low-order half byte
//  - Otherwise, cleared
//  SF
//  - Set to high-order bit of result (bit 7 or bit 15)
//  - Otherwise, cleared
//  ZF
//  - Set if result of arithmetic is zero
//  - Otherwise, cleared
//  PF
//  - Set if low-order byte contains an even number of 1-bits
//  - Otherwise, cleared
//  OF
//  - Set if result of arithmetic is too large a positive number, or too low a negative
//
//
// Flag to word:
//  15  | Undefined
//  14  | Undefined
//  13  | Undefined
//  12  | Undefined
//  11  | Overflow
//  10  | Direction
//  09  | Interupt enable
//  08  | Trap flag
//  07  | Sign flag
//  06  | Zero flag
//  05  | Undefined
//  04  | Auxilary carry flag
//  03  | Undefined
//  02  | Parity flag
//  01  | Undefined
//  00  | Carry flag
//
//

mod cpu;
mod isa;
mod memory;

// --------------------------------------------------------------------------------
// Parse
// --------------------------------------------------------------------------------

// Define isa
// comptime definitions
//  instruction type
//  count: number of bits to read
//  literal: Option: bits must be the same as literal if specified
//  identifier: i.e. mod, reg, disp-lo, etc...
//  flag: optionally apply this parser if flag condition met

// parse()
//      try_parse()

// try_parse () -> Option<Instruction>
//      for isa in isas:
//          parsing logic or break

// --------------------------------------------------------------------------------
// Instructions
// --------------------------------------------------------------------------------

// cpu.run()
//      while true
//          // fetch instruction
//          // decode instruction
//          // read operands
//          // calculate
//          // load/store
//          // write back
//

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: sim_8086 <mode> <binary>");
        std::process::exit(1);
    }

    // TODO: program expects mode as first arg, right now we just skip it
    let program = std::fs::read(&args[2]).expect("failed to read binary");
    if program.len() > 0x10000 {
        eprintln!("program too large (max 64KB)");
        std::process::exit(1);
    }

    let mut cpu = cpu::Cpu::new();
    cpu.load_program(&program, 0x0000);

    match cpu.run() {
        Ok(_) => println!("cpu ended execution"),
        Err(cpu::CpuError::Halted) => println!("program halted"),
        Err(err) => eprintln!("error: {err}"),
    }
}
