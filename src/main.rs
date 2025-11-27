mod config;
// mod server;
// mod client;
// mod protocol;

use anyhow::{Ok, Result};
use std::{net::SocketAddr, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() < 2 {
        eprintln!(
            "usage:\n {} listen <port>\n {} send <addr:port> <file>",
            args[0], args[0]
        );
        std::process::exit(1);
    }

    match args[1].as_str() {
        "listen" => {
            let port = args
                .get(2)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(5000);
            // server::run_listener(port).await?;
        }
        "send" => {
            if args.len() < 4 {
                anyhow::bail!("send requires <addr:port> <file>");
            }
            let addr: SocketAddr = args[2].parse()?;
            let file = PathBuf::from(&args[3]);
            // client::send_file(addr, file).await?;
        }
        _ => eprintln!("unknown command"),
    }

    Ok(())
}
