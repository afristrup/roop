use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCES: [&str; 2] = ["kernels/sme_dgemm.c", "kernels/sme_daxpy.c"];
const CLANGS: [&str; 2] = ["clang", "/opt/homebrew/opt/llvm/bin/clang"];

fn main() {
    println!("cargo:rerun-if-changed=kernels");
    println!("cargo::rustc-check-cfg=cfg(no_sme_kernel)");
    if !build_kernels() {
        println!("cargo:rustc-cfg=no_sme_kernel");
    }
}

/// Compiles the SME kernels into a static library, or says it could not: only
/// Apple Silicon has the matrix unit, and the compiler has to know SME.
fn build_kernels() -> bool {
    if std::env::var("TARGET").as_deref() != Ok("aarch64-apple-darwin") {
        return false;
    }
    let out = std::env::var("OUT_DIR").unwrap();
    let objects: Option<Vec<PathBuf>> = SOURCES.iter().map(|s| compile(s, &out)).collect();
    let archive = Path::new(&out).join("libroop_sme.a");
    let archived = objects.is_some_and(|objects| {
        succeeds(Command::new("ar").arg("crs").arg(&archive).args(&objects))
    });
    if archived {
        println!("cargo:rustc-link-search=native={out}");
        println!("cargo:rustc-link-lib=static=roop_sme");
    } else {
        println!("cargo:warning=no clang with SME support, so the matrix kernels use plain loops");
    }
    archived
}

fn compile(source: &str, out: &str) -> Option<PathBuf> {
    let stem = Path::new(source).file_stem()?;
    let object = Path::new(out).join(stem).with_extension("o");
    let compiled = CLANGS.iter().any(|clang| {
        succeeds(
            Command::new(clang)
                .args(["-O2", "-fno-builtin", "-mcpu=apple-m4"])
                .args(["-march=armv8.7-a+sme-f64f64+sme2", "-c", source, "-o"])
                .arg(&object),
        )
    });
    compiled.then_some(object)
}

fn succeeds(command: &mut Command) -> bool {
    command.output().is_ok_and(|o| o.status.success())
}
