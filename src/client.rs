use std::io::Write;
use std::{io, net};

/// Connect to a TCP server
pub fn run(host: &str, port: u16) -> io::Result<()> {
    let addr = format!("{}:{}", host, port);
    let mut stream = net::TcpStream::connect(addr)?;
    stream.write_all(b"Hello, TCP\n")?;
    Ok(())
}
