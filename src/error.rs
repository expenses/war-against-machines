use bincode;
use std::io;
use std::net;

error_chain! {
    foreign_links {
        Bincode(bincode::Error);
        Io(io::Error);
        AddrParse(net::AddrParseError);
    }
}

pub fn display_error(error: &Error) {
    error!("{}:", error);

    for child in error.iter().skip(1) {
        eprintln!("    {}", child);
    }
}
