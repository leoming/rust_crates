#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Copyright 2021 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
""" This script cleans up the vendor directory.
"""
import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess

# We only care about crates we're actually going to use and that's usually
# limited to ones with cfg(linux). For running `cargo metadata`, limit results
# to only this platform
DEFAULT_PLATFORM_FILTER = "x86_64-unknown-linux-gnu"


def _rerun_checksums(package_path):
    """Re-run checksums for given package.

    Writes resulting checksums to $package_path/.cargo-checksum.json.
    """
    hashes = dict()
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
        print("{} regenerated {} hashes".format(package_path,
                                                len(hashes.keys())))
        contents['files'] = hashes
        with open(checksum_path, 'w') as fwrite:
            json.dump(contents, fwrite, sort_keys=True)

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
            json.dump(contents, fwrite, sort_keys=True)

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

def load_metadata(working_dir, filter_platform=DEFAULT_PLATFORM_FILTER):
    """Load metadata for manifest at given directory.

    Args:
        working_dir: Directory to run from.
        filter_platform: Filter packages to ones configured for this platform.
    """
    manifest_path = os.path.join(working_dir, 'Cargo.toml')
    cmd = [
        'cargo', 'metadata', '--format-version', '1', "--filter-platform",
        filter_platform, '--manifest-path', manifest_path
    ]
    output = subprocess.check_output(cmd, cwd=working_dir)

    return json.loads(output)


