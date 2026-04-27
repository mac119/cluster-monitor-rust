#!/bin/bash
cargo build --release
mkdir -p bin
cp target/release/monitor-on-vip bin/
cp target/release/monitor-on-proxy bin/
echo "Build complete. Binaries are in bin/"
