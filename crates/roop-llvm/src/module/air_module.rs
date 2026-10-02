use crate::{AIR_DATALAYOUT, AIR_TRIPLE, AIR_VERSION, METAL_VERSION};
use crate::{Kernel, air_arg_metadata};

/// One AIR module holding every Metal kernel, with the `air.*` metadata that
/// makes each function a compute kernel.
pub fn air_module(types: &str, kernels: &[&Kernel]) -> String {
    let mut out = format!(
        "target datalayout = \"{AIR_DATALAYOUT}\"\ntarget triple = \"{AIR_TRIPLE}\"\n\n{types}\n"
    );
    let mut meta: Vec<String> = Vec::new();
    let mut alloc = |body: String| {
        meta.push(body);
        meta.len() - 1
    };
    let version = alloc(format!(
        "!{{i32 {}, i32 {}, i32 {}}}",
        AIR_VERSION.0, AIR_VERSION.1, AIR_VERSION.2
    ));
    let language = alloc(format!(
        "!{{!\"Metal\", i32 {}, i32 {}, i32 {}}}",
        METAL_VERSION.0, METAL_VERSION.1, METAL_VERSION.2
    ));
    let empty = alloc("!{}".into());
    let mut entries = Vec::new();
    for kernel in kernels {
        out.push_str(&kernel.text);
        out.push('\n');
        let args: Vec<String> = kernel
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| format!("!{}", alloc(air_arg_metadata(i, arg))))
            .collect();
        let signature = format!("void ({})*", kernel.param_types.join(", "));
        let args_node = alloc(format!("!{{{}}}", args.join(", ")));
        let entry = alloc(format!(
            "!{{{signature} @{}, !{empty}, !{args_node}}}",
            kernel.name
        ));
        entries.push(format!("!{entry}"));
    }
    out.push_str(&format!(
        "\n!air.kernel = !{{{}}}\n!air.version = !{{!{version}}}\n!air.language_version = !{{!{language}}}\n",
        entries.join(", ")
    ));
    for (id, body) in meta.iter().enumerate() {
        out.push_str(&format!("!{id} = {body}\n"));
    }
    out
}
