# CRAB

CRAB is an effort to try to audit the third-party crates we're pulling in.
`../vendor.py` will check for the existence of CRAB audits for each crate
version, and complain if there are versions missing.

Mechanically, every directory in `../vendor/` is expected to have a
corresponding TOML file in `crates/`. This TOML file describes the review that
was conducted by folks who landed the file. (Pedantically, the set of crates in
`../vendor/` that `../vendor.py` considers "empty" don't need CRAB reviews).

## How do I perform a CRAB audit?

The `crab-template.toml` file should hopefully be helpful, though please note
that it's somewhat aspirational in the keys it mentions. **Not every key in
`crab-template.toml` needs to be present in every audit.**

The keys we're currently asking folks to review for are:

- `code_is_not_malicious`
- `has_unsafe_code`
- `implements_crypto`

If you'd like to review for extra keys, thank you! The full set of supported
keys are listed in the CRAB file. That said, in order to keep this process
lightweight, we're only requiring those three are present at this time.

## Do I really need to do this for legacy crates?

For crates which are migrated as part of `dev-rust/` migrations, CRAB audits
_can_ be conducted, or we can choose to mark the to-be-reviewed crates as
legacy. gbiv@ can help with marking crates as legacy.

FIXME(b/240953811): Legacy crates should be minimized. Remove the above when
appropriate.
