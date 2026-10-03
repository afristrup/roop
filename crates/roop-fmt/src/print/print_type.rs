use roop_syntax::Type;

pub fn print_type(ty: &Type) -> String {
    match ty {
        Type::Named(name) => name.clone(),
        Type::Ref { mutable, inner } => {
            let keyword = if *mutable { "&mut " } else { "&" };
            format!("{keyword}{}", print_type(inner))
        }
        Type::Array(elem, len) => format!("[{}; {len}]", print_type(elem)),
        Type::Stack(elem, len) => format!("Stack<{}, {len}>", print_type(elem)),
        Type::Param { elem, len, stack } if *stack => {
            format!("Stack<{}, {len}>", print_type(elem))
        }
        Type::Param { elem, len, .. } => format!("[{}; {len}]", print_type(elem)),
    }
}
