use std::io::{self, Read, Write};
use std::net;
use std::net::TcpListener;

pub fn run(addr: &str, port: u16) -> io::Result<()> {
    let listener = TcpListener::bind((addr, port))?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream)?;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

pub fn handle_client(mut stream: net::TcpStream) -> io::Result<()> {
    let mut buf = [0; 1024];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        stream.write_all(&buf[..n])?;
    }
    Ok(())
}
