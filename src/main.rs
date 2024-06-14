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
    /// private key file, optional for client
    #[arg(
        short,
        long,
        value_name = "keyfile",
        requires = "cert",
        conflicts_with = "udp"
    )]
    key: Option<String>,
    /// certificate file, optional for client
    #[arg(
        short = 'c',
        long,
        value_name = "certfile",
        requires = "key",
        conflicts_with = "udp"
    )]
    cert: Option<String>,
    /// Flag to enable client verification (mTLS)
    #[arg(
        long,
        requires = "listen",
        requires = "cert",
        requires = "key",
        conflicts_with = "udp"
    )]
    client_auth: bool,
    #[arg(
        short = 'e',
        long,
        value_name = "shell",
        conflicts_with = "udp",
        conflicts_with = "ca"
    )]
    shell: Option<String>,
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
    let tls = if cli.listen.is_some() {
        cli.key.is_some() && cli.cert.is_some()
    } else {
        // key and cert is optional for client
        cli.ca.is_some()
    };
    let udp = cli.udp;

    let runtime = tokio::runtime::Runtime::new().unwrap();

    #[allow(clippy::collapsible_else_if)]
    if cli.listen.is_some() {
        if tls {
            if cli.client_auth {
                println!(
                    "Listening on {}:{} over mTLS [client verification on]",
                    addr, port
                );
            } else {
                println!(
                    "Listening on {}:{} over TLS [client verification off]",
                    addr, port
                );
            }
            runtime.block_on(async {
                let ca = cli.ca.clone();
                let key = cli.key.unwrap();
                let cert = cli.cert.unwrap();
                tokio::select! {
                    r = tls::tls_server(&addr, port, ca.as_deref(), &key, &cert, cli.client_auth) => {
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
            if let Some(shell) = cli.shell {
                println!("Listening on {}:{} with shell: {}", addr, port, shell);
                runtime.block_on(async {
                    tokio::select! {
                        r = server::tcp_shell(&addr, port, &shell) => {
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
        }
    } else {
        if tls {
            if cli.key.is_some() && cli.cert.is_some() {
                println!(
                    "Connecting to {}:{} over mTLS [client verification on]",
                    addr, port
                );
            } else {
                println!(
                    "Connecting to {}:{} over TLS [client verification off]",
                    addr, port
                );
            }

            runtime.block_on(async {
                let ca = cli.ca.clone();
                tokio::select! {
                    r = tls::tls_client(&addr, port, ca.as_deref(), cli.key.as_deref(), cli.cert.as_deref()) => {
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
            if let Some(shell) = cli.shell {
                println!("Connecting to {}:{} with shell: {}", addr, port, shell);
                runtime.block_on(async {
                    tokio::select! {
                        r = client::tcp_reverse_shell(&addr, port, &shell) => {
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
        }
        // client::client(&addr, port).await;
    }
    // Shutdown the runtime immediately after the client or server is done
    // The shutdown is required to prevent the runtime from waiting on the
    // stdin or stdout to be closed even after the client or server has
    // disconnected.
    runtime.shutdown_timeout(tokio::time::Duration::from_secs(0));
}
