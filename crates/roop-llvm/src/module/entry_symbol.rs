/// The program starts at `main`, so the C entry point takes that name and the
/// roop function is called `roop_main`, whoever calls it.
pub fn entry_symbol(name: &str) -> String {
    if name == "main" {
        "roop_main".into()
    } else {
        name.into()
    }
}
