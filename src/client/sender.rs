use anyhow::Result;
use quinn::Endpoint;
use std::str::from_utf8;
use std::{net::SocketAddr, path::PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::insecure_client_config;
use super::write_header;

pub async fn send_file(addr: SocketAddr, path: PathBuf) -> Result<()> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    endpoint.set_default_client_config(insecure_client_config());

    let conn = endpoint.connect(addr, "localhost")?.await?;

    let filename = path.file_name().unwrap().to_str().unwrap();
    let meta = tokio::fs::metadata(&path).await?;
    let size = meta.len();

    let (mut send, mut recv) = conn.open_bi().await?;

    write_header(&mut send, filename, size).await?;

    let mut f = File::open(&path).await?;
    let mut buf = vec![0u8; 64 * 1024];

    loop {
        let n = f.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        send.write_all(&buf[..n]).await?;
    }

    send.finish()?;

    let mut ack = vec![0u8; 16];
    let n = recv.read(&mut ack).await?.unwrap();
    println!("ack: {}", from_utf8(&ack[..n])?);

    Ok(())
}
