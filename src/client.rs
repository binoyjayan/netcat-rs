use std::io;

/// Client that connect to a TCP server and stream data
/// from stdin to the server and from the server to stdout
pub async fn client(addr: &str, port: u16) -> io::Result<()> {
    let conn = format!("{}:{}", addr, port);
    let stream = tokio::net::TcpStream::connect(conn).await?;
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
