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
}
