#!/usr/bin/env bash

set -ex

# Build all Wasm Components.

pushd wasm-components

cargo component build

popd # components

# Build main app.
cargo build
