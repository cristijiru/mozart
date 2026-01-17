#!/bin/bash

# Build WASM package for Mozart Core
# Outputs to web/src/wasm/pkg

set -e

echo "Building Mozart Core WASM..."

cd crates/mozart-core

# Build with wasm-pack
wasm-pack build \
    --target web \
    --features wasm \
    --out-dir ../../web/src/wasm/pkg

echo "WASM build complete!"
echo "Output: web/src/wasm/pkg/"
