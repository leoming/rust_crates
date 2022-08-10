#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Copyright 2021 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
""" This script cleans up the vendor directory.
"""
import argparse
import collections
import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import toml

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
    print(f"-- Applying {patch} to {workdir}")
    proc = subprocess.run(["patch", "-p1", "-i", patch], cwd=workdir)
    return proc.returncode == 0


def determine_vendor_crates(vendor_path):
    """Returns a map of {crate_name: [directory]} at the given vendor_path."""
    result = collections.defaultdict(list)
    for crate_name_plus_ver in os.listdir(vendor_path):
      name, _ = crate_name_plus_ver.rsplit('-', 1)
      result[name].append(crate_name_plus_ver)

    for crate_list in result.values():
      crate_list.sort()
    return result


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

    vendor_crate_map = determine_vendor_crates(vendor_path)
    # Look for all patches and apply them
    for d in os.listdir(patches_path):
        dir_path = os.path.join(patches_path, d)

        # We don't process patches in root dir
        if not os.path.isdir(dir_path):
            continue

        for patch in os.listdir(dir_path):
            file_path = os.path.join(dir_path, patch)

            # Skip if not a patch file
            if not os.path.isfile(file_path) or not patch.endswith(".patch"):
                continue

            # We accept one of two forms here:
            # - direct targets (these name # `${crate_name}-${version}`)
            # - simply the crate name (which applies to all versions of the
            #   crate)
            direct_target = os.path.join(vendor_path, d)
            if os.path.isdir(direct_target):
                # If there are any patches, queue checksums for that folder.
                checksums_for[d] = True

                # Apply the patch. Exit from patch loop if patching failed.
                if not apply_single_patch(file_path, direct_target):
                    print("Failed to apply patch: {}".format(patch))
                    break
            elif d in vendor_crate_map:
                for crate in vendor_crate_map[d]:
                  checksums_for[crate] = True
                  target = os.path.join(vendor_path, crate)
                  if not apply_single_patch(file_path, target):
                      print(f'Failed to apply patch {patch} to {target}')
                      break
            else:
                raise RuntimeError(f'Unknown crate in {vendor_path}: {d}')

    # Re-run checksums for all modified packages since we applied patches.
    for key in checksums_for.keys():
        _rerun_checksums(os.path.join(vendor_path, key))


def run_cargo_vendor(working_dir):
    """Runs cargo vendor.

    Args:
        working_dir: Directory to run inside. This should be the directory where
                     Cargo.toml is kept.
    """
    # Cargo will refuse to revendor into versioned directories, which leads to
    # repeated `./vendor.py` invocations trying to apply patches to
    # already-patched sources. Remove the existing vendor directory to avoid
    # this.
    vendor_dir = working_dir / 'vendor'
    if vendor_dir.exists():
      shutil.rmtree(vendor_dir)
    subprocess.check_call(
        ['cargo', 'vendor', '--versioned-dirs', '-v'],
        cwd=working_dir,
    )


