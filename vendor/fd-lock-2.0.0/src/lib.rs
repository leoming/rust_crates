//! Advisory cross-platform file locks using file descriptors.
//!
//! Note that advisory lock compliance is opt-in, and can freely be ignored by other parties. This
//! means this crate __should not be relied on for security__, but solely used to coordinate file
//! access.
//!
//! ## Example
//! ```rust
//! use fd_lock::FdLock;
//! # use tempfile::tempfile;
//! # use std::io::{self, prelude::*};
//! # use std::fs::File;
//!
//! # fn main() -> io::Result<()> {
//! // Lock a file and write to it.
//! let mut f = FdLock::new(tempfile()?);
//! f.try_lock()?.write_all(b"chashu cat")?;
//!
//! // Locks can also be held for extended durations.
//! let mut f = f.try_lock()?;
//! f.write_all(b"nori cat")?;
//! f.write_all(b"bird!")?;
//! # Ok(())}
//! ```

#![forbid(future_incompatible)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(
    missing_docs,
    missing_doc_code_examples,
    unreachable_pub,
    rust_2018_idioms
)]

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;
