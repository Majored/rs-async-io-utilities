// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

use std::io::{Error, IoSlice};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::AsyncWrite;

/// A wrapper around an [`AsyncWrite`] implementation which tracks the current byte offset.
pub struct AsyncOffsetWriter<W: AsyncWrite + Unpin> {
    inner: W,
    offset: usize,
}

impl<W: AsyncWrite + Unpin> AsyncOffsetWriter<W> {
    /// Constructs a new wrapper from an inner [`AsyncWrite`] writer.
    pub fn new(inner: W) -> Self {
        Self { inner, offset: 0 }
    }

    /// Constructs a new wrapper from an inner [`AsyncWrite`] writer and an initial offset.
    pub fn with_offset(inner: W, offset: usize) -> Self {
        Self { inner, offset }
    }

    /// Returns the current byte offset.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Consumes this wrapper and returns the inner [`AsyncWrite`] writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncOffsetWriter<W> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize, Error>> {
        let poll = Pin::new(&mut self.inner).poll_write(cx, buf);

        if let Poll::Ready(Ok(inner)) = &poll {
            self.offset += inner;
        }

        poll
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        bufs: &[IoSlice<'_>]
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut self.inner).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
}

#[cfg(test)]
#[tokio::test]
async fn basic() {
    use std::io::Cursor;
    use tokio::io::AsyncWriteExt;

    let mut writer = AsyncOffsetWriter::new(Cursor::new(Vec::new()));
    assert_eq!(writer.offset(), 0);

    writer.write_all(b"Foo. Bar. Foo. Bar.").await.expect("failed to write data");
    assert_eq!(writer.offset(), 19);

    writer.write_all(b"Foo. Foo.").await.expect("failed to write data");
    assert_eq!(writer.offset(), 28);

    writer.write_all(b"Bar. Bar.").await.expect("failed to write data");
    assert_eq!(writer.offset(), 37);
}