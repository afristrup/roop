/// Ends the program: an operation on the world was asked for what it cannot
/// give, such as taking back what was already shown. Failing fast here is what
/// keeps every inverse exact.
pub fn refuse(message: &str) -> ! {
    eprintln!("roop: {message}");
    std::process::exit(70)
}
