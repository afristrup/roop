use std::path::PathBuf;
use std::process::Command;

const SOURCE: &str = "kernels/sme_dgemm.c";

fn main() {
    println!("cargo:rerun-if-changed={SOURCE}");
    println!("cargo::rustc-check-cfg=cfg(no_sme_kernel)");
    if !built() {
        println!("cargo:rustc-cfg=no_sme_kernel");
    }
}

/// Compiles the SME matrix kernel into a static library, or says it could not:
/// only Apple Silicon has the unit, and the compiler needs to know SME.
fn built() -> bool {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target != "aarch64-apple-darwin" {
        return false;
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let object = out.join("sme_dgemm.o");
    let compiled = clang()
        .args([
            "-O2",
            "-fno-builtin",
            "-mcpu=apple-m4",
            "-march=armv8.7-a+sme-f64f64",
        ])
        .args(["-c", SOURCE, "-o"])
        .arg(&object)
        .status()
        .is_ok_and(|s| s.success());
    let archived = compiled
        && Command::new("ar")
            .arg("crs")
            .arg(out.join("libroop_sme.a"))
            .arg(&object)
            .status()
            .is_ok_and(|s| s.success());
    if archived {
        println!("cargo:rustc-link-search=native={}", out.display());
        println!("cargo:rustc-link-lib=static=roop_sme");
    } else {
        println!("cargo:warning=no clang with SME support; dgemm falls back to plain loops");
    }
    archived
}

fn clang() -> Command {
    let brew = "/opt/homebrew/opt/llvm/bin/clang";
    let path = if Command::new("clang")
        .arg("--version")
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("clang version"))
        && std::path::Path::new(brew).exists() == false
    {
        "clang"
    } else {
        brew
    };
    Command::new(path)
}
