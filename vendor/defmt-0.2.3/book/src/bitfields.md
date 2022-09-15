# Bitfields

`:m..n` is the bitfield formatting parameter.
When paired with a positional parameter it can be used to display the bitfields of a register.

``` rust
# extern crate defmt;
# let pcnf1 = 0u32;
// -> TRACE: PCNF1 { MAXLEN: 125, STATLEN: 3, BALEN: 2 }
defmt::trace!(
    "PCNF1: {{ MAXLEN: {0=0..8}, STATLEN: {0=8..16}, BALEN: {0=16..19} }}",
    //                  ^                  ^                 ^ same argument
    pcnf1, // <- type must be `u32`
);
```

The bitfield argument is expected to be a *fully typed, unsigned* integer that's large enough to contain the bitfields.
For example, if bitfield ranges only cover up to bit `11` (e.g. `=8..12`) then the argument must be `u16`.

Bit indices are little-endian: the 0th bit is the rightmost bit.

Bitfields are not range inclusive, e.g.
``` rust
# extern crate defmt;
defmt::trace!("first two bits: {0=0..3}", 254u32);
```
will evaluate to `2` (`0b10`).

⚠️ You can not reuse the same argument in a bitfield- and a non bitfield parameter. This will not compile:
``` rust,compile_fail
# extern crate defmt;
defmt::trace!("{0=5..13} {0=u16}", 256u16);
```
