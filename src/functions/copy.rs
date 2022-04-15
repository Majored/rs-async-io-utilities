// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

use std::io::Result;
use tokio::io::{AsyncRead, AsyncWrite, BufReader, AsyncReadExt};

/// A suggested buffer size for IO operations; equal to 16Kb.
/// 
/// Note: This was originally 64Kb, however, it was causing a stack overflow for rs-async-zip on Windows in Debug mode.
pub const SUGGESTED_BUFFER_SIZE: usize = 16384;

/// A buffered alternative to [`tokio::io::copy`] for high-throughput use cases.
pub async fn copy<R, W>(reader: &mut R, writer: &mut W, buf_size: usize) -> Result<()> 
    where R: AsyncRead + Unpin, W: AsyncWrite + Unpin
{
    let mut buf_reader = BufReader::with_capacity(buf_size, reader);
    tokio::io::copy_buf(&mut buf_reader, writer).await?;
    Ok(())
}

/// Read and return a dynamic length string from a reader which impls AsyncRead.
pub async fn read_string<R: AsyncRead + Unpin>(reader: &mut R, length: usize) -> Result<String> {
    let mut buffer = String::with_capacity(length);
    reader.take(length as u64).read_to_string(&mut buffer).await?;

    Ok(buffer)
}

/// Read and return a dynamic length vector of bytes from a reader which impls AsyncRead.
pub async fn read_bytes<R: AsyncRead + Unpin>(reader: &mut R, length: usize) -> Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(length);
    reader.take(length as u64).read_to_end(&mut buffer).await?;

    Ok(buffer)
}