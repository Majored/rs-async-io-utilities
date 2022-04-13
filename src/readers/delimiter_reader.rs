// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

use crate::readers::prepend_reader::AsyncPrependReader;
use tokio::io::{AsyncRead, ReadBuf};

use std::pin::Pin;
use std::task::{Context, Poll};

/// A wrapper around an [`AsyncRead`] implementation that allows matching of a multi-byte delimiter.
///
/// Whilst the matched state of this wrapper is true, [`Self::poll_read()`] will return `Poll::Ready(Ok(()))`,
/// signifying EOF.
pub struct AsyncDelimiterReader<R: AsyncRead + Unpin> {
    inner: AsyncPrependReader<R>,
    delimiter: Vec<u8>,
    matched: bool,
}

impl<R: AsyncRead + Unpin> AsyncDelimiterReader <R> {
    /// Constructs a new wrapper from an inner [`AsyncRead`] implementation and a byte slice delimiter.
    pub fn new(inner: R, delimiter: &[u8]) -> Self {
        Self::with_owned_delimiter(inner, delimiter.to_vec())
    }

    /// Constructs a new wrapper from an inner [`AsyncRead`] implementation and a byte vector delimiter.
    pub fn with_owned_delimiter(inner: R, delimiter: Vec<u8>) -> Self {
        if delimiter.len() == 0 {
            panic!("Delimiter byte length must be non-zero.");
        }

        Self {
            inner: AsyncPrependReader::new(inner),
            delimiter,
            matched: false,
        }
    }

    /// Returns whether or not the specified delimiter has been reached.
    pub fn matched(&self) -> bool {
        self.matched
    }

    /// Resets the matched state of this wrapper.
    pub fn reset(&mut self) {
        self.matched = false;
    }

    /// Consumes this wrapper and returns the inner [`AsyncRead`] reader.
    pub fn into_inner(self) -> R {
        self.inner.into_inner()
    }

    /// Returns a mutable reference to the inner reader.
    pub fn get_mut(&mut self) -> &mut R {
        self.inner.get_mut()
    }

    /// Returns a shared reference to the inner buffer.
    pub fn buffer(&self) -> &[u8] {
        self.inner.buffer()
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncDelimiterReader <R> {
    fn poll_read(mut self: Pin<&mut Self>, c: &mut Context<'_>, b: &mut ReadBuf<'_>) -> Poll<tokio::io::Result<()>> {
        if self.matched {
            return Poll::Ready(Ok(()));
        }

        let (prev_len, new_len, poll) = fill_buffer(&mut self, c, b);

        if new_len - prev_len == 0 {
            return poll;
        }

        let read_slice = &b.filled()[prev_len..new_len];

        if let Some((full_match, match_index)) = match_delimiter(&mut self, read_slice) {
            let read_slice_len = read_slice.len();
            let index = match_index + self.delimiter.len();
            if read_slice_len >= index {
                let actual_read_slice = &read_slice[index..];
                self.inner.prepend(actual_read_slice);
                b.set_filled(match_index);
            }

            if full_match {
                self.matched = true;
                return Poll::Ready(Ok(()));
            }
        }

        poll
    }
}

fn fill_buffer<W: AsyncRead + Unpin>(
    reader: &mut Pin<&mut AsyncDelimiterReader<W>>, c: &mut Context<'_>, b: &mut ReadBuf<'_>
) -> (usize, usize, Poll<tokio::io::Result<()>>) {
    let prev_len = b.filled().len();

    let poll = Pin::new(&mut reader.inner).poll_read(c, b);
    let new_len = b.filled().len();

    (prev_len, new_len, poll)
}

fn match_delimiter<W: AsyncRead + Unpin>(reader: &mut AsyncDelimiterReader<W>, buf: &[u8]) -> Option<(bool, usize)> {
    // A naive linear search along the buffer for the specified delimiter. This is already surprisingly performant.
    // 
    // For instance, using memchr::memchr() to match for the first byte of the delimiter, and then manual byte
    // comparisons for the remaining delimiter bytes was actually slower by a factor of 2.25. This method was explored
    // as tokio's `read_until()` implementation uses memchr::memchr(). Please submit an issue or PR if you know of
    // a better algorithm for this delimiter matching (and have tested/verified its performance).

    'outer: for index in 0..buf.len() {
        for (delim_index, delim_byte) in reader.delimiter.iter().enumerate() {
            if index + delim_index >= buf.len() && delim_index != 0 { 
                // We have a partial match but have hit the end of the buffer.
                return Some((false, index));
            }

            if buf[index + delim_index] != *delim_byte {
                continue 'outer;
            }
        }

        return Some((true, index));
    }

    None
}

#[cfg(test)]
#[tokio::test]
async fn foo_bar() {
    use std::io::Cursor;
    use tokio::io::AsyncReadExt;

    let data = b"Foo. Bar. Foo. Foo. Bar. Foo. Bar.";
    let delimiter = b"Bar.";

    let cursor = Cursor::new(data);
    let mut buffer = String::new();
    let mut reader = AsyncDelimiterReader::new(cursor, delimiter);

    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, "Foo. ");
    assert!(reader.matched());
    reader.reset();
    buffer.clear();

    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, " Foo. Foo. ");
    assert!(reader.matched());
    reader.reset();
    buffer.clear();

    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, " Foo. ");
    assert!(reader.matched());
    reader.reset();
    buffer.clear();
    
    reader.read_to_string(&mut buffer).await.expect("failed to read");
    assert_eq!(buffer, "");
    assert!(!reader.matched());
}