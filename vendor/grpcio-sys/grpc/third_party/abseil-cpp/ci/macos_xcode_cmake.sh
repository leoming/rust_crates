#!/bin/bash
#
# Copyright 2019 The Abseil Authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    https://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# This script is invoked on Kokoro to test Abseil on macOS.
# It is not hermetic and may break when Kokoro is updated.

set -euox pipefail

if [[ -z ${ABSEIL_ROOT:-} ]]; then
  ABSEIL_ROOT="$(dirname ${0})/.."
fi
ABSEIL_ROOT=$(realpath ${ABSEIL_ROOT})

if [[ -z ${ABSL_CMAKE_BUILD_TYPES:-} ]]; then
  ABSL_CMAKE_BUILD_TYPES="Debug"
fi

if [[ -z ${ABSL_CMAKE_BUILD_SHARED:-} ]]; then
  ABSL_CMAKE_BUILD_SHARED="OFF ON"
fi

for compilation_mode in ${ABSL_CMAKE_BUILD_TYPES}; do
  for build_shared in ${ABSL_CMAKE_BUILD_SHARED}; do
    BUILD_DIR=$(mktemp -d ${compilation_mode}.XXXXXXXX)
    cd ${BUILD_DIR}

    # TODO(absl-team): Enable -Werror once all warnings are fixed.
    time cmake ${ABSEIL_ROOT} \
      -GXcode \
      -DBUILD_SHARED_LIBS=${build_shared} \
      -DCMAKE_BUILD_TYPE=${compilation_mode} \
      -DCMAKE_CXX_STANDARD=11 \
      -DCMAKE_MODULE_LINKER_FLAGS="-Wl,--no-undefined" \
      -DABSL_USE_GOOGLETEST_HEAD=ON \
      -DABSL_RUN_TESTS=ON
    time cmake --build .
    time ctest -C ${compilation_mode} --output-on-failure
  done
done
