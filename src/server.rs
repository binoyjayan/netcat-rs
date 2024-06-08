use std::io;

/// Server that listens on a port and streams data
/// from stdin to the client and from the client to stdout
pub async fn server(addr: &str, port: u16) -> io::Result<()> {
    let conn = format!("{}:{}", addr, port);
    let listener = tokio::net::TcpListener::bind(conn).await?;
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
