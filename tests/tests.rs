#[macro_use]
mod infra;

decode_tests! {
    {
        name: mov_imm8,
        file: "mov_imm8.s",
    },
    {
        name: mov_imm16,
        file: "mov_imm16.s",
    },
    {
        name: mov_reg_reg8,
        file: "mov_reg_reg8.s",
    },
    {
        name: mov_reg_reg16,
        file: "mov_reg_reg16.s",
    },
    {
        name: mov_mem_reg,
        file: "mov_mem_reg.s",
    },
    {
        name: mov_mem_imm,
        file: "mov_mem_imm.s",
    },
    {
        name: mov_mem_disp8,
        file: "mov_mem_disp8.s",
    },
    {
        name: mov_mem_disp16,
        file: "mov_mem_disp16.s",
    },
    {
        name: mov_imm8_signed,
        file: "mov_imm8_signed.s",
    },
    {
        name: mov_imm16_signed,
        file: "mov_imm16_signed.s",
    },
    {
        name: mov_mem_disp8_signed,
        file: "mov_mem_disp8_signed.s",
    },
    {
        name: mov_mem_disp16_signed,
        file: "mov_mem_disp16_signed.s",
    },
    {
        name: mov_mem_acc,
        file: "mov_mem_acc.s",
    },
    {
        name: mov_acc_mem,
        file: "mov_acc_mem.s",
    },
    {
        name: mov_seg_reg,
        file: "mov_seg_reg.s",
    },
    {
        name: mov_seg_mem,
        file: "mov_seg_mem.s",
    },
    {
        name: mov_zero_disp,
        file: "mov_zero_disp.s",
    },
}
