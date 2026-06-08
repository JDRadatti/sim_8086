use std::path::{Path, PathBuf};
use std::process::Command;

macro_rules! decode_tests {
    ($(
        {
            name: $name:ident,
            file: $file:literal $(,)?
        }
    ),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                $crate::infra::run_decode_test(stringify!($name), $file);
            }
        )*
    };
}

pub(crate) fn run_decode_test(name: &str, file: &str) {
    let src = Path::new("tests").join(file);
    let ref_bin = Path::new("tests").join(format!("{name}_ref.bin"));
    let dec_asm = Path::new("tests").join(format!("{name}_dec.s"));
    let dec_bin = Path::new("tests").join(format!("{name}_dec.bin"));

    // Step 1: Assemble the source to get the reference binary
    nasm_compile(&src, &ref_bin);

    // Step 2: Decode the reference binary
    let decoded = run_decoder(&ref_bin);

    // Step 3: Re-assemble the decoded output
    let dec_source = format!("BITS 16\n{}", decoded);
    std::fs::write(&dec_asm, dec_source).expect("could not write decoded assembly");
    nasm_compile(&dec_asm, &dec_bin);

    // Step 4: Compare the two binaries byte-for-byte
    let ref_bytes = std::fs::read(&ref_bin).expect("could not read reference binary");
    let dec_bytes = std::fs::read(&dec_bin).expect("could not read decoded binary");
    assert_eq!(ref_bytes, dec_bytes, "mismatch for test '{name}'");
}

fn nasm_compile(input: &Path, output: &Path) {
    let out = Command::new("nasm")
        .arg("-f")
        .arg("bin")
        .arg(input)
        .arg("-o")
        .arg(output)
        .output()
        .expect("could not run nasm");
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        panic!("nasm assembly failed: {stderr}");
    }
}

fn run_decoder(bin_path: &Path) -> String {
    let decoder: PathBuf = ["target", "debug", env!("CARGO_PKG_NAME")].iter().collect();
    let out = Command::new(&decoder)
        .arg("decode")
        .arg(bin_path)
        .output()
        .expect("could not run decoder");

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        panic!("decoder failed: {stderr}");
    }

    String::from_utf8(out.stdout).unwrap().trim().to_string()
}
