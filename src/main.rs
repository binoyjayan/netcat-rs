use clap::Parser;

mod client;
mod server;

#[derive(Parser, Debug)]
#[command(name = "netcat", author, version = "1.0", long_about)]
struct Cli {
    /// Listen on a port
    #[arg(short, long)]
    listen: Option<u16>,
    /// Address to connect or listen
    addr: Option<String>,
    /// Port to connect
    port: Option<u16>,
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

    if cli.listen.is_some() {
        let r = server::run(&addr, port);
        match r {
            Ok(_) => println!("Listening on {}:{}", addr, port),
            Err(e) => eprintln!("{}:{} - {}", addr, port, e),
        }
    } else {
        let r = client::run(&addr, port);
        match r {
            Ok(_) => println!("Connected!"),
            Err(e) => eprintln!("{}:{} - {}", addr, port, e),
        }
    }
}
