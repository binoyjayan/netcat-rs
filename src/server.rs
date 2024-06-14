use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::common::read_write_exec;

/// Server that listens on a port and streams data
/// from stdin to the client and from the client to stdout
pub async fn tcp_server(addr: &str, port: u16) -> io::Result<()> {
    let listener = tokio::net::TcpListener::bind((addr, port)).await?;
    let (stream, addr) = listener.accept().await?;

    // Get reader and writer from the stream
    let (mut reader, mut writer) = stream.into_split();
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // Stream data from client stream to stdout
    let client_reader = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdout).await.unwrap();
    });

    // Stream data from stdin to the client stream
    let client_writer = tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut writer).await.unwrap();
    });

    tokio::select! {
        _ = client_reader => { eprintln!("Client {} disconnected", addr); }
        _ = client_writer => { eprintln!("Client {} disconnected", addr); }
    }

    Ok(())
}

/// Server that listens on a port and streams data
/// from stdin to the client and from the client to stdout
/// This function uses UDP instead of TCP
pub async fn udp_server(addr: &str, port: u16) -> io::Result<()> {
    let socket = tokio::net::UdpSocket::bind((addr, port)).await?;

    let mut stdin_buf = vec![0u8; 1024];
    let mut recv_buf = vec![0u8; 1024];

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    let mut is_connected = false;
    let mut buf = Vec::new();

    loop {
        tokio::select! (
            res = socket.recv_from(&mut recv_buf) => {
                match res {
                    Ok((nbytes, addr)) => {
                        if !is_connected {
                            socket.connect(addr).await?;
                            is_connected = true;
                            // Send buffered data first upon connection
                            if !buf.is_empty() {
                                let _ = socket.send(&buf).await?;
                                buf.clear();
                            }
                        }
                        let _ = stdout.write(&recv_buf[..nbytes]).await?;
                    }
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }
            res = stdin.read(&mut stdin_buf) => {
                match res {
                    Ok(nbytes) => {
                        if nbytes == 0 {
                            break;
                        }
                        if is_connected {
                            let _ = socket.send(&stdin_buf[..nbytes]).await?;
                        } else {
                            buf.extend_from_slice(&stdin_buf[..nbytes]);
                        }
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

/// Server that listens on a port and streams data from a remote  shell
pub async fn tcp_shell(addr: &str, port: u16, cmd: &str) -> io::Result<()> {
    let listener = tokio::net::TcpListener::bind((addr, port)).await?;
    let (stream, _addr) = listener.accept().await?;
    // Get reader and writer from the stream
    let (reader, writer) = stream.into_split();
    read_write_exec(reader, writer, cmd).await?;
    Ok(())
}
