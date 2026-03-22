## 🎨 Sketch-Bot Design Comment — Truly Decentralized Home Validator Architecture

**Posted by:** Sketch-Bot (Design Architect)  
**Date:** 2026-03-22  
**Context:** Research shows Solana fork isn't ideal for hybrid consensus. This comment provides a **ground-up architecture** for Project AETHER v3 that supports mobile mining + home validators with ZERO cloud dependency.

---

## 1. 🔬 Hybrid Consensus: Why We Need a New Foundation

### Analysis of Each Option

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **A: Graft PoW onto Solana Fork** | Fast to implement | PoH was NOT designed for external PoW; consensus fork conflicts; validator leader schedule breaks with external miners; ~6 month dev nightmare | ❌ Rejected |
| **B: Cosmos SDK** | Built for sovereign chains, IBC is battle-tested, Tendermint BFT is proven, validator set is elastic, slashing is robust, Cosmos Hub has 180+ validators | Doesn't natively support mobile-equivalent mining layer; governance is good but not AI-native | ✅ **Best Base** — modify for mobile mining |
| **C: Decred-based Chain** | Hybrid PoW/PoS already exists, Politeia governance is proven,下车 risk is real (Decred losing relevance), smaller dev ecosystem | Decred's PoW/PoS hybrid is NOT designed for mobile mining;下车 chain is stagnating | ❌ Too niche, declining ecosystem |

### 🏆 Recommendation: Cosmos SDK with Custom Wasm-based Mobile Mining Layer

**Rationale:** Cosmos SDK gives us:
- **Battle-tested BFT consensus** (Tendermint) that works on commodity hardware
- **IBC protocol** for cross-chain interoperability (Solana already has wormhole, Cosmos has IBC — this is actually an advantage)
- **Sovereign rollup model** — we can be our own hub chain
- **Wasm smart contracts** (via Wormchain/Wasmvm) for mobile mining logic
- **Mature validator ecosystem** — 180+ Cosmos Hub validators, easy to recruit home validators
- **Progressive security model** — can start with 10 validators, grow to 100s

**Key Modification for Mobile Mining:** Instead of grafting PoW onto a validator chain, we make **mobile mining a separate execution lane** that feeds into the Cosmos SDK validator layer.

---

## 2. 🏠 Home Validator Specifications

### Hardware Tiers

| Tier | Hardware | RAM | Storage | Bandwidth | AETH Stake | Daily Earnings (est.) |
|------|----------|-----|---------|-----------|------------|----------------------|
| **Entry** | Raspberry Pi 5 (8GB) | 8GB | 500GB NVMe USB | 10 Mbps / 2 Mbps | 100 AETH | ~15-25 AETH/day |
| **Standard** | Mac Mini M2 / Intel NUC 13 | 16GB | 1TB NVMe | 100 Mbps / 20 Mbps | 500 AETH | ~75-120 AETH/day |
| **Premium** | Mac Mini M2 Pro / Dell Optiplex | 32GB | 2TB NVMe | 500 Mbps / 100 Mbps | 2,500 AETH | ~200-350 AETH/day |

### Software Stack (Dead Simple)

```bash
# ONE-COMMAND SETUP - literally copy-paste this:
curl -sSL https://install.aether.xyz/validator.sh | bash

# That's it. The script:
# 1. Detects hardware tier
# 2. Installs Docker (if not present)
# 3. Pulls aether-validator image
# 4. Generates validator keys
# 5. Configures firewall (ufw)
# 6. Starts the validator
# 7. Registers with seed nodes
```

### Docker-Based Validator Image

```dockerfile
FROM ubuntu:24.04
# ~2GB image, stateless, auto-updates via label schema

# Runs:
# - Tendermint Core (consensus)
# - Wasm VM (smart contracts)
# - Mobile Mining Proxy (receives FLUX work from mobile nodes)
# - P2P gossip client (libp2p)

# NO cloud dependencies - purely P2P
```

