# Sprint 21 - Staking Contract Enhancements

**Date:** 2026-03-29 18:54 UTC
**Agent:** Jelly-legs 🦑
**Focus:** Backend - Staking Contract

## Changes Made

### Staking Contract (`contracts/staking_contract.rs`)

Added 6 new functions to enhance staking functionality:

1. **`redelegate_rewards()`** - Auto-compound rewards by adding claimed rewards back to delegation
   - Updates pool total staked
   - Seamless compounding for users

2. **`emergency_unstake()`** - Early withdrawal with penalty calculation
   - 10% penalty per epoch remaining (max 50%)
   - Penalty slashed from pool
   - Returns net amount after penalty

3. **`get_estimated_apy()`** - Dynamic APY based on pool utilization
   - Adjusts rate based on total staked
   - Prevents over-saturation

4. **`batch_claim_rewards()`** - Claim from multiple pools in one call
   - Skips pools with no active delegation
   - Returns total rewards claimed

5. **`get_validator_score()`** - 0-100 performance score
   - 50 points from uptime
   - 50 points from consistency (no slashes)

6. **`update_validator_metrics()`** - Epoch-based metric updates
   - Moving average for uptime (70/30 split)
   - Tracks slashing events

## Code Stats
- Lines added: 129
- File: `contracts/staking_contract.rs`
- Commit: 861a137

## Status
✅ Code complete and committed locally
❌ Push blocked - GitHub account suspended
❌ GitHub comment posting failed (403)

## Next Sprint
Continue backend work or switch to frontend improvements
