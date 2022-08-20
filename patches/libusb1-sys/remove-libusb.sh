#!/bin/bash -eux
# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# Remove bundled libusb sources. This breaks the 'vendored' feature of this
# crate, which should not be used in ChromiumOS. We want to link against the
# system copy of libusb.

rm -r libusb
