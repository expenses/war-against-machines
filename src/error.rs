use std::io;
use bincode;

error_chain! {
    foreign_links {
        Bincode(bincode::Error);
        Io(io::Error);
    }
}

pub fn display_error(error: &Error) {
    error!("{}", error);

    for child in error.iter().skip(1) {
        eprintln!("    {}", child);
    }
}