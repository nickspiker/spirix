// Compiles each port and links it in.
//
// clang accepts LLVM IR and Objective-C directly, so no separate LLVM install is needed.
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn archive(out: &Path, name: &str, obj: &Path) {
    let lib = out.join(format!("lib{name}.a"));
    let _ = std::fs::remove_file(&lib);
    assert!(
        Command::new("ar")
            .arg("crs")
            .arg(&lib)
            .arg(obj)
            .status()
            .expect("ar not found")
            .success(),
        "ar failed for {name}"
    );
    println!("cargo:rustc-link-lib=static={name}");
}

fn main() {
    let ports = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search=native={}", out.display());
    for f in ["circle_divide.ll", "circle_divide.m"] {
        println!("cargo:rerun-if-changed={}", ports.join(f).display());
    }

    assert!(
        have("clang"),
        "clang is required to build the LLVM IR and Objective-C ports"
    );

    // --- hand-written LLVM IR ---
    let obj = out.join("ir.o");
    assert!(
        Command::new("clang")
            .args(["-c", "-O2"])
            .arg(ports.join("circle_divide.ll"))
            .arg("-o")
            .arg(&obj)
            .status()
            .unwrap()
            .success(),
        "clang failed on circle_divide.ll"
    );
    archive(&out, "spxir", &obj);

    // --- Objective-C, C structs, no Foundation ---
    let obj = out.join("objc.o");
    assert!(
        Command::new("clang")
            .args(["-c", "-O2"])
            .arg(ports.join("circle_divide.m"))
            .arg("-o")
            .arg(&obj)
            .status()
            .unwrap()
            .success(),
        "clang failed on circle_divide.m"
    );
    archive(&out, "spxobjc", &obj);
}
