use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/syscalls.S");
    println!("cargo:rerun-if-changed=link.x");

    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("fasm")
        .arg("src/syscalls.S")
        .arg(&format!("{}/libsyscalls.a", out_dir))
        .status()
        .unwrap();
    // Command::new("ar")
    //     .arg("crus")
    //     .arg(&format!("{}/libsyscalls.a", out_dir))
    //     .arg(&format!("{}/libsyscalls.o", out_dir))
    //     .status().unwrap();

    println!("cargo:rustc-link-search={}", out_dir)
}
