#![cfg_attr(rustfmt, rustfmt_skip)]

// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A library that provides ASCII-only string and character types, equivalent to the `char`, `str`
//! and `String` types in the standard library.
//!
//! Please refer to the readme file to learn about the different feature modes of this crate.
//!
//! # Minimum supported Rust version
//!
//! The minimum Rust version for 1.0.\* releases is 1.33.0.
//! Later 1.y.0 releases might require newer Rust versions, but the three most
//! recent stable releases at the time of publishing will always be supported.  
//! For example this means that if the current stable Rust version is 1.38 when
//! ascii 1.1.0 is released, then ascii 1.1.* will not require a newer
//! Rust version than 1.36.
//!
//! # History
//!
//! This package included the Ascii types that were removed from the Rust standard library by the
//! 2014-12 [reform of the `std::ascii` module](https://github.com/rust-lang/rfcs/pull/486). The
//! API changed significantly since then.

#![cfg_attr(not(feature = "std"), no_std)]

#![allow(clippy::trivially_copy_pass_by_ref)] // for compatibility with methods on char and u8

#[cfg(feature = "std")]
extern crate core;

#[cfg(feature = "serde")]
extern crate serde;

#[cfg(all(test, feature = "serde_test"))]
extern crate serde_test;

mod ascii_char;
mod ascii_str;
#[cfg(feature = "std")]
mod ascii_string;
mod free_functions;
#[cfg(feature = "serde")]
mod serialization;

pub use ascii_char::{AsciiChar, ToAsciiChar, ToAsciiCharError};
pub use ascii_str::{AsciiStr, AsAsciiStr, AsMutAsciiStr, AsAsciiStrError};
pub use ascii_str::{Chars, CharsMut, CharsRef};
#[cfg(feature = "std")]
pub use ascii_string::{AsciiString, IntoAsciiString, FromAsciiError};
pub use free_functions::{caret_encode, caret_decode};
