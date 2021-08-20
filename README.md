WIP: Vendored Rust Libraries
===

This repository contains vendored Rust libraries for the system bluetooth
project. It is currently in an experimental state where we are simply running
`cargo vendor` to store all our dependencies.

Please reach out to the individuals listed in the OWNERS file if you'd like to
know more about this repo.

# Updating packages

In order to update any package, follow these steps:

* Modify `Cargo.toml` to add, remove or upgrade packages.
* Run `python vendor.py`
    * This runs `cargo vendor` first, which updates `Cargo.lock` and puts
      downloaded crates into the `vendor` directory
    * It applies any patches in the `patches` directory. It also regenerates
      checksums for packages that were modified.
    * It removes OWNER files from packages that have it (interferes with our own
      OWNERS management) and regenerates checksums

## Adding patches

When it is necessary to patch a package due to incompatibility, you can create
a patch targetting the specific package and store it in
`patches/${package_name}/` with the extension `.patch`. The patch will be
applied when you run `vendor.py`.

## Testing updates

Updates to this repo will be captured by CQ (currently zork-floss-cq). To
directly test changes, build the `net-wireless/floss` package (it is only
available on the zork-floss board right now).
