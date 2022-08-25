Vendored Rust Libraries
===

This repository contains vendored Rust libraries for the system bluetooth
project, and for ChromeOS' `dev-rust/third-party-crates-src` ebuild. It is
generated by running `cargo vendor` to store all our dependencies.

Please see the individuals listed in the OWNERS file if you'd like to learn more
about this repo.

# Updating packages

In order to update or add any package, follow these steps:

* Find the project in `projects/` corresponding to the first-party package you'd
  like to update. If it does not exist, please see the "Adding a first-party
  package," section.
* Modify its `Cargo.toml` to add, remove or upgrade packages.
* Run `python vendor.py`
    * This runs `cargo vendor` first, which updates `Cargo.lock` and puts
      downloaded crates into the `vendor` directory
    * It applies any patches in the `patches` directory. It also regenerates
      checksums for packages that were modified.
    * It removes OWNER files from packages that have it (interferes with our own
      OWNERS management) and regenerates checksums
    * It checks that all packages have a supported license and lists the set of
      licenses used by the crates.
    * If `--license-map=<filename>` is given, it will dump a json file which is
      a dictionary with the crate names as keys and another dictionary with the
      `license` and `license_file` as keys.
* Verify that no patches need to be updated. Patches in `patches/` can either
  be named after a crate (in which case, they apply to _all_ versions of their
  corresponding crate), or can have a version in their name, in which case
  they're meant to apply specifically to a given version of a crate.
* If any licenses are unsupported, do the following:
    * Check if the package is actually included in the build. `cargo vendor`
      seems to also pick up dependencies for unused configs (i.e. windows). You
      will need to make sure these packages are stripped by `cargo vendor`.
    * Check if the license file exists in the crate's repository. Sometimes the
      crate authors just forget to list it in Cargo.toml. In this case, add
      a new patch to apply the license to the crate and send a patch upstream to
      include the license in the crate as well.
    * Check if the license is permissive and ok with ChromeOS. If you don't know
      how to do this, reach out to the OWNERS of this repo.
      * Yes: Update the `vendor.py` script with the new license and also update
        `net-wireless/floss-9999.ebuild` with the new license.
      * No: Do not use this package. Contact the OWNERS of this repo for next
        steps.

## Adding patches

When it is necessary to patch a package due to incompatibility, you can create
a patch targeting the specific package and store it in
`patches/`. For any given `${crate}` at `${version}`, if
`patches/${crate}-${version}` exists, the patches from that directory are
applied to the crate. If such a directory does not exist, `patches/${crate}` is
checked. Similarly, if this exists, patches are applied; otherwise, the crate
is left unpatched.

If `./vendor.py` complains about a specific directory in `patches/` not having
a corresponding `vendor/` directory, the most likely fixes are:

* the crate is no longer required, and the patches should be deleted.
* the crate has been upgraded, and the patches that were previously applied
  should be evaluated for whether they are still relevant.

Patches can come in two forms. Files with names ending in `.patch` are always applied with
`patch -p1` to the vendor directory. Executable files that do not have names
ending in `patch` will be executed in the vendor directory which they should
apply to. All other files are ignored.

## Testing updates

Updates to this repo will be captured by the CQ. To directly test changes,
either build the `net-wireless/floss` package, or run
`cros_workon --board=${BOARD} dev-rust/third-party-crates-src` and build
packages + run tests for your board of choice.

## Adding a first-party package

The `packages/` subdirectory contains a set of `Cargo.toml` files that roughly
correspond to what exists in the ChromeOS tree. These exist only to provide
dependency information with which we can create our `vendor/` directory, so
everything is removed except for author information, dependencies, and (if
necessary) a minimal set of features. Dependency sections outside of what would
build for ChromeOS, like `[target.'cfg(windows)'.dependencies]`, are also
removed.

Admittedly, it's sort of awkward to have two `Cargo.toml`s for each first-party
project. It may be worth trying to consolidate this in the future, though our
future bazel migration potentially influences what the 'ideal' setup here is.
