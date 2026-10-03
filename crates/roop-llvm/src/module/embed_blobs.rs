/// Replaces the host module's external `@roop_metallib` and `@roop_ptx`
/// declarations with definitions holding the compiled device code, so the
/// result links without any side files.
pub fn embed_blobs(host: &str, metallib: Option<&[u8]>, ptx: Option<&str>) -> String {
    let bytes = |data: &[u8]| -> String { data.iter().map(|b| format!("\\{b:02X}")).collect() };
    let lib = metallib.unwrap_or(&[]);
    let mut ptx_text = ptx.unwrap_or("").as_bytes().to_vec();
    ptx_text.push(0);
    host.replace(
        "@roop_metallib = external constant i8",
        &format!(
            "@roop_metallib = constant [{} x i8] c\"{}\"",
            lib.len(),
            bytes(lib)
        ),
    )
    .replace(
        "@roop_metallib_len = external constant i64",
        &format!("@roop_metallib_len = constant i64 {}", lib.len()),
    )
    .replace(
        "@roop_ptx = external constant i8",
        &format!(
            "@roop_ptx = constant [{} x i8] c\"{}\"",
            ptx_text.len(),
            bytes(&ptx_text)
        ),
    )
}
