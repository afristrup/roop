use crate::Options;

/// Triple line and the attribute group that pins the CPU model.
pub fn module_header(options: &Options) -> String {
    let mut out = String::new();
    if let Some(triple) = &options.triple {
        out.push_str(&format!("target triple = \"{triple}\"\n"));
    }
    if let Some(cpu) = &options.cpu {
        out.push_str(&format!("attributes #0 = {{ \"target-cpu\"=\"{cpu}\" }}\n"));
    }
    out
}
