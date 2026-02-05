#!/bin/bash
# Run the client in development mode with trunk
cd "$(dirname "$0")/../client"
trunk serve --port 8081
