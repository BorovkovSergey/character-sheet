#!/bin/bash
# Run the server in development mode
cd "$(dirname "$0")/.."
RUST_LOG=server=debug,tower_http=debug cargo run -p server
