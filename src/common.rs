use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
    let stream_reader =
        tokio::spawn(async move { tokio::io::copy(&mut reader, &mut stdout).await });

    // Stream data from stdin to the server or client stream
    let stream_writer = tokio::spawn(async move { tokio::io::copy(&mut stdin, &mut writer).await });

    tokio::select! {
        res = stream_reader => {
            match res {
                Ok(Ok(_)) => (),
                Ok(Err(err)) => {
                    return Err(err);
                },
                Err(err) => {
                    return Err(io::Error::new(io::ErrorKind::Other, err));
                }
            }
        },
        res = stream_writer => {
            match res {
                Ok(Ok(_)) => (),
                Ok(Err(err)) => {
                    return Err(err);
                },
                Err(err) => {
                    return Err(io::Error::new(io::ErrorKind::Other, err));
                }
            }
        }
    }
    Ok(())
}

/// Generic function to exec a shell command and stream data from a reader to a writer
pub async fn read_write_exec<R, W>(mut reader: R, mut writer: W, cmd: &str) -> io::Result<()>
where
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let child = tokio::process::Command::new(cmd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut stdin = child.stdin.unwrap();
    let mut stdout = child.stdout.unwrap();
    let mut stderr = child.stderr.unwrap();

    // Stream stdout and stderr output of command to the client
    let mut stdout_buf = vec![0u8; 1024];
    let mut stderr_buf = vec![0u8; 1024];
    // Buffer to receive data from the client
    let mut recv_buf = vec![0u8; 1024];

    loop {
        tokio::select! (
            // Read data from the stdout of the command and write to the client
            res = stdout.read(&mut stdout_buf) => {
                match res {
                    Ok(nbytes) => {
                        if nbytes == 0 {
                            break;
                        }
                        let _ = writer.write(&stdout_buf[..nbytes]).await?;
                    }
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }
            // Read data from the stderr of the command and write to the client
            res = stderr.read(&mut stderr_buf) => {
                match res {
                    Ok(nbytes) => {
                        if nbytes == 0 {
                            break;
                        }
                        let _ = writer.write(&stderr_buf[..nbytes]).await?;
                    }
                    Err(e) => {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }
            // Read data from the client and write to the stdin of the command
            res = reader.read(&mut recv_buf) => {
                match res {
                    Ok(nbytes) => {
                        let _ = stdin.write(&recv_buf[..nbytes]).await?;
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
