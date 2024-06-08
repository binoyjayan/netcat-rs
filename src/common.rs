use tokio::io::{self, AsyncRead, AsyncWrite};

/// Generic function to stream data from a reader to a writer
/// This function is used by both the client and server
/// to stream data from the server to stdout and from stdin to the server
/// or from the client to stdout and from stdin to the client
/// The 'static lifetime is required to ensure that the reader and writer
/// can live as long as the tokio tasks that are spawned live.
pub async fn read_write<R, W>(mut reader: R, mut writer: W) -> io::Result<()>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // Stream data from the server or client stream to stdout
    let stream_reader = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdout).await.unwrap();
    });

    // Stream data from stdin to the server or client stream
    let stream_writer = tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut writer).await.unwrap();
    });

    tokio::select! {
        _ = stream_reader => { eprintln!("Connection closed"); }
        _ = stream_writer => { eprintln!("Connection closed"); }
    }
    Ok(())
}
