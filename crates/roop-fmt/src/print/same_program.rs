use roop_syntax::parse;

/// Debug text of the program with every span blanked out.
fn shape(src: &str) -> Option<String> {
    let text = format!("{:?}", parse(src).ok()?);
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() && !out.ends_with(|p: char| p.is_alphanumeric() || p == '.') {
            let mut digits = String::from(c);
            while let Some(d) = chars.next_if(char::is_ascii_digit) {
                digits.push(d);
            }
            let range = chars.clone().take(2).collect::<String>() == "..";
            if range {
                chars.next();
                chars.next();
                while chars.next_if(char::is_ascii_digit).is_some() {}
                out.push('_');
            } else {
                out.push_str(&digits);
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}

/// Whether both texts parse to the same program, apart from where things sit.
pub fn same_program(before: &str, after: &str) -> bool {
    shape(before).is_some_and(|b| shape(after) == Some(b))
}
