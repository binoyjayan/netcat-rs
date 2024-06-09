use rustls::pki_types;
use std::{io, sync};
use tokio_rustls::rustls;

use crate::common::read_write;

pub async fn tls_server(
    host: &str,
    port: u16,
    cafile: Option<&str>,
    cert_file: &str,
    key_file: &str,
) -> io::Result<()> {
    let addr = format!("{}:{}", host, port);

    // Create a new empty root certificate store
    let mut root_cert_store = rustls::RootCertStore::empty();
    // Load an optional CA certificate
    if let Some(cafile) = cafile {
        // We have to use a cursor since rustls_pemfile::certs() expects a type that
        // implements 'std::io::BufRead' but not 'tokio::io::BufReader'
        let data = tokio::fs::read(cafile).await?;
        let mut pem = std::io::Cursor::new(data);
        // let mut pem = std::io::BufReader::new(std::fs::File::open(cafile)?);
        for cert in rustls_pemfile::certs(&mut pem) {
            root_cert_store
                .add(cert?)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        }
    } else {
        // Load the default CA certificates
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let certs = load_certs(cert_file).await?;
    let key = load_key(key_file).await?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let acceptor = tokio_rustls::TlsAcceptor::from(sync::Arc::new(config));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let (tcp_stream, _addr) = listener.accept().await?;
    // Wrap the tcp stream with a TLS acceptor
    let stream = acceptor.accept(tcp_stream).await?;
    let (reader, writer) = tokio::io::split(stream);

    read_write(reader, writer).await?;

    Ok(())
}

/// establishes a TLS (Transport Layer Security) connection to a specified
/// host and port, with an optional custom certificate authority (CA)
pub async fn tls_client(host: &str, port: u16, ca: Option<&str>) -> io::Result<()> {
    let addr = format!("{}:{}", host, port);

    // Create a new empty root certificate store
    let mut root_cert_store = rustls::RootCertStore::empty();
    // Load an optional CA certificate
    if let Some(cafile) = ca {
        let certs = load_certs(cafile).await?;
        for cert in certs {
            root_cert_store
                .add(cert)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        }
    } else {
        // Load the default CA certificates
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = tokio_rustls::TlsConnector::from(sync::Arc::new(config));
    let server_name = host.to_string().try_into().unwrap();
    let stream = tokio::net::TcpStream::connect(&addr).await?;
    let stream = connector.connect(server_name, stream).await?;

    // TlsStream does not implement 'into_split()' so use 'split()' instead
    let (reader, writer) = tokio::io::split(stream);

    read_write(reader, writer).await?;

    Ok(())
}

async fn load_certs(path: &str) -> io::Result<Vec<pki_types::CertificateDer<'static>>> {
    let data = tokio::fs::read(path).await?;
    // We have to use a cursor since rustls_pemfile::certs() expects a type that
    // implements 'std::io::BufRead' but not 'tokio::io::BufReader'
    let mut cursor = std::io::Cursor::new(data);
    rustls_pemfile::certs(&mut cursor).collect()
}

async fn load_key(path: &str) -> io::Result<pki_types::PrivateKeyDer<'static>> {
    let data = tokio::fs::read(path).await?;
    let mut cursor = std::io::Cursor::new(data);
    let key = rustls_pemfile::private_key(&mut cursor)?;
    match key {
        Some(key) => Ok(key),
        None => {
            let msg = format!("no keys found in the provided file {}", path);
            Err(io::Error::new(io::ErrorKind::InvalidData, msg))
        }
    }
}

async fn _load_key(path: &str) -> io::Result<pki_types::PrivateKeyDer<'static>> {
    let data = tokio::fs::read(path).await?;
    let mut cursor = std::io::Cursor::new(data);
    // Iterate over RSA private keys in the PEM file
    let key = match rustls_pemfile::rsa_private_keys(&mut cursor).next() {
        Some(Ok(key)) => Ok(key.into()),
        Some(Err(err)) => Err(io::Error::new(io::ErrorKind::InvalidData, err)),
        None => {
            let msg = format!("no keys found in the provided file {}", path);
            Err(io::Error::new(io::ErrorKind::InvalidData, msg))
        }
    };

    key
}
