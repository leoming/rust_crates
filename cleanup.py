#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# Copyright 2021 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
""" This script cleans up the vendor directory.
"""
import json
import os
import pathlib


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


def main():
    current_path = pathlib.Path(__file__).parent.absolute()

    # All cleanups
    cleanup_owners(os.path.join(current_path, 'vendor'))


if __name__ == '__main__':
    main()
