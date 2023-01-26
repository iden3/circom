use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let fr_ll = "src/llvm_elements/fr.ll";
    println!("cargo:rerun-if-changed={}", fr_ll);
    let fr_bc = PathBuf::from(env::var("OUT_DIR").unwrap());
    let fr_bc = fr_bc.join("fr.bc");
    Command::new("llvm-as")
        .args([fr_ll, "-o", fr_bc.to_str().unwrap()])
        .output()
        .expect("failed to execute process");

    Command::new("ls").args([env::var("OUT_DIR").unwrap()]).output().expect("Failed to ls");
}