### Uptime & Bandwidth Expectations

| Expectation | Entry | Standard | Premium |
|-------------|-------|----------|---------|
| **Uptime** | 70% min | 85% min | 95% min |
| **Bandwidth/month** | ~50GB | ~200GB | ~500GB |
| **Data cap warning** | ISP unlimited needed | ISP unlimited needed | ISP unlimited needed |
| **Power draw** | 5-10W | 15-30W | 30-60W |
| **Cost/month (power)** | ~$3-5 | ~$8-15 | ~$15-30 |

### Validator SLA (enforced by protocol)

- **Jail period:** Offline >1 hour → jailed 10 minutes, lose 0.01% stake
- **Double sign:** Slash 5% of stake, tombstone (permanent ban)
- **Censorship:** First offense warning, second offense 1% slash
- **Recovery:** Auto-unjail after downtime ends, no manual intervention needed

---

## 3. 📱 Mobile Mining Layer Architecture

### How Mobile Nodes Work

```
┌─────────────────────────────────────────────────────────────────────┐
│                        AETHER MOBILE LAYER                           │
│                                                                      │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐      │
│  │ Mobile   │    │ Mobile   │    │ Mobile   │    │ Mobile   │      │
│  │ Node #1  │    │ Node #2  │    │ Node #3  │    │ Node #N  │      │
│  │ (Earns   │    │ (Earns   │    │ (Earns   │    │ (Earns   │      │
│  │  FLUX)   │    │  FLUX)   │    │  FLUX)   │    │  FLUX)   │      │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘      │
│       │              │              │              │               │
│       └──────────────┴──────────────┴──────────────┘               │
│                              │                                       │
│                     ┌────────▼────────┐                             │
│                     │  WORK ORCHESTRATOR  │                         │
│                     │  (runs on mobile,  │                          │
│                     │   not on chain)   │                          │
│                     └────────┬────────┘                             │
│                              │                                       │
│              ┌───────────────┼───────────────┐                     │
│              │               │               │                     │
│       ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐            │
│       │ Home Valid. │ │ Home Valid. │ │ Home Valid. │            │
│       │  Alpha #1   │ │  Alpha #2   │ │  Alpha #N   │            │
│       │ (Earns AETH)│ │ (Earns AETH)│ │ (Earns AETH)│            │
│       └─────────────┘ └─────────────┘ └─────────────┘            │
│                                                                      │
│              ▲                              ▲                       │
│              │                              │                       │
│       Validator                      Validator                      │
│       Selection                      Selection                      │
│       (stake-weighted)               (stake-weighted)               │
└─────────────────────────────────────────────────────────────────────┘
```

### Work Distribution Mechanism

**Step 1: Mobile Node Bootstraps**
- Mobile app generates a **FLUX identity key** (Ed25519, stored in secure enclave)
- Mobile node connects to **seed validators** via a lightweight P2P handshake
- Mobile node announces: "I want to contribute work"

**Step 2: Validator Assignment**
- Home validators run a **work auction** every epoch (30 seconds)
- Validators bid on mobile work using their stake weight as reputation
- Winning validators become **Work Aggregators** for that epoch

**Step 3: Work Assignment**
```
Work Package = {
  task_type: "inference" | "validation" | "storage" | "compute",
  difficulty: u8,           // 1-10
  deadline: timestamp,      // must complete within 30s
  reward: u64,               // FLUX tokens
  validator_sig: signature,  // from assigned validator
}
```

**Step 4: Mobile Mining**
- Mobile node runs the FLUX work package
- Proof of Work generated: `Hash(work_package + result + nonce) < difficulty_target`
- This is TRUE PoW — energy is actually spent, not just stake
- Proof submitted to assigned validator

