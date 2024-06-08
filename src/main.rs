use clap::Parser;

mod client;
mod server;

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
}

#[tokio::main]
async fn main() {
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

    if cli.listen.is_some() {
        println!("Listening on {}:{}", addr, port);
        let r = server::server(&addr, port).await;
        match r {
            Ok(_) => {}
            Err(e) => eprintln!("{}:{} - {}", addr, port, e),
        }
    } else {
        let r = client::client(&addr, port).await;
        match r {
            Ok(_) => {}
            Err(e) => eprintln!("{}:{} - {}", addr, port, e),
        }
    }
}
