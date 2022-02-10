// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

use std::io::Result;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

/// A suggested buffer size for IO operations; equal to 64Kb.
pub const SUGGESTED_BUFFER_SIZE: usize = 65536;

/// A buffered alternative to [`tokio::io::copy`] for high-throughput use cases.
pub async fn copy<R, W>(reader: &mut R, writer: &mut W, buf_size: usize) -> Result<()> 
    where R: AsyncRead + Unpin, W: AsyncWrite + Unpin
{
    let mut buf_reader = BufReader::with_capacity(buf_size, reader);
    tokio::io::copy_buf(&mut buf_reader, writer).await?;
    Ok(())
}