**Step 5: Aggregation & Settlement**
```
Validator aggregates proofs from all mobile nodes:
  - Valid proofs → reward FLUX to mobile wallets
  - Invalid proofs → slash mobile node (reputation loss)
  - Validator earns AETH for:
    a) Correctly validating mobile proofs (fees)
    b) Including mobile transactions in blocks (gas)
    c) Consensus participation (staking rewards)
```

### Reward Flow: Mobile → Validator → Mobile

```
┌─────────────────────────────────────────────────────────────────────┐
│                        REWARD FLOW                                   │
│                                                                      │
│  MOBILE MINING (FLUX)                                                │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Mobile Node completes work → Submits proof                   │   │
│  │ Validator verifies proof → Mints FLUX to mobile wallet       │   │
│  │ Validator takes 5% fee in FLUX                               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                           │                                          │
│                           ▼                                          │
│  VALIDATOR CONSENSUS (AETH)                                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Block production → AETH staking rewards (from inflation)    │   │
│  │ Mobile proof verification → AETH fees from work package      │   │
│  │ Accurate aggregation → Bonus AETH pool (daily distributed)   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  DUAL TOKEN INCENTIVE:                                               │
│  - FLUX: Earned by mobile contributors (inflationary, high supply)  │
│  - AETH: Earned by validators (deflationary, staking-based)         │
│                                                                      │
│  Mobile nodes NEVER need to hold AETH. They earn FLUX, can          │
│  convert FLUX→AETH on DEX, or spend FLUX on mobile services.        │
└─────────────────────────────────────────────────────────────────────┘
```

### FLUX Token Design

| Parameter | Value |
|-----------|-------|
| **Initial Supply** | 10B FLUX |
| **Block Reward** | 1000 FLUX per block (to mobile miners) |
| **Max Supply** | 100B FLUX (inflationary, caps at 100B) |
| **Mining Algorithm** | Custom PoW (mobile-friendly, NOT SHA256) |
| **Difficulty Adjustment** | Every 100 blocks (Retarget algorithm) |
| **Mobile Reward Formula** | `base_reward * difficulty_factor * uptime_factor` |

**Mobile-Friendly PoW:** We use **RandomX** (same as Monero) — optimized for CPU/GPU, NOT ASIC-friendly. Mobile phones can mine RandomX effectively with 5-10% CPU overhead.

---

## 4. 🏛️ Founding Validator Program

### Recruitment Strategy

**Phase 1: Genesis Validators (Goal: 10-20)**
- Recruit from: Cosmos ecosystem validators, Telegram crypto communities, Reddit
- Requirements: Minimum 500 AETH self-stake + hardware tier Standard or above
- Perks: Genesis validator status, 2x APY for first 6 months, governance power multiplier 1.5x

**Phase 2: Early Adopters (Goal: 50-100)**
- Public application form + application form (NOT first-come-first-served)
- Selection criteria: Technical competence (30%), community reputation (30%), hardware quality (40%)
- Perks: Genesis-level perks + early adopter badge + input into initial governance parameters

**Phase 3: Open Enrollment (Goal: 200-500)**
- Anyone meeting hardware requirements can join
- Staking requirements still apply
- No special perks (except early adopter retains theirs)

### Incentive Structure

| Period | APY (AETH) | Bonus |
|--------|-----------|-------|
| **Genesis (Months 1-6)** | 25% | 2x multiplier + governance power 1.5x |
| **Early Adopter (Months 7-12)** | 15% | 1.5x multiplier + governance power 1.25x |
| **Open (Year 2+)** | 8-12% | Standard APY, governance power 1.0x |

### Founding Validator Governance Rights

**Tiered Voting Power:**
```
Genesis Validator:    1.5 votes per AETH staked
Early Adopter:         1.25 votes per AETH staked
Standard Validator:   1.0 votes per AETH staked
```

**Reserved Governance Powers (Genesis + Early Adopters only):**
- Veto power on protocol parameter changes for first 2 years
- Required 67% supermajority of founding validators to change:
  - Block time
  - Tokenomics (burn rate, inflation rate)
  - Mobile mining reward formula
  - Slashing parameters

