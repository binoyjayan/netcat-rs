use std::{io::Write, net::TcpStream};

/// Connect to a TCP server
pub fn run(host: &str, port: u16) -> Result<(), String> {
    let addr = format!("{}:{}", host, port);
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("{}:{}", addr, e))?;
    stream
        .write(b"Hello, TCP\n")
        .map_err(|e| format!("{}:{}", addr, e))?;
    Ok(())
}
