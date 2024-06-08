use std::io;

pub async fn client(addr: &str, port: u16) -> io::Result<()> {
    let conn = format!("{}:{}", addr, port);
    let stream = tokio::net::TcpStream::connect(conn).await?;
    // Get reader and writer from the stream
    let (mut reader, mut writer) = stream.into_split();
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // Stream data from stdin to the network stream
    let client_read = tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut writer).await.unwrap();
    });

    // Stream data from network to stdout
    let client_write = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdout).await.unwrap();
    });

    tokio::select! {
        _ = client_read => { /* connection closed */ }
        _ = client_write => { /* connection closed */ }
    }

    Ok(())
}


pub async fn server(addr: &str, port: u16) -> io::Result<()> {
    let conn = format!("{}:{}", addr, port);
    let stream = tokio::net::TcpStream::connect(conn).await?;
    // Get reader and writer from the stream
    let (mut reader, mut writer) = stream.into_split();
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // Stream data from stdin to the network stream
    let client_read = tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut writer).await.unwrap();
    });

    // Stream data from network to stdout
    let client_write = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdout).await.unwrap();
    });

    tokio::select! {
        _ = client_read => { /* connection closed */ }
        _ = client_write => { /* connection closed */ }
    }

    Ok(())
}
