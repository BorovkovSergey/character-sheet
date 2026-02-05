#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building client for production..."
cd client
trunk build --release

echo "Copying to server static directory..."
mkdir -p ../server/static
cp -r dist/* ../server/static/

echo "Building server..."
cd ../server
cargo build --release

echo "Production build complete!"
echo "Run with: ./target/release/server"