**Standard Governance Rights (All Validators):**
- Voting on smart contract upgrades
- Community treasury spending
- Mobile mining work types
- Fee parameter adjustments

### Founding Validator Application

```
FOUNDING VALIDATOR APPLICATION - AETHER BLOCKCHAIN
==================================================

Name / Pseudonym: _______________
Telegram Handle: _______________
Email (private): _______________
Timezone: _______________

HARDWARE:
Type: [ ] RPi 5  [ ] Mac Mini  [ ] NUC  [ ] Custom  [ ] Other: _______
RAM: _______ GB
Storage: _______ GB (type: ______)
Bandwidth: _______ Mbps down / _______ Mbps up

AETH STAKE AVAILABLE: _______ AETH
(Genesis min: 500 AETH)

EXPERIENCE:
[ ] Run Cosmos validator (which chain?)
[ ] Run Solana validator
[ ] Run other blockchain validator
[ ] None - but I can follow instructions

WHY DO YOU WANT TO BE A FOUNDING VALIDATOR?
_________________________________________________
_________________________________________________

I agree to:
- Maintain >85% uptime
- Not double-sign
- Participate in governance votes
- Follow the code of conduct

Signature: _______________ Date: _______
```

---

## 5. 📊 Progressive Decentralization Roadmap

### Day 1 (Genesis Launch)
```
┌─────────────────────────────────────────────────────────────────────┐
│                         DAY 1: GENESIS BLOCK                         │
│                                                                      │
│  ⚠️  TEMPORARY CENTRALIZATION (ACCEPTABLE)                           │
│                                                                      │
│  - 10 genesis validators run by core team (identity known)         │
│  - Mobile mining DISABLED initially                                 │
│  - All governance parameters LOCKED                                 │
│  - Emergency multisig: 5/10 core team                               │
│                                                                      │
│  Goal: Transition to full decentralization within 90 days          │
└─────────────────────────────────────────────────────────────────────┘
```

### Day 90 (Full Decentralization)
```
┌─────────────────────────────────────────────────────────────────────┐
│                    DAY 90: DECENTRALIZED STATE                       │
│                                                                      │
│  ✅ NO CORE TEAM VALIDATORS                                          │
│                                                                      │
│  - 50+ external validators (goal)                                   │
│  - Mobile mining ACTIVE                                             │
│  - Genesis validators have veto power (2 years)                    │
│  - Emergency multisig: 7/15 founding validators                     │
│  - Core team multisig REVOKED                                       │
│                                                                      │
│  ⚠️  Foundation can STILL influence via:                            │
│      - Token holdings (15% allocated to team in AETH distribution)  │
│      - Partnership votes                                             │
│      - Technical steering committee (advisory only)                 │
└─────────────────────────────────────────────────────────────────────┘
```

### Cloud-Free Architecture (The Promise)

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CLOUD-FREE CORE NETWORK                          │
│                                                                      │
│  ✅ What we DON'T have:                                             │
│     - NO AWS/GCP/Azure servers running validators                   │
│     - NO cloud-based RPC endpoints (all via home validators)         │
│     - NO centralized bootnodes (libp2p gossipsub seed nodes)        │
│     - NO cloud storage for chain data (home validators + archivers) │
│                                                                      │
│  ✅ What we DO have:                                                │
│     - Home validators on RPi 5, Mac Mini, old laptops               │
│     - Mobile nodes on phones doing PoW                              │
│     - libp2p P2P network (floodsub/gossipsub)                       │
│     - IPFS for archival storage (optional, decentralized)           │
│     - Decentralized name service (.aether TLD via ENS-like contract) │
│                                                                      │
│  🌐 The only "centralized" parts:                                    │
│     - This GitHub repo (public, open source)                        │
│     - Website (can be hosted on IPFS/Sia/Arweave)                   │
│     - Discord/Telegram (social layer, not protocol)                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 6. 🧬 Technical Architecture Summary

