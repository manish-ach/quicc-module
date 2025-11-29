use anyhow::Result;
use quinn::Endpoint;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::{build_server_config, read_header};

pub async fn run_listener(port: u16) -> Result<()> {
    let cert = build_server_config()?;
    println!("Listening on 0.0.0.0:{port}");
    println!("Cert SHA256: {}", cert.fingerprint);

    let addr = ([0, 0, 0, 0], port).into();
    let endpoint = Endpoint::server(cert.server_config, addr)?;

    loop {
        let incoming = endpoint.accept().await;
        if incoming.is_none() {
            continue;
        }
        tokio::spawn(async move {
            match incoming.unwrap().await {
                Ok(conn) => {
                    if let Err(e) = handle_connection(conn).await {
                        eprintln!("conn error: {e:?}")
                    }
                }
                Err(e) => eprintln!("accept error: {e:?}"),
            }
        });
    }
}

async fn handle_connection(conn: quinn::Connection) -> Result<()> {
    while let Ok(bi) = conn.accept_bi().await {
        let (mut send, mut recv) = bi;

        let (filename, size) = read_header(&mut recv).await?;
        println!("Receiving {} ({} bytes)", filename, size);

        let mut file = File::create(format!("received-{filename}")).await?;
        let mut remaining = size;
        let mut buf = vec![0u8; 64 * 1024];

        while remaining > 0 {
            let to_read = remaining.min(buf.len() as u64) as usize;
            let n = recv.read(&mut buf[..to_read]).await?.unwrap_or(0);
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n]).await?;
            remaining -= n as u64;
        }

        send.write_all(b"OK").await?;
        send.finish()?;
    }

    Ok(())
}
