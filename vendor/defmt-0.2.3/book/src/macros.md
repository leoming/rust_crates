# Logging macros

Logging is done using the `error`, `warn`, `info`, `debug` and `trace` macros.
Each macro logs at the logging level indicated in its name.
The syntax of these macros is roughly the same as the `println` macro.
Positional parameters are supported but named parameters are not.
Escaping rules are the same: the characters `{` and `}` are escaped as `{{` and `}}`.
The biggest difference is in the supported formatting parameters (`:?`, `:>4`, `:04`).

``` rust
# extern crate defmt;
# let len = 80u8;
// -> INFO:  message arrived (length=80)
defmt::info!(
    "message arrived (length={})",
    len /*: usize */,
);

# struct Message;
# impl Message { fn header(&self) -> u8 { 0 } }
# let message = Message;
// -> DEBUG: Header { source: 2, destination: 3, sequence: 16 }
defmt::debug!("{:?}", message.header() /*: Header */);
```

Unlike `core::fmt` which has several formatting traits (`Debug`, `Display`), `defmt` has a single formatting trait called `Format`.
The `{}` formatting parameter indicates that the `Format` trait will be used.
When `{}` is used the corresponding argument must implement the `Format` trait.

``` rust
# extern crate defmt;
# let x = 0;
defmt::trace!("{}", x);
//                  ^ must implement the `Format` trait
```
