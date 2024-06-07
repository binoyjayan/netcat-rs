use clap::Parser;

mod client;

#[derive(Parser)]
#[command(name = "netcat")]
#[command(version = "1.0")]
#[command(about = "netcat", long_about = None)]
struct Cli {
    addr: Option<String>,
    port: Option<u16>,
}

fn main() {
    let cli = Cli::parse();
    let addr = cli.addr.expect("Missing address");
    let port = cli.port.expect("Missing port");

    let r = client::run(&addr, port);
    match r {
        Ok(_) => println!("Connected!"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
