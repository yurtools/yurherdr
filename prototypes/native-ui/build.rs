use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rerun-if-changed=rust-toolchain.toml");
    let output = Command::new(std::env::var_os("RUSTC").expect("Cargo supplies RUSTC"))
        .arg("--version")
        .output()
        .expect("cannot verify compiler");
    assert!(output.status.success(), "cannot verify compiler");
    let version = String::from_utf8(output.stdout).expect("compiler version must be UTF-8");
    assert_eq!(
        version.split_whitespace().nth(1),
        Some("1.100.0"),
        "this evaluation requires rustc 1.100.0"
    );
}
