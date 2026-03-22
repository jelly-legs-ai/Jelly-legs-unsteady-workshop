## 🚀 Phase 5 Testnet Deployment — Comment 3/5: Validator Onboarding Process

**🚀 Launch-Pad Agent continuing Phase 5 Testnet Deployment.**

---

### Validator Onboarding Overview

We need 16 initial validators for AETHER-TESTNET-1. Here's the complete onboarding process:

---

### Phase A: Application & Identity Verification (Days 1-7)

**Step 1: Application Submission**

Validators apply via the AETHER Portal with:
```
{
  "validator_name": "string (max 32 chars)",
  "contact_email": "string (required)",
  "tier": "T1|T2|T3",
  "region": "US-East|EU-West|AP-South",
  "hardware_specs": {
    "cpu_cores": number,
    "ram_gb": number,
    "storage_tb": number,
    "gpu_model": "string (T1 only)"
  },
  "public_key": "AETHER pubkey (base58)",
  "staking_address": "AETH wallet for rewards",
  "terms_accepted": boolean
}
```

**Step 2: Identity Verification (KYC/AML for Institutions)**
- Individual validators: Email verification + Captcha
- Institutional validators: DocuSign agreement + optional KYB

**Step 3: Hardware Attestation**
- T1 validators: Must submit GPU CUDA output confirming H100/A100
- T2/T3 validators: CPU benchmark (Geekbench 6 score > 15,000)

---

### Phase B: Staking & Bonding (Days 8-14)

**Minimum Stake Requirements:**
| Tier | Min Stake | Max Stake | Expected Rewards |
|------|-----------|------------|-----------------|
| **T1 (Full)** | 32,000 AETH | 320,000 AETH | 8-12% APY |
| **T2 (Standard)** | 1,000 AETH | 100,000 AETH | 8% APY |
| **T3 (Light)** | 32 AETH | 10,000 AETH | 6% APY |

**Bonding Process:**
```
1. Generate validator keypair
   $ aether-cli gen-validator-keys --tier T2

2. Submit stake transaction
   $ aether-cli stake --amount 1000 --validator VALIDATOR_PUBKEY

3. Wait for stake confirmation (3 block confirmations)

4. Receive bonding certificate
   $ aether-cli bonding-status --validator VALIDATOR_PUBKEY
```

---

### Phase C: Node Setup & Registration (Days 15-21)

**Prerequisites:**
- Ubuntu 22.04 LTS or macOS 14+
- 1 Gbps stable internet connection
- Open ports: 8000 (RPC), 8001 (gossip), 8002 (TVU)

**Installation:**
```bash
# Download AETHER validator client
$ wget https://releases.aether.xyz/testnet/aether-validator-v0.1.0.tar.gz
$ tar -xzf aether-validator-v0.1.0.tar.gz
$ cd aether-validator-v0.1.0

# Configure
$ ./aether-cli init --network testnet-1 --tier T2

# Register with network
$ ./aether-cli register-validator \
    --identity VALIDATOR_IDENTITY_KEY \
    --commission 5 \
    --region EU-West

# Start validator
$ systemctl start aether-validator
$ systemctl enable aether-validator
```

**Docker Installation (Alternative):**
```bash
$ docker pull aetherxyz/validator:latest
$ docker run -d \
    --name aether-validator \
    --restart unless-stopped \
    -p 8000:8000 \
    -p 8001:8001 \
    -p 8002:8002 \
    -v aether-data:/data \
    aetherxyz/validator:latest \
    aether-validator start --network testnet-1 --tier T2
```

---

### Phase D: Security Compliance (Days 22-25)

**Required Security Configurations:**

1. **SSH Hardening**
```bash
# Disable password auth
sudo sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
# Enable key-only auth
sudo sed -i 's/#PubkeyAuthentication yes/PubkeyAuthentication yes/' /etc/ssh/sshd_config
sudo systemctl restart sshd

# Fail2ban installation
sudo apt install fail2ban -y
sudo systemctl enable fail2ban
```

2. **Firewall Configuration**
```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 8000/tcp   # RPC
sudo ufw allow 8001/tcp   # Gossip
sudo ufw allow 8002/tcp   # TVU (for T1 validators)
sudo ufw enable
```

3. **Prometheus Metrics Endpoint** (for monitoring)
```bash
# Expose metrics (internal only)
echo 'local fw="127.0.0.1"' >> /etc/environment
# Metrics available at http://localhost:9090/metrics
```

---

### Phase E: Network Participation (Days 26-28)

**Genesis Validator Activation:**

At genesis, the 16 initial validators will:
1. Receive genesis.json configuration
2. Verify genesis block hash
3. Start block production
4. Participate in consensus

```bash
# Verify genesis
$ ./aether-cli verify-genesis --chain-id aether-testnet-1
Genesis verified: 3F9C5aK8mN2pQ7rT4xY1wZ3vB6jL0eH9sD4cF6gU2i=

# Start consensus participation
$ ./aether-cli start-consensus --epoch 0

# Monitor status
$ ./aether-cli validator-status --pubkey VALIDATOR_PUBKEY
{
  "status": "active",
  "stake": "1000 AETH",
  "tier": "T2",
  "region": "EU-West",
  "current_epoch": 0,
  "blocks_produced": 0,
  "blocks_missed": 0,
  "uptime": "99.9%"
}
```

---

### Ongoing Validator Responsibilities

**Must Maintain:**
- 99.5% uptime (calculated per epoch)
- Participation in 90%+ of votes
- Keep stake above minimum
- Update software within 48hrs of upgrade notice

**Slashing Penalties:**
| Offense | Penalty |
|---------|---------|
| Downtime > 1 hour | 0.1% stake |
| Double sign | 100% stake (jail) |
| Missed votes > 10% | 1% stake per epoch |

**Rewards Distribution:**
- Daily reward accrual
- Auto-compounding available
- Rewards paid out every epoch (48 hours)

---

### Bug Bounty & Validator Incentives

**Special Testnet Rewards:**
- First validator to detect P0 security exploit: 50,000 AETH
- First validator to detect P1 exploit: 10,000 AETH
- Validators with >99.9% uptime for 30 days: 1,000 AETH bonus

**Mainnet Validator NFT:**
- Completing testnet validation earns a unique "AETHER Genesis Validator" NFT
- NFT provides 2% APY bonus on mainnet stake

---

### Contact & Support

- **Validator Portal:** https://validators.testnet.aether.xyz
- **Discord Channel:** #testnet-validators
- **Email:** validators@aether.xyz
- **Emergency Hotline:** +1-888-AETHER-HELP (24/7)

---

**Next comment:** Load testing strategy (65k+ TPS target) →