def load_metadata(working_dir, filter_platform=DEFAULT_PLATFORM_FILTER):
    """Load metadata for manifest at given directory.

    Args:
        working_dir: Directory to run from.
        filter_platform: Filter packages to ones configured for this platform.
    """
    manifest_path = os.path.join(working_dir, 'Cargo.toml')
    cmd = [
        'cargo', 'metadata', '--format-version', '1', '--manifest-path',
        manifest_path
    ]

    # Conditionally add platform filter
    if filter_platform:
        cmd.append("--filter-platform")
        cmd.append(filter_platform)

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
            pkg_version = package['version']
            license_files = [
                x for x in self._find_license_in_dir(
                    os.path.join(self.vendor_dir, f'{pkg_name}-{pkg_version}'))
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


# TODO(abps) - This needs to be replaced with datalog later. We should compile
#              all crab files into datalog and query it with our requirements
#              instead.
class CrabManager:
    """Manage audit files."""
    def __init__(self, working_dir, crab_dir):
        self.working_dir = working_dir
        self.crab_dir = crab_dir

    def _check_bad_traits(self, crabdata):
        """Checks that a package's crab audit meets our requirements.

        Args:
            crabdata: Dict with crab keys in standard templated format.
        """
        common = crabdata['common']
        # TODO(b/200578411) - Figure out what conditions we should enforce as
        #                     part of the audit.
        conditions = [
            common.get('deny', None),
        ]

        # If any conditions are true, this crate is not acceptable.
        return any(conditions)

    def verify_traits(self):
        """ Verify that all required CRAB traits for this repository are met.
        """
        metadata = load_metadata(self.working_dir)

        failing_crates = {}

        # Verify all packages have a CRAB file associated with it and they meet
        # all our required traits
        for package in metadata['packages']:
            # Skip vendor_libs
            if package['name'] == 'vendor_libs':
                continue

            crabname = "{}-{}".format(package['name'], package['version'])
            filename = os.path.join(self.crab_dir, "{}.toml".format(crabname))

            # If crab file doesn't exist, the crate fails
            if not os.path.isfile(filename):
                failing_crates[crabname] = "No crab file".format(filename)
                continue

            with open(filename, 'r') as f:
                crabdata = toml.loads(f.read())

            # If crab file's crate_name and version keys don't match this
            # package, it also fails. This is just housekeeping...
            if package['name'] != crabdata['crate_name'] or package[
                    'version'] != crabdata['version']:
                failing_crates[crabname] = "Crate name or version don't match"
                continue

            if self._check_bad_traits(crabdata):
                failing_crates[crabname] = "Failed bad traits check"

        # If we had any failing crates, list them now
        if failing_crates:
            print('Failed CRAB audit:')
            for k, v in failing_crates.items():
                print('  {}: {}'.format(k, v))


class CrateDestroyer():
    LIB_RS_BODY = """compile_error!("This crate cannot be built for this configuration.");\n"""

    def __init__(self, working_dir, vendor_dir):
        self.working_dir = working_dir
        self.vendor_dir = vendor_dir

    def _modify_cargo_toml(self, pkg_path):
        with open(os.path.join(pkg_path, "Cargo.toml"), "r") as cargo:
            contents = toml.load(cargo)

        # Change description, license and delete license key
        contents["package"]["description"] = "Empty crate that should not build."
        contents["package"]["license"] = "Apache-2.0"
        if contents["package"].get("license_file"):
            del contents["package"]["license_file"]

        with open(os.path.join(pkg_path, "Cargo.toml"), "w") as cargo:
            toml.dump(contents, cargo)

    def _replace_source_contents(self, package_path):
        # First load the checksum file before starting
        checksum_file = os.path.join(package_path, ".cargo-checksum.json")
        with open(checksum_file, 'r') as csum:
            checksum_contents = json.load(csum)

        # Also load the cargo.toml file which we need to write back
        cargo_file = os.path.join(package_path, "Cargo.toml")
        with open(cargo_file, 'rb') as cfile:
            cargo_contents = cfile.read()

        shutil.rmtree(package_path)

        # Make package and src dirs and replace lib.rs
        os.makedirs(os.path.join(package_path, "src"), exist_ok=True)
        with open(os.path.join(package_path, "src", "lib.rs"), "w") as librs:
            librs.write(self.LIB_RS_BODY)

        # Restore cargo.toml
        with open(cargo_file, 'wb') as cfile:
            cfile.write(cargo_contents)

        # Restore checksum
        with open(checksum_file, 'w') as csum:
            json.dump(checksum_contents, csum)

    def destroy_unused_crates(self):
        all_packages = load_metadata(self.working_dir, filter_platform=None)
        used_packages = set([p["name"] for p in load_metadata(self.working_dir)["packages"]])

        cleaned_packages = []
        for package in all_packages["packages"]:

            # Skip used packages
            if package["name"] in used_packages:
                continue

            # Detect the correct package path to destroy
            pkg_path = os.path.join(self.vendor_dir, "{}-{}".format(package["name"], package["version"]))
            if not os.path.isdir(pkg_path):
                print(f'Crate {package["name"]} not found at {pkg_path}')
                continue

            self._replace_source_contents(pkg_path)
            self._modify_cargo_toml(pkg_path)
            _rerun_checksums(pkg_path)
            cleaned_packages.append(package["name"])

        for pkg in cleaned_packages:
            print("Removed unused crate", pkg)

def main(args):
    current_path = pathlib.Path(__file__).parent.absolute()
    patches = os.path.join(current_path, "patches")
    vendor = os.path.join(current_path, "vendor")
    crab_dir = os.path.join(current_path, "crab", "crates")

    # First, actually run cargo vendor
    run_cargo_vendor(current_path)

    # Order matters here:
    # - Apply patches (also re-calculates checksums)
    # - Cleanup any owners files (otherwise, git check-in or checksums are
    #   unhappy)
    # - Destroy unused crates
    apply_patches(patches, vendor)
    cleanup_owners(vendor)
    destroyer = CrateDestroyer(current_path, vendor)
    destroyer.destroy_unused_crates()

    # Combine license file and check for any bad licenses
    lm = LicenseManager(current_path, vendor)
    lm.generate_license(args.skip_license_check, args.license_map)

    # Run crab audit on all packages
    crab = CrabManager(current_path, crab_dir)
    crab.verify_traits()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Vendor packages properly')
    parser.add_argument('--skip-license-check',
                        '-s',
                        help='Skip the license check on a specific package',
                        action='append')
    parser.add_argument('--license-map', help='Write license map to this file')
    args = parser.parse_args()

    main(args)
