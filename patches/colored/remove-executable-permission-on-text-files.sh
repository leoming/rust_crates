#!/bin/bash -eux
# Copyright 2022 The ChromiumOS Authors.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# These text files should not be executable. Our preupload checks will complain
# if they are.
chmod -x CHANGELOG.md README.md
