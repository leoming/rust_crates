# ascii

A library that provides ASCII-only string and character types, equivalent to the
`char`, `str` and `String` types in the standard library.

Types and conversion traits are described in the [Documentation](https://docs.rs/ascii).

You can include this crate in your cargo project by adding it to the
dependencies section in `Cargo.toml`:

```toml
[dependencies]
ascii = "1.0"
```

## Using ascii without libstd

Most of `AsciiChar` and `AsciiStr` can be used without `std` by disabling the
default features. The owned string type `AsciiString` and the conversion trait
`IntoAsciiString` as well as all methods referring to these types and
`CStr` and `CString` are unavailable.
The `Error` trait is also unavailable, but `description()` is made
available as an inherent method for `ToAsciiCharError` and `AsAsciiStrError`.

To use the `ascii` crate in `core`-only mode in your cargo project just add the
following dependency declaration in `Cargo.toml`:

```toml
[dependencies]
ascii = { version = "1.0", default-features = false }
```

## Minimum supported Rust version

The minimum Rust version for 1.0.\* releases is 1.33.0.
Later 1.y.0 releases might require newer Rust versions, but the three most
recent stable releases at the time of publishing will always be supported.  
For example this means that if the current stable Rust version is 1.38 when
ascii 1.1.0 is released, then ascii 1.1.\* will not require a newer
Rust version than 1.36.

## History

This package included the Ascii types that were removed from the Rust standard
library by the 2014-12 [reform of the `std::ascii` module](https://github.com/rust-lang/rfcs/pull/486).
The API changed significantly since then.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
