#!/bin/bash
# Issue #8 Deployment Script
# Deploys OpenClaw as systemd service for persistence across SSH disconnects

set -e

echo "=========================================="
echo "🚀 Jelly Persistence Fix - Deployment"
echo "Issue #8: systemd service setup"
echo "=========================================="

SERVICE_FILE="/etc/systemd/system/openclaw.service"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

echo ""
echo "Step 1: Stopping any running OpenClaw processes..."
pkill -f "openclaw gateway" 2>/dev/null || true
sleep 2

echo ""
echo "Step 2: Creating systemd user directory..."
mkdir -p "$SYSTEMD_USER_DIR"

echo ""
echo "Step 3: Installing service file..."
cat > "$SYSTEMD_USER_DIR/openclaw.service" << 'EOF'
[Unit]
Description=OpenClaw AI Agent (Jelly)
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/reggie_marley_gardner/.openclaw/workspace
Environment="HOME=/home/reggie_marley_gardner"
Environment="PATH=/usr/local/bin:/usr/bin:/bin"
ExecStart=/usr/bin/openclaw gateway start --config /home/reggie_marley_gardner/.openclaw/workspace/config.json
ExecStop=/usr/bin/openclaw gateway stop
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
EOF

echo ""
echo "Step 4: Reloading systemd daemon..."
systemctl --user daemon-reload

echo ""
echo "Step 5: Enabling linger for user services..."
sudo loginctl enable-linger reggie_marley_gardner || echo "Note: linger enable may require sudo"

echo ""
echo "Step 6: Enabling OpenClaw service..."
systemctl --user enable openclaw

echo ""
echo "Step 7: Starting OpenClaw service..."
systemctl --user start openclaw

echo ""
echo "Step 8: Verifying service status..."
systemctl --user status openclaw --no-pager || true

echo ""
echo "=========================================="
echo "✅ Deployment Complete!"
echo "=========================================="
echo ""
echo "Check status with:"
echo "  systemctl --user status openclaw"
echo ""
echo "View logs with:"
echo "  journalctl --user -u openclaw -f"
echo ""
echo "Test: Close your laptop, wait 30s, reopen."
echo "Jelly should still be running via systemd."
echo ""
