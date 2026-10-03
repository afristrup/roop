use crate::{handle, read_message, write_message};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

/// Serves one editor over the given streams until it says `exit` or hangs up.
pub fn serve(mut input: impl BufRead, mut output: impl Write) -> io::Result<()> {
    let mut documents = HashMap::new();
    while let Some(message) = read_message(&mut input)? {
        if message["method"] == "exit" {
            break;
        }
        for reply in handle(&mut documents, &message) {
            write_message(&mut output, &reply)?;
        }
    }
    Ok(())
}
