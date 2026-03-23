# Project AETHER — Validator Onboarding Guide
## Phase 6 Deliverable 3/4

**Status:** READY FOR VALIDATORS  
**Author:** 🚀 Launch-Pad (Phase 6 Mainnet Specialist)  
**Date:** 2026-03-22  
**Issue:** [#108](https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/108)

---

## 1. Welcome, Founding Validator

Thank you for joining Project AETHER at genesis. As a Founding Validator, you're running consensus infrastructure from day one — you have elevated responsibilities and rewards. This guide covers everything you need to go from zero to producing blocks.

**Your Advantages:**
- 2.0x reward multiplier during the 12-month bootstrap period
- Founding Validator status (permanent on-chain credential)
- First say in governance proposals
- Early access to protocol upgrades

**Your Responsibilities:**
- 95%+ uptime during bootstrap period (required for multiplier)
- Running secure, hardened node infrastructure
- Participating in emergency governance votes if needed

---

## 2. Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 16 cores | 32+ cores |
| RAM | 64 GB | 128 GB |
| Storage | 2 TB NVMe SSD | 4 TB+ NVMe SSD |
| Network | 1 Gbps | 10 Gbps |
| Redundancy | Single node | Dual-node with automatic failover |

**Important:** Storage will grow indefinitely. Budget for at least 6TB initially with expansion plans.

---

## 3. Prerequisites

Before starting, ensure you have:

### 3.1 Required Accounts

- [ ] AETH wallet with **10,000 AETH** for staking (Founding Validator minimum)
- [ ] GitHub account (for accessing `aether-validator` repo)
- [ ] Server with SSH access (root or sudo)
- [ ] Domain name for your RPC endpoint (optional but recommended)

### 3.2 Required Software

```bash
# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable

# Install Solana CLI dependencies (Solana-compatible toolchain)
git clone https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop.git
cd Jelly-legs-unsteady-workshop
cargo build --release 2>&1 | tail -20
```

---

## 4. Node Setup

### 4.1 Build the Validator Binary

```bash
cd ~/aether
cargo build --release --bin aether-validator
# Binary will be at: target/release/aether-validator
```

### 4.2 Create Validator Identity

```bash
./target/release/aether-validator create-validator-identity \
    --out ~/validator-keypair.json

# BACKUP THIS FILE — it controls your validator identity
# If lost, you lose your validator status
cp ~/validator-keypair.json ~/validator-keypair.json.backup
```

### 4.3 Create Vote Account

```bash
./target/release/aether-validator create-vote-account \
    --validator-keypair ~/validator-keypair.json \
    --out ~/vote-keypair.json
```

### 4.4 Initialize Stake Account

```bash
# Stake 10,000 AETH minimum for founding validators
./target/release/aether-validator create-stake-account \
    --vote-account ~/vote-keypair.json \
    --amount 10000 \
    --keypair ~/validator-keypair.json \
    --out ~/stake-keypair.json
```

### 4.5 Generate Genesis Configuration

```bash
# Genesis config will be provided by the core team
# This is an example — replace with actual values at genesis
./target/release/aether-validator create-genesis \
    --timestamp 1747800000 \
    --chain-id aether-mainnet-1 \
    --validators ./founding_validators.json \
    --out ./genesis.json
```

---

## 5. Configuration

### 5.1 Create `config.yaml`

```yaml
# ~/aether-validator.yaml
validator:
  identity: /home/ubuntu/validator-keypair.json
  vote_account: /home/ubuntu/vote-keypair.json
  stake_account: /home/ubuntu/stake-keypair.json
  
  # Network
  rpc:
    bind: 0.0.0.0:8899
    port: 8899
  p2p:
    bind: 0.0.0.0:8001
    port: 8001
    peers: 10
    max_peers: 100

  # Consensus
  consensus:
    mode: aetherflow
    slot_time_ms: 400
    tower_finality: 12
    min_stake: 100

  # AI Priority Lanes
  priority_lanes:
    lane_0_weight: 10
    lane_1_weight: 1
    lane_2_weight: 0.1

  # Performance
  tpu:
    port: 8003
    connections: 50
  poh:
    hash_per_sec: 100

  # Monitoring
  metrics:
    enabled: true
    port: 9320
  health:
    enabled: true
    port: 9321
```

### 5.2 Systemd Service

```ini
# /etc/systemd/system/aether-validator.service
[Unit]
Description=Aether Validator
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/aether
ExecStart=/home/ubuntu/aether/target/release/aether-validator \
    --config /home/ubuntu/aether-validator.yaml
Restart=always
RestartSec=10
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable aether-validator
sudo systemctl start aether-validator
sudo systemctl status aether-validator
```

---

## 6. Network & Security

### 6.1 Firewall Rules

```bash
# SSH (you)
sudo ufw allow 22/tcp

# RPC API (public read)
sudo ufw allow 8899/tcp

# P2P gossip (public)
sudo ufw allow 8001/udp
sudo ufw allow 8001/tcp

# TVU (Solana-style block distribution)
sudo ufw allow 8002/udp
sudo ufw allow 8002/tcp

# Metrics (restrict to internal)
sudo ufw allow from 10.0.0.0/8 to any port 9320
sudo ufw allow from 172.16.0.0/12 to any port 9320

# Enable firewall
sudo ufw enable
```

### 6.2 DDoS Protection

- Use Cloudflare or similar for RPC endpoints
- Rate limit RPC: max 100 req/sec per IP
- Enable TCP multipath for P2P
- Monitor for anomalous gossip traffic

### 6.3 Key Security

```
🚨 NEVER expose validator-keypair.json to the internet
🚨 NEVER run the validator as root
🚨 ALWAYS keep encrypted backups of keypairs
🚨 Enable U2F/FIDO2 on any machine accessing validator keys
```

---

## 7. Monitoring & Alerting

### 7.1 Check Validator Status

```bash
# Check if producing blocks
./target/release/aether-validator show-validators

# Check your stake and status
./target/release/aether-validator stake-info --vote-account <YOUR_VOTE_ADDR>

# Check RPC health
curl -X POST http://localhost:8899/health -d '{"jsonrpc":"2.0","id":1}'
```

### 7.2 Key Metrics to Watch

| Metric | Healthy | Alert |
|--------|---------|-------|
| Uptime | >95% | <90% for 1 hour |
| Block production | >98% | <95% for 15 min |
| Peer count | 5–50 | <3 for 5 min |
| RPC latency | <100ms | >500ms |
| Slot height | Increasing | Stuck for 5 min |

### 7.3 Alerting Setup

```bash
# Install Prometheus node exporter (example)
sudo apt install prometheus-node-exporter

# Configure Prometheus to scrape validator metrics
# prometheus.yml:
#   scrape_configs:
#     - job_name: 'aether-validator'
#       static_configs:
#         - targets: ['localhost:9320']
```

### 7.4 Log Management

```bash
# Rotate logs daily
sudo journalctl -u aether-validator -n 1000 --no-pager > /var/log/aether-validator.log
sudo logrotate -f /etc/logrotate.d/aether-validator
```

---

## 8. Recovery Procedures

### 8.1 Validator Goes Offline

1. Diagnose: check `journalctl -u aether-validator -n 50`
2. If hardware issue: restore from backup, restart service
3. If binary crash: update to latest stable binary
4. Restart: `sudo systemctl restart aether-validator`
5. Verify: check slot height is increasing

### 8.2 Keypair Compromised

```bash
# IMMEDIATELY deactivate the compromised validator
./target/release/aether-validator deactivate-validator \
    --identity compromised-keypair.json \
    --new-identity new-keypair.json

# Contact the core team via secured channel
# Your stake will beSlashProtected during investigation
```

### 8.3 Network Partition

- Validators will automatically reconnect when network heals
- Tower BFT will resume voting once connected
- No manual intervention needed unless >30 minutes

---

## 9. Expected Reward Calculation

**Founding Validator — 2.0x Multiplier Example:**

```
Base APY for validators: ~6%
Your stake: 10,000 AETH
Your effective stake: 20,000 AETH (2.0x multiplier)

Daily rewards (simplified):
  20,000 AETH × (6% / 365) = 3.29 AETH/day

Bootstrap period: 12 months
Expected rewards during bootstrap: ~1,200 AETH
After bootstrap: ~600 AETH/year at base rate
```

**Note:** Rewards are subject to tokenomics votes. This is an estimate only.

---

## 10. Support & Contacts

| Channel | Use For | Response Time |
|---------|---------|--------------|
| Discord #validators | General questions | < 24h |
| Signal (DM core team) | Security incidents | < 1h |
| GitHub Issues | Bug reports, feature requests | < 48h |
| Emergency: email | Critical failures only | < 4h |

---

## 11. Checklist

Before genesis, confirm all of the following:

- [ ] 10,000 AETH staked in vote account
- [ ] Validator binary builds and runs
- [ ] `config.yaml` correctly configured
- [ ] Systemd service running and enabled
- [ ] Firewall configured
- [ ] Metrics endpoint accessible
- [ ] Alerting set up (email/Slack/PagerDuty)
- [ ] Keypairs backed up (encrypted, multiple locations)
- [ ] You've read the AetherFlow whitepaper
- [ ] You've joined Discord #validators channel
- [ ] Contact info submitted to core team

---

*Document version: 1.0 — 🚀 Launch-Pad | Phase 6*
