#!/bin/bash
set -e

cd "$(dirname "$0")/.."
PROJECT_ROOT=$(pwd)

APP_DIR="/opt/character-sheet"
SERVICE_NAME="character-sheet"

echo "=== Setting up Character Sheet on VPS ==="

# Create app directory
sudo mkdir -p "$APP_DIR/static"
sudo mkdir -p "$APP_DIR/data"

# Stop running service before overwriting binary
if sudo systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
    echo "Stopping running service..."
    sudo systemctl stop "$SERVICE_NAME"
fi

# Copy binary and static files
echo "Copying server binary..."
sudo cp "$PROJECT_ROOT/target/release/server" "$APP_DIR/server"

echo "Copying static files..."
sudo cp -r "$PROJECT_ROOT/server/static/"* "$APP_DIR/static/"

echo "Copying data files..."
BACKUP_DIR="$PROJECT_ROOT/backup/$(date +%Y-%m-%d_%H-%M-%S)"
mkdir -p "$BACKUP_DIR"
sudo cp "$APP_DIR/data/"* "$BACKUP_DIR/"
sudo cp "$PROJECT_ROOT/data/"* "$APP_DIR/data/"

# Set ownership
sudo chown -R www-data:www-data "$APP_DIR"
sudo chmod +x "$APP_DIR/server"

# Install systemd service
echo "Installing systemd service..."
sudo tee "/etc/systemd/system/${SERVICE_NAME}.service" > /dev/null <<EOF
[Unit]
Description=Character Sheet Game Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=${APP_DIR}
ExecStart=${APP_DIR}/server
Environment=RUST_LOG=server=info
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"
sudo systemctl restart "$SERVICE_NAME"

echo "Waiting for server to start..."
sleep 2

if sudo systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "Server is running on port 8080"
else
    echo "Server failed to start. Check logs:"
    echo "  journalctl -u $SERVICE_NAME -n 50"
    exit 1
fi

echo ""
echo "=== Setup complete ==="
echo "App: http://$(hostname -I | awk '{print $1}'):8080"
echo ""
echo "Useful commands:"
echo "  sudo systemctl status $SERVICE_NAME"
echo "  sudo journalctl -u $SERVICE_NAME -f"
echo "  sudo systemctl restart $SERVICE_NAME"
