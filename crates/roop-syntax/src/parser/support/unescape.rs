/// The bytes a string or byte literal stands for, from its text without the
/// quotes. The escapes are `\n`, `\t`, `\r`, `\0`, `\\`, `\"`, `\'` and `\xHH`.
pub fn unescape(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            continue;
        }
        let byte = match chars.next() {
            Some('n') => b'\n',
            Some('t') => b'\t',
            Some('r') => b'\r',
            Some('0') => 0,
            Some('\\') => b'\\',
            Some('"') => b'"',
            Some('\'') => b'\'',
            Some('x') => {
                let digits: String = chars.by_ref().take(2).collect();
                u8::from_str_radix(&digits, 16).map_err(|_| format!("bad escape `\\x{digits}`"))?
            }
            other => return Err(format!("unknown escape `\\{}`", other.unwrap_or(' '))),
        };
        out.push(byte);
    }
    Ok(out)
}
