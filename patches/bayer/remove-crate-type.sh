#!/bin/bash -eu
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Cargo.toml has:
#   crate-type = ["rlib", "dylib"]
# so that this crate can be used as a normal Rust library or as a DSO from
# other languages. But that confuses cargo when it tries to use this crate
# in an executable, like hps-mon, because it thinks we want to link this
# crate as a dylib instead of an rlib:
#   error: cannot prefer dynamic linking when performing LTO
#   note: only 'staticlib', 'bin', and 'cdylib' outputs are supported with LTO
# Since nothing uses the DSO from this crate, and we aren't packaging it,
# just hack Cargo.toml to make this into a normal Rust library.
sed -i -e '/crate-type = /d' Cargo.toml
