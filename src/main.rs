use clap::Parser;

mod client;
mod common;
mod server;
mod tls;

#[derive(Parser, Debug)]
#[command(name = "netcat", author, version = "1.0", long_about)]
struct Cli {
    /// Listen on a port
    #[arg(short, long, conflicts_with = "port", value_name = "port")]
    listen: Option<u16>,
    /// Address to connect or listen
    addr: Option<String>,
    /// Port to connect
    port: Option<u16>,
    /// certificate authority file
    #[arg(short = 'C', long, value_name = "file")]
    ca: Option<String>,
    /// certificate file, use only with server
    #[arg(
        short = 'c',
        long,
        value_name = "file",
        requires = "key",
        requires = "listen"
    )]
    cert: Option<String>,
    /// private key file
    #[arg(
        short,
        long,
        value_name = "file",
        requires = "cert",
        requires = "listen"
    )]
    key: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let (addr, port) = if let Some(port) = cli.listen {
        // If listen address is not provided, use localhost
        let addr = cli.addr.unwrap_or_else(|| "localhost".to_string());
        (addr, port)
    } else {
        if cli.addr.is_none() {
            eprintln!("address is required while connecting");
            return;
        }
        if cli.port.is_none() {
            eprintln!("port is required while connecting");
            return;
        }
        (cli.addr.unwrap(), cli.port.unwrap())
    };

    // Check if TLS is enabled
    let tls = if cli.ca.is_some() || cli.cert.is_some() && cli.key.is_some() {
        if cli.listen.is_some() && (cli.ca.is_none() || cli.cert.is_none() && cli.key.is_none()) {
            eprintln!("TLS requires CA, cert and key");
            return;
        }
        true
    } else {
        false
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();

    if cli.listen.is_some() {
        if tls {
            println!("Listening on {}:{} over TLS", addr, port);
            runtime.block_on(async {
                let ca = cli.ca.clone();
                let cert = cli.cert.unwrap();
                let key = cli.key.unwrap();
                tokio::select! {
                    r = tls::tls_server(&addr, port, ca.as_deref(), &cert, &key) => {
                        match r {
                            Ok(_) => {}
                            Err(e) => eprintln!("Err: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        } else {
            println!("Listening on {}:{}", addr, port);
            runtime.block_on(async {
                tokio::select! {
                    r = server::tcp_server(&addr, port) => {
                        match r {
                            Ok(_) => {}
                            Err(e) => eprintln!("Err: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        }
    } else {
        if tls {
            runtime.block_on(async {
                let ca = cli.ca.clone();
                tokio::select! {
                    r = tls::tls_client(&addr, port, ca.as_deref()) => {
                        match r {
                            Ok(_) => {}
                            Err(e) => eprintln!("Err: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        } else {
            runtime.block_on(async {
                tokio::select! {
                    r = client::tcp_client(&addr, port) => {
                        match r {
                            Ok(_) => {}
                            Err(e) => eprintln!("Err: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        }
        // client::client(&addr, port).await;
    }
    // Shutdown the runtime immediately after the client or server is done
    // The shutdown is required to prevent the runtime from waiting on the
    // stdin or stdout to be closed even after the client or server has
    // disconnected.
    runtime.shutdown_timeout(tokio::time::Duration::from_secs(0));
}
