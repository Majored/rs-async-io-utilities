// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

use tokio::io::{AsyncRead, ReadBuf};

use std::pin::Pin;
use std::task::{Context, Poll};

// TODO: Handle buffer boundary (ie. a delimiter split over two separate calls to poll_read()).

/// A wrapper around an [`AsyncRead`] implementation that allows matching of a multi-byte delimiter.
///
/// Whilst the matched state of this wrapper is true, [`Self::poll_read()`] will return `Poll::Ready(Ok(()))`,
/// signifying EOF.
pub struct AsyncDelimiterReader<R: AsyncRead + Unpin> {
    inner: R,
    delimiter: Vec<u8>,
    buffer: Vec<u8>,
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
            inner,
            buffer: Vec::new(),
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
        self.inner
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncDelimiterReader <R> {
    fn poll_read(mut self: Pin<&mut Self>, c: &mut Context<'_>, b: &mut ReadBuf<'_>) -> Poll<tokio::io::Result<()>> {
        if self.matched {
            return Poll::Ready(Ok(()));
        }

        let prev_len = b.filled().len();

        // Append the local buffer first if it's not empty.
        if !self.buffer.is_empty() {
            b.put_slice(&self.buffer);
            self.buffer.clear();
        }

        let poll = Pin::new(&mut self.inner).poll_read(c, b);
        let new_len = b.filled().len();

        if new_len - prev_len == 0 {
            return poll;
        }

        let read_slice = &b.filled()[prev_len..new_len];

        if match_delimiter(&mut self, read_slice) {
            let actual_read_slice = &read_slice[self.delimiter.len()..];
            self.buffer.extend_from_slice(actual_read_slice);

            // TODO: This needs to also negate any data found after the delimiter.
            b.set_filled(b.filled().len() - self.delimiter.len());

            self.matched = true;
            return Poll::Ready(Ok(()));
        }

        poll
    }
}

fn match_delimiter<W: AsyncRead + Unpin>(reader: &mut AsyncDelimiterReader<W>, buf: &[u8]) -> bool {
    // A naive linear search along the buffer for the specified delimiter. This is already surprisingly performant.
    // 
    // For instance, using memchr::memchr() to match for the first byte of the delimiter, and then manual byte
    // comparisons for the remaining delimiter bytes was actually slower by a factor of 2.25. This method was explored
    // as tokio's `read_until()` implementation uses memchr::memchr(). Please submit an issue or PR if you know of
    // a better algorithm for this delimiter matching (and have tested/verified its performance).

    'outer: for index in 0..buf.len() {
        if index + reader.delimiter.len() >= buf.len() { 
            break;
        }

        for (delim_index, delim_byte) in reader.delimiter.iter().enumerate() {
            if buf[index + delim_index] != *delim_byte {
                continue 'outer;
            }
        }

        return true;
    }

    false
}