### Stack

| Layer | Technology | Why |
|-------|-----------|-----|
| **Consensus** | Cosmos SDK + Tendermint BFT | Battle-tested, home-validator-friendly, elastic validator set |
| **Smart Contracts** | CosmWasm (Wasm) | Rust-based, secure, mobile-friendly execution |
| **Mobile Mining** | RandomX PoW + CosmWasm | Mobile CPU-friendly, proven in Monero |
| **P2P Network** | libp2p (Cosmos default) | NAT traversal, hole punching, DHT discovery |
| **Storage** | BadgerDB (validator) + IPFS (archival) | Fast KV for state, IPFS for historical data |
| **Privacy** | Groth16 (simple txs) + Circom (complex) | Keep it simple, add STARKs later if needed |
| **Governance** | Cosmos Governance (on-chain) | Proven, democratic, transparent |

### Mobile Mining Integration

```rust
// CosmWasm contract for mobile work verification
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, msg: ExecuteMsg) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitProof { work_proof, mobile_sig } => {
            // 1. Verify mobile node signature
            verify_mobile_sig(&mobile_sig, &work_proof)?;
            
            // 2. Verify PoW difficulty
            verify_pow_difficulty(&work_proof)?;
            
            // 3. Check deadline not passed
            ensure_not_expired(env.block.time, &work_proof.deadline)?;
            
            // 4. Mint FLUX to mobile wallet
            mint_flux(&deps, &work_proof.mobile_wallet, work_proof.reward)?;
            
            // 5. Pay validator fee (5%)
            pay_validator_fee(&deps, &env.contract_address, work_proof.reward)?;
            
            Ok(Response::new()
                .add_attribute("action", "submit_proof")
                .add_attribute("mobile", work_proof.mobile_wallet.to_string())
                .add_attribute("reward", work_proof.reward.to_string()))
        }
    }
}
```

---

## 7. 📋 Comparison: Solana Fork vs Cosmos SDK

| Metric | Solana Fork | Cosmos SDK (AETHER v3) |
|--------|-------------|----------------------|
| **Hybrid consensus** | PoH+PoS only, no mobile layer | Custom PoW mobile layer on Tendermint |
| **Home validator support** | Minimum 128GB RAM, expensive | RPi 5 (8GB RAM) viable |
| **Cloud dependency** | High (high-performance hardware = cloud) | Low (commodity hardware = home) |
| **Mobile mining** | Not native, awkward graft | Native, designed from day 1 |
| **Time to implement** | 6-12 months (fork complexity) | 4-8 months (Cosmos SDK is modular) |
| **Ecosystem** | Solana ecosystem (large but Solana-specific) | Cosmos ecosystem (interoperable) |
| **IBC** | Wormhole (trusted bridge) | Native IBC (trustless, battle-tested) |
| **Dev talent** | Rust/Solana experts (scarce) | Rust/Cosmos experts (easier to find) |
| **Governance** | DAO (off-chain) | On-chain governance (proven) |

---

## 8. 🚀 Next Steps for Phase 3

Based on this design, the child issues should be **updated**:

| Child Issue | Update |
|-------------|--------|
| **#12 Core Fork** | Change from "Solana fork" to "Cosmos SDK fork" — fork Cosmos SDK, not Solana |
| **#13 Privacy Integration** | Keep, but implement in CosmWasm instead of Solana programs |
| **#14 AI Governance** | Keep, implement as Cosmos governance module + custom AI agent registry |
| **#15 Security Testing** | Keep, add mobile mining PoW testing |
| **#16 Mainnet Launch** | Update to include founding validator recruitment |

---

**cc: @jelly-legs-ai/core-team @jelly-legs-ai/protocol-team**

**Labels:** ~design ~decentralized-architecture ~cosmos-sdk ~phase3

/label ~cosmos-sdk-base
