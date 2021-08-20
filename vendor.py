#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Copyright 2021 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
""" This script cleans up the vendor directory.
"""
import hashlib
import json
import os
import pathlib
import subprocess


def _rerun_checksums(package_path):
    """Re-run checksums for given package.

    Writes resulting checksums to $package_path/.cargo-checksum.json.
    """
    hashes = {}
    checksum_path = os.path.join(package_path, '.cargo-checksum.json')
    if not pathlib.Path(checksum_path).is_file():
        return False

    with open(checksum_path, 'r') as fread:
        contents = json.load(fread)

    for root, _, files in os.walk(package_path, topdown=True):
        for f in files:
            # Don't checksum an existing checksum file
            if f == ".cargo-checksum.json":
                continue

            file_path = os.path.join(root, f)
            with open(file_path, 'rb') as frb:
                m = hashlib.sha256()
                m.update(frb.read())
                d = m.hexdigest()

                # Key is relative to the package path so strip from beginning
                key = os.path.relpath(file_path, package_path)
                hashes[key] = d

    if hashes:
        print("{} regenerated {} hashes".format(package_path, len(hashes.keys())))
        contents['files'] = hashes

        with open(checksum_path, 'w') as fwrite:
            json.dump(contents, fwrite)

    return True


def _remove_OWNERS_checksum(root):
    """ Delete all OWNERS files from the checksum file.

    Args:
        root: Root directory for the vendored crate.

    Returns:
        True if OWNERS was found and cleaned up. Otherwise False.
    """
    checksum_path = os.path.join(root, '.cargo-checksum.json')
    if not pathlib.Path(checksum_path).is_file():
        return False

    with open(checksum_path, 'r') as fread:
        contents = json.load(fread)

    del_keys = []
    for cfile in contents['files']:
        if 'OWNERS' in cfile:
            del_keys.append(cfile)

    for key in del_keys:
        del contents['files'][key]

    if del_keys:
        print('{} deleted: {}'.format(root, del_keys))
        with open(checksum_path, 'w') as fwrite:
            json.dump(contents, fwrite)

    return bool(del_keys)


def cleanup_owners(vendor_path):
    """ Remove owners checksums from the vendor directory.

    We currently do not check in the OWNERS files from vendored crates because
    they interfere with the find-owners functionality in gerrit. This cleanup
    simply finds all instances of "OWNERS" in the checksum files within and
    removes them.

    Args:
        vendor_path: Absolute path to vendor directory.
    """
    deps_cleaned = []
    for root, dirs, _ in os.walk(vendor_path):
        for d in dirs:
            removed = _remove_OWNERS_checksum(os.path.join(root, d))
            if removed:
                deps_cleaned.append(d)

    if deps_cleaned:
        print('Cleanup owners:\n {}'.format("\n".join(deps_cleaned)))


def apply_single_patch(patch, workdir):
    """Apply a single patch and return whether it was successful.

    Returns:
        True if successful. False otherwise.
    """
    print("-- Applying {}".format(patch))
    proc = subprocess.run(["patch", "-p1", "-i", patch], cwd=workdir)
    return proc.returncode == 0


def apply_patches(patches_path, vendor_path):
    """Finds patches and applies them to sub-folders in the vendored crates.

    Args:
        patches_path: Path to folder with patches. Expect all patches to be one
                    level down (matching the crate name).
        vendor_path: Root path to vendored crates directory.
    """
    checksums_for = {}

    # Don't bother running if patches directory is empty
    if not pathlib.Path(patches_path).is_dir():
      return

    # Look for all patches and apply them
    for d in os.listdir(patches_path):
        dir_path = os.path.join(patches_path, d)

        # We don't process patches in root dir
        if not os.path.isdir(dir_path):
            continue

        for patch in os.listdir(os.path.join(dir_path)):
            file_path = os.path.join(dir_path, patch)

            # Skip if not a patch file
            if not os.path.isfile(file_path) or not patch.endswith(".patch"):
                continue

            # If there are any patches, queue checksums for that folder.
            checksums_for[d] = True

            # Apply the patch. Exit from patch loop if patching failed.
            success = apply_single_patch(file_path,
                                         os.path.join(vendor_path, d))
            if not success:
                print("Failed to apply patch: {}".format(patch))
                break


    # Re-run checksums for all modified packages since we applied patches.
    for key in checksums_for.keys():
        _rerun_checksums(os.path.join(vendor_path, key))

def run_cargo_vendor(working_dir):
    """Runs cargo vendor.

    Args:
        working_dir: Directory to run inside. This should be the directory where
                    Cargo.toml is kept.
    """
    subprocess.check_call(["cargo", "vendor"], cwd=working_dir)

def main():
    current_path = pathlib.Path(__file__).parent.absolute()
    patches = os.path.join(current_path, "patches")
    vendor = os.path.join(current_path, "vendor")

    # First, actually run cargo vendor
    run_cargo_vendor(current_path)

    # Order matters here:
    # - Apply patches (also re-calculates checksums)
    # - Cleanup any owners files (otherwise, git check-in or checksums are
    #   unhappy)
    apply_patches(patches, vendor)
    cleanup_owners(vendor)


if __name__ == '__main__':
    main()
