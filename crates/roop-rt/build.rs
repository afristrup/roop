use std::path::{Path, PathBuf};
use std::process::Command;

const CLANGS: [&str; 2] = ["clang", "/opt/homebrew/opt/llvm/bin/clang"];

/// A static library of C kernels, and the cfg that says it could not be built.
struct Library {
    name: &'static str,
    sources: &'static [&'static str],
    march: &'static str,
    missing: &'static str,
    reason: &'static str,
}

const LIBRARIES: [Library; 2] = [
    Library {
        name: "roop_sme",
        sources: &["kernels/sme_dgemm.c", "kernels/sme_daxpy.c"],
        march: "armv8.7-a+sme-f64f64+sme2",
        missing: "no_sme_kernel",
        reason: "no clang with SME support, so the matrix kernels use plain loops",
    },
    Library {
        name: "roop_q12",
        sources: &["kernels/q12_matmul.c", "kernels/i64_matmul.c"],
        march: "armv8.7-a",
        missing: "no_q12_kernel",
        reason: "no clang for aarch64, so the integer matrix kernels use plain loops",
    },
];

fn main() {
    println!("cargo:rerun-if-changed=kernels");
    for library in &LIBRARIES {
        println!("cargo::rustc-check-cfg=cfg({})", library.missing);
        if !build(library) {
            println!("cargo:rustc-cfg={}", library.missing);
        }
    }
}

/// Compiles a library of kernels, or says it could not: only Apple Silicon has
/// the matrix unit, and the compiler has to know it.
fn build(library: &Library) -> bool {
    if std::env::var("TARGET").as_deref() != Ok("aarch64-apple-darwin") {
        return false;
    }
    let out = std::env::var("OUT_DIR").unwrap();
    let objects: Option<Vec<PathBuf>> = library
        .sources
        .iter()
        .map(|s| compile(s, library.march, &out))
        .collect();
    let archive = Path::new(&out).join(format!("lib{}.a", library.name));
    let archived = objects.is_some_and(|objects| {
        succeeds(Command::new("ar").arg("crs").arg(&archive).args(&objects))
    });
    if archived {
        println!("cargo:rustc-link-search=native={out}");
        println!("cargo:rustc-link-lib=static={}", library.name);
    } else {
        println!("cargo:warning={}", library.reason);
    }
    archived
}

fn compile(source: &str, march: &str, out: &str) -> Option<PathBuf> {
    let stem = Path::new(source).file_stem()?;
    let object = Path::new(out).join(stem).with_extension("o");
    let compiled = CLANGS.iter().any(|clang| {
        succeeds(
            Command::new(clang)
                .args(["-O2", "-fno-builtin", "-mcpu=apple-m4"])
                .arg(format!("-march={march}"))
                .args(["-c", source, "-o"])
                .arg(&object),
        )
    });
    compiled.then_some(object)
}

fn succeeds(command: &mut Command) -> bool {
    command.output().is_ok_and(|o| o.status.success())
}
