## 🎨 DESIGN COMMENT 2/5: Reward System Design

### Benchmark Tests by Tier

**Tier 1 - Mobile (Phone/Tablet):**
- CPU benchmark: Solve SHA-256 puzzle variants (difficulty adjusted for mobile SoC)
- Memory benchmark: Sort/filter 100K records (tests RAM bandwidth)
- Proof-of-Storage: Merkle proof generation for delegated data
- Score range: 1-100 points/hour based on completion time and consistency

**Tier 2 - Laptop:**
- CPU: Multi-threaded SHA-256 + AES encryption rounds
- GPU (if discrete): CUDA/OpenCL proof-of-GPU compute tasks
- Memory: 500K record operations
- Score range: 100-500 points/hour

**Tier 3 - Desktop:**
- Full CPU suite + optional GPU acceleration
- Memory: 1M+ record operations
- Network: Relay capability tests (serve as sync relays for mobile nodes)
- Score range: 500-2000 points/hour

**Tier 4 - Server:**
- Full node duties: block validation, transaction processing
- Specialized: ZK proof generation assistance
- Score range: 2000-10000 points/hour

### Reward Calculation Formula

```
DailyReward = BaseReward × TrustMultiplier × TierMultiplier × ActiveHours

Where:
- BaseReward = 10 $AETH (network-defined daily reward pool / active nodes)
- TrustMultiplier = 0.5 to 1.5 (based on 30-day rolling trust score)
- TierMultiplier = {Mobile: 1.0, Laptop: 2.5, Desktop: 5.0, Server: 10.0}
- ActiveHours = min(actual_hours_mined_today / 24, 1.0) capped at 100%

Additionally:
- $COMPUTE credits earned = TierMultiplier × ActiveHours × 0.1 (per day)
```

**Example:**
- Mobile node, trust score 0.9, mined 18 hours
- Reward = 10 × 0.9 × 1.0 × 0.75 = 6.75 $AETH/day
- COMPUTE earned = 1.0 × 0.75 × 0.1 = 0.075 $COMPUTE

### Payout Schedule

- **Daily settlement:** Rewards calculated at 00:00 UTC, claims available after 12:00 UTC
- **Weekly distribution:** Main payout batch every Monday
- **Minimum payout:** 10 $AETH (to avoid dust/UX problems)
- **Distribution method:** On-chain claim (pull over push - user claims, not auto-sent)

### Anti-Gaming Mechanisms

1. **Proof-of-Device:** Hardware attestation via SafetyNet/DeviceCheck (prevents emulator farming)
2. **Behavioral Analysis:** ML model detects abnormal patterns (too consistent, perfect timing, impossible speeds)
3. **Sybil Resistance:** Account age requirements (min 7 days before earning), social graph verification
4. **Benchmark Variance:** Random difficulty adjustments prevent pre-computation
5. **Human Verification:** Periodic captcha/photo verification (monthly)
6. **Stake Requirement:** Lock 100 $AETH to enable earning (slashed for cheating)
7. ** audits:** Random audit sampling where nodes must re-run proofs on-demand