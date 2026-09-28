// Compiles each port and links it in.
//
// clang accepts LLVM IR and Objective-C directly; swiftc needs -parse-as-library because the file has no top-level entry point. A missing toolchain is not a build failure: the port is skipped and the harness reports it as skipped, so this still runs somewhere without Swift installed.
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
    for f in ["circle_divide.ll", "circle_divide.m", "CircleDivide.swift"] {
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

    // --- Swift ---
    if have("swiftc") {
        let obj = out.join("swift.o");
        // -module-name is explicit: without it the module is named after the output file, which leaks into every mangled symbol.
        let ok = Command::new("swiftc")
            .args([
                "-O",
                "-wmo",
                "-parse-as-library",
                "-module-name",
                "SpirixPorts",
                "-c",
            ])
            .arg(ports.join("CircleDivide.swift"))
            .arg("-o")
            .arg(&obj)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            archive(&out, "spxswift", &obj);
            // The struct's Equatable conformance needs Swift runtime metadata.
            if cfg!(target_os = "macos") {
                println!("cargo:rustc-link-search=native=/usr/lib/swift");
            }
            println!("cargo:rustc-link-lib=dylib=swiftCore");
            println!("cargo:rustc-cfg=have_swift");
        } else {
            println!("cargo:warning=swiftc present but failed; skipping the Swift port");
        }
    } else {
        println!("cargo:warning=swiftc not found; skipping the Swift port");
    }
    println!("cargo:rustc-check-cfg=cfg(have_swift)");
}