class LicenseManager:
    """ Manage consolidating licenses for all packages."""

    # These are all the licenses we support. Keys are what is seen in metadata and
    # values are what is expected by the ebuild.
    SUPPORTED_LICENSES = {
        'Apache-2.0': 'Apache-2.0',
        'MIT': 'MIT',
        'BSD-3-Clause': 'BSD-3',
        'ISC': 'ISC'
    }

    # Prefer to take attribution licenses in this order. All these require that
    # we actually use the license file found in the package so they MUST have
    # a license file set.
    PREFERRED_ATTRIB_LICENSE_ORDER = ['MIT', 'BSD-3', 'ISC']

    # If Apache license is found, always prefer it (simplifies attribution)
    APACHE_LICENSE = 'Apache-2.0'

    # Regex for license files found in the vendored directories. Search for
    # these files with re.IGNORECASE.
    #
    # These will be searched in order with the earlier entries being preferred.
    LICENSE_NAMES_REGEX = [
        r'^license-mit$',
        r'^copyright$',
        r'^licen[cs]e.*$',
    ]

    # Some crates have their license file in other crates. This usually occurs
    # because multiple crates are published from the same git repository and the
    # license isn't updated in each sub-crate. In these cases, we can just
    # ignore these packages.
    MAP_LICENSE_TO_OTHER = {
        'failure_derive': 'failure',
        'grpcio-compiler': 'grpcio',
        'grpcio-sys': 'grpcio',
        'rustyline-derive': 'rustyline',
    }

    # Map a package to a specific license and license file. Only use this if
    # a package doesn't have an easily discoverable license or exports its
    # license in a weird way. Prefer to patch the project with a license and
    # upstream the patch instead.
    STATIC_LICENSE_MAP = {
        # "package name": ( "license name", "license file relative location")
    }

    def __init__(self, working_dir, vendor_dir):
        self.working_dir = working_dir
        self.vendor_dir = vendor_dir

    def _find_license_in_dir(self, search_dir):
        for p in os.listdir(search_dir):
            # Ignore anything that's not a file
            if not os.path.isfile(os.path.join(search_dir, p)):
                continue

            # Now check if the name matches any of the regexes
            # We'll return the first matching file.
            for regex in self.LICENSE_NAMES_REGEX:
                if re.search(regex, p, re.IGNORECASE):
                    yield os.path.join(search_dir, p)
                    break

    def _guess_license_type(self, license_file):
        if '-MIT' in license_file:
            return 'MIT'
        elif '-APACHE' in license_file:
            return 'APACHE'
        elif '-BSD' in license_file:
            return 'BSD-3'

        with open(license_file, 'r') as f:
            lines = f.read()
            if 'MIT' in lines:
                return 'MIT'
            elif 'Apache' in lines:
                return 'APACHE'
            elif 'BSD 3-Clause' in lines:
                return 'BSD-3'

        return ''

    def generate_license(self, skip_license_check, print_map_to_file):
        """Generate single massive license file from metadata."""
        metadata = load_metadata(self.working_dir)

        has_license_types = set()
        bad_licenses = {}

        # Keep license map ordered so it generates a consistent license map
        license_map = {}

        skip_license_check = skip_license_check or []

        for package in metadata['packages']:
            pkg_name = package['name']

            # Skip vendor libs directly
            if pkg_name == "vendor_libs":
                continue

            if pkg_name in skip_license_check:
                print(
                    "Skipped license check on {}. Reason: Skipped from command line"
                    .format(pkg_name))
                continue

            if pkg_name in self.MAP_LICENSE_TO_OTHER:
                print(
                    'Skipped license check on {}. Reason: License already in {}'
                    .format(pkg_name, self.MAP_LICENSE_TO_OTHER[pkg_name]))
                continue

            # Check if we have a static license map for this package. Use the
            # static values if we have it already set.
            if pkg_name in self.STATIC_LICENSE_MAP:
                (license, license_file) = self.STATIC_LICENSE_MAP[pkg_name]
                license_map[pkg_name] = {
                    "license": license,
                    "license_file": license_file,
                }
                continue

            license_files = []
            license = package.get('license', '')

            # We ignore the metadata for license file because most crates don't
            # have it set. Just scan the source for licenses.
            license_files = [
                x for x in self._find_license_in_dir(
                    os.path.join(self.vendor_dir, pkg_name))
            ]

            # If there are multiple licenses, they are delimited with "OR" or "/"
            delim = ' OR ' if ' OR ' in license else '/'
            found = license.split(delim)

            # Filter licenses to ones we support
            licenses_or = [
                self.SUPPORTED_LICENSES[f] for f in found
                if f in self.SUPPORTED_LICENSES
            ]

            # If apache license is found, always prefer it because it simplifies
            # license attribution (we can use existing Apache notice)
            if self.APACHE_LICENSE in licenses_or:
                has_license_types.add(self.APACHE_LICENSE)
                license_map[pkg_name] = {'license': self.APACHE_LICENSE}

            # Handle single license that has at least one license file
            # We pick the first license file and the license
            elif len(licenses_or) == 1:
                if license_files:
                    l = licenses_or[0]
                    lf = license_files[0]

                    has_license_types.add(l)
                    license_map[pkg_name] = {
                        'license': l,
                        'license_file': os.path.relpath(lf, self.working_dir),
                    }
                else:
                    bad_licenses[pkg_name] = "{} missing license file".format(
                        licenses_or[0])
            # Handle multiple licenses
            elif len(licenses_or) > 1:
                # Check preferred licenses in order
                license_found = False
                for l in self.PREFERRED_ATTRIB_LICENSE_ORDER:
                    if not l in licenses_or:
                        continue

                    for f in license_files:
                        if self._guess_license_type(f) == l:
                            license_found = True
                            has_license_types.add(l)
                            license_map[pkg_name] = {
                                'license':
                                l,
                                'license_file':
                                os.path.relpath(f, self.working_dir),
                            }
                            break

                    # Break out of loop if license is found
                    if license_found:
                        break
            else:
                bad_licenses[pkg_name] = license

        # If we had any bad licenses, we need to abort
        if bad_licenses:
            for k in bad_licenses.keys():
                print("{} had no acceptable licenses: {}".format(
                    k, bad_licenses[k]))
            raise Exception("Bad licenses in vendored packages.")

        # Write license map to file
        if print_map_to_file:
            with open(os.path.join(self.working_dir, print_map_to_file),
                      'w') as lfile:
                json.dump(license_map, lfile, sort_keys=True)

        # Raise missing licenses unless we have a valid reason to ignore them
        raise_missing_license = False
        for name, v in license_map.items():
            if 'license_file' not in v and v.get('license',
                                                 '') != self.APACHE_LICENSE:
                raise_missing_license = True
                print("  {}: Missing license file. Fix or add to ignorelist.".
                      format(name))

        if raise_missing_license:
            raise Exception(
                "Unhandled missing license file. "
                "Make sure all are accounted for before continuing.")

        print("Add the following licenses to the ebuild: \n",
              sorted([x for x in has_license_types]))


def main(args):
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

    # Combine license file and check for any bad licenses
    lm = LicenseManager(current_path, vendor)
    lm.generate_license(args.skip_license_check, args.license_map)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Vendor packages properly')
    parser.add_argument('--skip-license-check',
                        '-s',
                        help='Skip the license check on a specific package',
                        action='append')
    parser.add_argument('--license-map', help='Write license map to this file')
    args = parser.parse_args()

    main(args)
