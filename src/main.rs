use clap::Parser;

mod client;
mod common;
mod server;
mod tls;

#[derive(Parser, Debug)]
#[command(name = "netcat", author, version, long_about)]
struct Cli {
    /// Flag to use UDP
    #[arg(short, long)]
    udp: bool,
    /// Listen on a port
    #[arg(short, long, conflicts_with = "port", value_name = "port")]
    listen: Option<u16>,
    /// Address to connect or listen
    addr: Option<String>,
    /// Port to connect
    port: Option<u16>,
    /// certificate authority file
    #[arg(short = 'C', long, value_name = "file", conflicts_with = "udp")]
    ca: Option<String>,
    /// certificate file, use only with server
    #[arg(
        short = 'c',
        long,
        value_name = "file",
        requires = "key",
        requires = "listen",
        conflicts_with = "udp"
    )]
    cert: Option<String>,
    /// private key file
    #[arg(
        short,
        long,
        value_name = "file",
        requires = "cert",
        requires = "listen",
        conflicts_with = "udp"
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
    let udp = cli.udp;

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
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        } else if udp {
            println!("Listening on {}:{} over UDP", addr, port);
            runtime.block_on(async {
                tokio::select! {
                    r = server::udp_server(&addr, port) => {
                        match r {
                            Ok(_) => {}
                            Err(e) => eprintln!("Error: {}", e),
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
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        }
    } else {
        if tls {
            println!("Connecting to {}:{} over TLS", addr, port);
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
        } else if udp {
            println!("Connecting to {}:{} over UDP", addr, port);
            runtime.block_on(async {
                tokio::select! {
                    r = client::udp_client(&addr, port) => {
                        match r {
                            Ok(_) => {}
                            Err(e) => eprintln!("Err: {}", e),
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {}
                }
            });
        } else {
            println!("Connecting to {}:{}", addr, port);
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
