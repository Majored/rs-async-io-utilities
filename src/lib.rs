// Copyright (c) 2021 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-io-utilities/blob/main/LICENSE)

//! An asynchronous IO utilities crate powered by [`tokio`].
//!
//! ## Features
//! TODO
//!
//! [Read more.](https://github.com/Majored/rs-async-io-utilities)
 
pub(crate) mod readers;
pub(crate) mod writers;
pub(crate) mod functions;

pub use writers::offset_writer::AsyncOffsetWriter;

pub use readers::delimiter_reader::AsyncDelimiterReader;
pub use readers::prepend_reader::AsyncPrependReader;

pub use functions::copy::{copy, SUGGESTED_BUFFER_SIZE};