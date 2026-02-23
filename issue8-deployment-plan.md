# Issue #8 Deployment Plan: Jelly Persistence Fix

**Objective:** Make Jelly (OpenClaw) persistent across SSH disconnections

**Target:** Google Cloud VM running OpenClaw

---

## The Problem

```
[Laptop] --SSH--> [Google Cloud VM]
   |                     |
 Sleep/close           SSH drops
   |                     |
   +-----> [Jelly dies] <-----+
```

OpenClaw is currently started manually via SSH. When the SSH session terminates, the process receives SIGHUP and shuts down.

---

## The Solution: Systemd Service

Deploy OpenClaw as a system service that:
- Starts automatically on boot
- Restarts on failure
- Survives SSH disconnections
- Runs under dedicated user for security

---

## Deployment Artifacts

### 1. Systemd Service File (`openclaw.service`)

```systemd
[Unit]
Description=OpenClaw Agent (Jelly)
After=network.target

[Service]
Type=simple
User=openclaw
Group=openclaw
WorkingDirectory=/home/openclaw/openclaw
ExecStart=/usr/bin/openclaw agent start
Restart=always
RestartSec=5
Environment="OPENCLAW_CONFIG=/home/openclaw/.openclaw/config.json"

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/home/openclaw/.openclaw

[Install]
WantedBy=multi-user.target
```

### 2. Deployment Script (`deploy-issue8.sh`)

```bash
#!/bin/bash
set -e

SERVICE_FILE="/etc/systemd/system/openclaw.service"

echo "=== Jelly Persistence Fix Deployment ==="
echo "Step 1: Stopping any running OpenClaw..."
pkill -f "openclaw agent" || true

echo "Step 2: Creating dedicated user (if needed)..."
id -u openclaw &>/dev/null || useradd -r -s /bin/false openclaw

echo "Step 3: Installing systemd service..."
cat > $SERVICE_FILE << 'EOF'
[Unit]
Description=OpenClaw Agent (Jelly)
After=network.target

[Service]
Type=simple
User=openclaw
Group=openclaw
WorkingDirectory=/home/openclaw/openclaw
ExecStart=/usr/bin/openclaw agent start
Restart=always
RestartSec=5
Environment="OPENCLAW_CONFIG=/home/openclaw/.openclaw/config.json"
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/home/openclaw/.openclaw

[Install]
WantedBy=multi-user.target
EOF

echo "Step 4: Setting correct ownership..."
chown -R openclaw:openclaw /home/openclaw/.openclaw || true

echo "Step 5: Reloading systemd..."
systemctl daemon-reload

echo "Step 6: Enabling and starting service..."
systemctl enable openclaw
systemctl start openclaw

echo "=== Deployment Complete ==="
echo "Status check:"
systemctl status openclaw --no-pager
```

### 3. Rollback Script (`rollback-issue8.sh`)

```bash
#!/bin/bash
set -e

echo "=== Rolling back Issue #8 ==="
systemctl stop openclaw || true
systemctl disable openclaw || true
rm -f /etc/systemd/system/openclaw.service
systemctl daemon-reload

echo "Rollback complete. You can now start OpenClaw manually via SSH."
```

---

## Security Considerations

- **Dedicated user:** `openclaw` user with minimal privileges
- **No shell access:** `/bin/false` as shell
- **Filesystem restrictions:** `ProtectSystem=strict` limits write access
- **No new privileges:** `NoNewPrivileges=true` prevents privilege escalation
- **Network-only exposure:** Only required ports exposed

---

## Verification Steps

After deployment:
1. ✅ `systemctl status openclaw` shows "active (running)"
2. ✅ Close SSH connection: `exit`
3. ✅ Reconnect and verify: `systemctl status openclaw` still running
4. ✅ Verify AI team cron jobs still active: `systemctl list-timers`

---

## Deployment Checklist

- [ ] User approval received: "I approve this deployment"
- [ ] Backup current OpenClaw config
- [ ] Execute deploy-issue8.sh
- [ ] Verify service is running
- [ ] Test SSH disconnect/reconnect
- [ ] Monitor for 24 hours

---

**Prepared by:** Deployment Team 🚀  
**Awaiting:** Human approval to proceed
