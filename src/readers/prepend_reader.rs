// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

use tokio::io::{AsyncRead, ReadBuf};

use std::pin::Pin;
use std::task::{Context, Poll};

/// A wrapper around an [`AsyncRead`] implementation that allows prepending bytes from a buffer during reads.
pub struct AsyncPrependReader<R: AsyncRead + Unpin> {
    inner: R,
    buffer: Vec<u8>,
}

impl<R: AsyncRead + Unpin> AsyncPrependReader<R> {
    /// Constructs a new wrapper from an inner [`AsyncRead`] implementation.
    pub fn new(inner: R) -> Self {
        Self { inner, buffer: Vec::new() }
    }

    /// Prepends data to this buffer.
    pub fn prepend(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Returns a shared reference to the buffer.
    pub fn buffer(&self) -> &Vec<u8> {
        &self.buffer
    }

    /// Returns a mutable reference to the buffer.
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buffer
    }

    /// Consumes this wrapper and returns the inner [`AsyncRead`] reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncPrependReader<R> {
    fn poll_read(mut self: Pin<&mut Self>, c: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<tokio::io::Result<()>> {
        let buffer_filled = !self.buffer.is_empty();

        if buffer_filled {
            buf.put_slice(&self.buffer);
            self.buffer.clear();
        }
    
        let poll = Pin::new(&mut self.inner).poll_read(c, buf);

        if let Poll::Ready(Err(_)) = &poll {
            return poll;
        } 
        
        if buffer_filled {
            Poll::Ready(Ok(()))
        } else {
            poll
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn prepend_only() {
    use std::io::Cursor;
    use tokio::io::AsyncReadExt;

    let data = b"";

    let cursor = Cursor::new(data);
    let mut buffer = String::new();
    let mut reader = AsyncPrependReader::new(cursor);

    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, "");

    reader.prepend(b"Foo. Foo. Foo. Foo.");
    buffer.clear();

    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, "Foo. Foo. Foo. Foo.");
    assert!(reader.buffer().is_empty());

    reader.prepend(b"Bar. Bar. Bar. Bar.");
    buffer.clear();

    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, "Bar. Bar. Bar. Bar.");
    assert!(reader.buffer().is_empty());
}