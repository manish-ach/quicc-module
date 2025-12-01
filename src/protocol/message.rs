use anyhow::{Ok, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn write_header<W: AsyncWriteExt + Unpin>(
    mut writer: W,
    filename: &str,
    size: u64,
) -> Result<()> {
    let name_bytes = filename.as_bytes();
    writer.write_u16(name_bytes.len() as u16).await?;
    writer.write_all(name_bytes).await?;
    writer.write_u64(size).await?;
    Ok(())
}

pub async fn read_header<R: AsyncReadExt + Unpin>(
    mut reader: R) -> Result<(String, u64)> {
    let name_len = reader.read_u16().await?;
    let mut name = vec![0u8; name_len as usize];
    reader.read_exact(&mut name).await?;
    let filename = String::from_utf8(name)?;
    let size = reader.read_u64().await?;
    Ok((filename, size))
}
