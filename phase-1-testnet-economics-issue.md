## Overview

Implement the testnet mining and contributor reward system. This is the economic foundation — the system that incentivizes people to run validator nodes and contribute compute to secure the AETHER testnet.

---

## Mining Rate Specification

### Base Rate (Handheld Devices)
- **0.001 FLUX/hour** — baseline for mobile/low-power devices
- Rate is constant, accrues while node is online

### Streak Bonuses (Continuous Mining)
| Streak | Multiplier | Effective Rate |
|--------|------------|----------------|
| < 30 days | 1.0x | 0.001 FLUX/hr |
| 30–59 days | 1.5x | 0.0015 FLUX/hr |
| 60–89 days | 2.0x | 0.002 FLUX/hr |
| 90+ days | 2.2x | 0.0022 FLUX/hr |

Streak resets if node is offline >4 hours.

### Referral Program (Security Circle)
- Referrer gets **+20% boost** to their mining rate per active referral
- A referral becomes "active" after the referred user accumulates 24h uptime
- No limit on referrals
- Example: base 0.001 + (5 active referrals × 0.0002) = 0.002 FLUX/hr

### Claim Lock
- **Minimum 30 days** before first claim
- After 30 days: claim anytime, no forced lock
- No partial claims — claim all accrued rewards

### Lockup Option (Security Boost)
- User can lock 100% of accrued rewards
- **While locked: 350% mining boost** (until mainnet launch)
- Locked rewards are staked against $ATH
- Users earn $ATH on their locked $FLUX position (rate: TBD by governance)
- Locked rewards cannot be transferred but can be claimed post-mainnet

### Ambassador Program
Unlocked when a contributor has ≥3 active referrals:

| Role | Per-Hour Bonus |
|------|---------------|
| Contributor (base) | 0.001 FLUX/hr |
| +3 active referrals → **Ambassador** | +1.00 FLUX/hr flat |
| Per active referral (ambassador) | +0.25 FLUX/hr |

Example: Ambassador with 10 active referrals = 0.001 + 1.00 + (10 × 0.25) = 3.501 FLUX/hr

### Persistent Node Bonuses
For contributors running persistent (always-on) validator nodes:

| Node Reliability Score | Bonus Multiplier |
|------------------------|------------------|
| < 50% uptime | 0.5x |
| 50–80% uptime | 1.0x |
| 80–95% uptime | 1.5x |
| 95–99% uptime | 2.0x |
| 99%+ uptime | 2.5x |

Reliability score = (total_uptime_hours / total_enrolled_hours) adjusted for network latency.

### Node Computational Power Bonus
Additional bonus based on device capability (measured at enrollment):

| Capability Tier | Criteria | Bonus |
|-----------------|----------|-------|
| Light | Mobile/ARM, < 2GB RAM | 1.0x |
| Standard | Desktop, 2–8GB RAM | 1.5x |
| Performance | Desktop, 8–32GB RAM, GPU | 2.0x |
| Validator | Server, 32GB+ RAM, high bandwidth | 3.0x |

Score = Reliability_Multiplier × Capability_Multiplier × Streak_Multiplier × Referral_Count

---

## Technical Requirements

### Data Model

```rust
struct MinerInfo {
    address: Pubkey,
    enrolled_at: u64,           // Unix timestamp
    last_active_at: u64,        // Unix timestamp
    total_uptime_hours: f64,
    current_streak_days: u32,
    current_streak_hours: u64,
    base_rate: u64,             // in micro-FLUX
    referral_count: u32,
    active_referral_count: u32,
    ambassador_level: u8,       // 0 = none, 1 = ambassador
    locked_rewards: u64,        // Amount locked for security boost
    is_locked: bool,
    lock_end_timestamp: u64,
    node_tier: NodeTier,       // Light, Standard, Performance, Validator
    reliability_score: f64,
    total_earned: u64,
    total_claimed: u64,
}

enum NodeTier {
    Light,
    Standard,
    Performance,
    Validator,
}
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/miner/enroll` | POST | Enroll as miner, set node tier |
| `/api/miner/status` | GET | Get current mining status, rate, streak |
| `/api/miner/heartbeat` | POST | Report uptime (call every 5 min) |
| `/api/miner/claim` | POST | Claim accrued rewards |
| `/api/miner/lockup` | POST | Lock rewards for 350% boost |
| `/api/miner/referral` | GET | Get referral link and stats |
| `/api/miner/leaderboard` | GET | Top miners by rate |

### Implementation Checklist

- [ ] `contracts/mining_contract.rs` — add miner enrollment, heartbeat tracking
- [ ] Streak tracking: reset on >4h offline gap
- [ ] Referral tracking: credit referrer when referee hits 24h uptime
- [ ] Lockup mechanism: freeze claims, apply 3.5x multiplier
- [ ] Ambassador unlocks at 3 active referrals
- [ ] Reliability scoring: rolling 7-day window
- [ ] Node tier detection at enrollment
- [ ] Rate calculation: base × streak × referral × ambassador × node_reliability
- [ ] Frontend: mining dashboard showing all metrics
- [ ] Tests: streak reset, referral credit, lockup boost calculation

---

## Integration Points

- Mining contract integrates with **staking contract** for lockup staking
- Ambassador bonuses credited from **treasury** (governance-approved budget)
- $ATH rewards for locked FLUX come from **governance pool**

---

## Priority

After Phase 0 (testnet runs). This is the economic engine — nothing else matters for testnet validation without it.
