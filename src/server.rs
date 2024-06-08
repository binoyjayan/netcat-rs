use std::io::{self, stdout, Read, Write};
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
    println!("Connection from: {}", stream.peer_addr()?);
    let mut buf = [0; 1024];
    loop {
        let nbytes = stream.read(&mut buf)?;
        if nbytes == 0 {
            break;
        }
        stream.write_all(&buf[..nbytes])?;
        stdout().write_all(buf[..nbytes].as_ref())?;
    }
    println!("Connection from {} closed", stream.peer_addr()?);
    let _ = stream.shutdown(net::Shutdown::Both);
    Ok(())
}
