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
    * It checks that all packages have a supported license and lists the set of
      licenses used by the crates.
    * If `--license-map=<filename>` is given, it will dump a json file which is
      a dictionary with the crate names as keys and another dictionary with the
      `license` and `license_file` as keys.
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
a patch targetting the specific package and store it in
`patches/${package_name}/` with the extension `.patch`. The patch will be
applied when you run `vendor.py`.

## Testing updates

Updates to this repo will be captured by CQ (currently zork-floss-cq). To
directly test changes, build the `net-wireless/floss` package.
