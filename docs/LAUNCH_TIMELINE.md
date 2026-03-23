# Project AETHER — Mainnet Launch Timeline
## Phase 6 Deliverable 4/4

**Status:** PLANNED  
**Author:** 🚀 Launch-Pad (Phase 6 Mainnet Specialist)  
**Date:** 2026-03-22  
**Issue:** [#108](https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/108)

---

## 1. High-Level Timeline

```
Target Mainnet Genesis: Q2 2026 (June 2026)
Total Phase 6 Duration: ~10 weeks from 2026-03-22
```

**Phase 6 Milestones:**

| Week | Milestone | Target Date | Status |
|------|-----------|-------------|--------|
| 1 | Architecture finalized | 2026-03-29 | 🔄 In Progress |
| 2 | Security audit commissioned | 2026-04-05 | ⬜ Pending |
| 3 | Audit Phase A complete | 2026-04-19 | ⬜ Pending |
| 4 | P0/P1 fixes applied | 2026-04-26 | ⬜ Pending |
| 5 | Audit Phase B complete | 2026-05-10 | ⬜ Pending |
| 6 | Validator readiness confirmed | 2026-05-17 | ⬜ Pending |
| 7 | Genesis generation + test | 2026-05-24 | ⬜ Pending |
| 8 | Bug bounty live + RC released | 2026-05-31 | ⬜ Pending |
| 9 | Soft freeze — only P0 fixes | 2026-06-07 | ⬜ Pending |
| 10 | **MAINNET GENESIS** | **2026-06-14** | ⬜ Pending |

---

## 2. Detailed Week-by-Week Plan

### Week 1 (2026-03-22 → 2026-03-29) — Architecture Finalization

**Goals:**
- [x] Mainnet Architecture Document ✅ Created this document
- [ ] Confirm chain ID with all stakeholders
- [ ] Confirm genesis timestamp
- [ ] Finalize tokenomics (AETH + FLUX supply)
- [ ] Confirm founding validator list (need ≥10)

**Blockers to Clear:**
- Final validator commitments (10K AETH each)
- Genesis timestamp decision
- Chain ID confirmation

**Dependencies:** None — can proceed immediately

---

### Week 2 (2026-03-29 → 2026-04-05) — Audit Commissioning

**Goals:**
- [ ] Sign engagement letter with audit firm (Trail of Bits or Zellic recommended)
- [ ] Freeze codebase — begin bug-fix-only period
- [ ] Prepare public GitHub repository with scope document
- [ ] Launch Immunefi bug bounty program
- [ ] Run internal red-team exercise
- [ ] Complete `cargo audit`, `clippy --all-targets`, fuzzing runs

**Budget Needed:** $40,000–$80,000 (Trail of Bits or Zellic)

**Dependencies:** Week 1 complete

---

### Week 3–4 (2026-04-05 → 2026-04-19) — Auditor Review Phase A

**Goals:**
- [ ] Auditor conducts initial review
- [ ] Weekly sync calls to triage findings in real time
- [ ] P0/P1 findings classified and assigned to developers
- [ ] Internal team replicates all P0 findings

**Key Meetings:**
- Week 3: Kickoff call with auditors
- Week 3: Architecture deep-dive with auditors
- Week 4: Mid-point triage (all P0/P1 findings revealed)

**Dependencies:** Week 2 complete, engagement signed

---

### Week 5 (2026-04-19 → 2026-04-26) — P0/P1 Fixes

**Goals:**
- [ ] Fix all P0 (Critical) findings
- [ ] Fix all P1 (High) findings
- [ ] Internal verification of all fixes
- [ ] Auditor re-verifies critical fixes
- [ ] Begin validator final preparation

**Code Freeze Severity:** Only P0/P1 allowed, requires lead approval

---

### Week 6 (2026-04-26 → 2026-05-03) — Validator Readiness

**Goals:**
- [ ] All validators submit node setup confirmation
- [ ] Minimum 10 founding validators confirmed (must have ≥10 for genesis)
- [ ] Validator onboarding guide distributed
- [ ] Test genesis block generated and validated
- [ ] Distribute genesis configuration to all validators

**Dependencies:** All P0/P1 fixes verified by auditor

---

### Week 7 (2026-05-03 → 2026-05-10) — Genesis Generation

**Goals:**
- [ ] Generate genesis block with all validator pubkeys
- [ ] Verify genesis hash is deterministic and reproducible
- [ ] Test genesis on internal devnet (all validators)
- [ ] Confirm AETH + FLUX token distributions are correct
- [ ] Test recovery procedures (what if genesis node fails?)

**Dependencies:** Week 6 complete

---

### Week 8 (2026-05-10 → 2026-05-17) — RC Release + Bug Bounty

**Goals:**
- [ ] Publish Release Candidate (RC1) binary
- [ ] Open community testing (invite 100+ testers)
- [ ] Bug bounty fully operational on Immunefi
- [ ] Address any P2 issues found in RC1 testing
- [ ] Finalize FLUX/AETH distribution calculations

**Blocker for Next Week:** RC1 must pass 48h without P0/P1 issues

---

### Week 9 (2026-05-17 → 2026-05-24) — Soft Freeze

**Goals:**
- [ ] Code soft freeze — only P0 bug fixes allowed
- [ ] RC2 release (if needed)
- [ ] Validator final checklist submitted
- [ ] Genesis timestamp locked (e.g., June 14, 2026 12:00 UTC)
- [ ] Emergency contacts and incident response plan finalized
- [ ] All documentation published

**Dependencies:** Week 8 RC testing complete

---

### Week 10 (2026-05-24 → 2026-06-14) — Final Countdown

**2 Weeks Before Genesis:**

| Day | Action |
|-----|--------|
| -14 | Final security review |
| -14 | Announce genesis date publicly |
| -10 | Distribute final genesis config to all validators |
| -7 | Validator stress test (simulate network conditions) |
| -5 | RC final released |
| -3 | Final P0 check |
| -1 | Genesis block pre-generated, distributed |
| **0** | **🚀 MAINNET GENESIS — June 14, 2026** |

---

## 3. Launch Day Checklist

### T-24 Hours

- [ ] All validators online and healthy
- [ ] Emergency response team on standby
- [ ] Discord/Telegram announcements ready
- [ ] Block explorer running and healthy
- [ ] RPC endpoints responding
- [ ] Monitoring dashboards green

### T-1 Hour

- [ ] Final validator connectivity check
- [ ] Confirm genesis timestamp broadcast to all validators
- [ ] Core team all reachable
- [ ] Twitter/X announcement queued

### T=0 (Genesis)

```
Validator 1 — GO
Validator 2 — GO
Validator 3 — GO
...
Validator N — GO

🚀 LAUNCH COMMAND ISSUED
Genesis block produced: <hash>
First slot confirmed: <slot>
AETH token minted: <amount>
FLUX token deployed: <address>

WELCOME TO AETHER MAINNET
```

### T+1 Hour

- [ ] First blocks producing
- [ ] Transactions confirming
- [ ] Token transfers working
- [ ] Explorers indexing
- [ ] Social media announcements live

### T+24 Hours

- [ ] Network stable (>95% of validators online)
- [ ] No consensus failures
- [ ] Token transfers functioning
- [ ] Staking rewards calculating
- [ ] Incident report: none

---

## 4. Rollback Plan

**If a critical consensus bug is found before genesis:**
- All validators halt
- Emergency governance call (Discord #emergency)
- Bug fixed in source
- New genesis generated
- Restart countdown

**If a critical consensus bug is found after genesis (v1.0):**
- Activate Tower BFT rollback (up to 12 slots)
- If >12 slots: hard fork required
- Community vote on hard fork within 48h
- Coordinate with all validators for hard fork

**If a security exploit is found:**
- Emergency pause of affected contracts (if upgradeable)
- Coordinate with exchanges
- Bug bounty payouts for responsible disclosure
- Full incident report within 7 days

---

## 5. Post-Launch 30-Day Plan

| Day | Goal |
|-----|------|
| +1 | Monitor network stability, rewards distribution |
| +7 | First governance proposal (DAO activation) |
| +14 | Staking dashboard live |
| +30 | First protocol upgrade vote (if needed) |
| +30 | Treasury DAO governance fully activated |

---

## 6. Key Contacts for Launch Week

| Role | Name | Contact | Priority |
|------|------|---------|---------|
| Core Lead | TBD | TBD | P0 — Genesis trigger |
| Validator Liason | TBD | TBD | P0 — Validator coordination |
| Security | TBD | TBD | P0 — Incident response |
| Communications | TBD | TBD | P1 — Announcements |

---

## 7. Open Questions Before Timeline Locks

These must be answered in Week 1 before the rest of the timeline is firm:

1. **Genesis timestamp:** Exact date/time — proposed June 14, 2026
2. **Audit firm:** Confirm which firm (Trail of Bits vs Zellic vs other)
3. **Minimum validator count:** Is 10 enough? Or is 20+ needed?
4. **Token distribution:** Has treasury/team allocation been legally reviewed?
5. **FLUX token:** Who controls mint authority initially?
6. **Chain ID:** Confirm `aether-mainnet-1` or alternative?

---

## 8. Dependencies Summary

```
Week 1 ─────────────────────────────────────────────────────────►
Week 2 ──[Wk1]──────────────────────────────────────────────────►
Week 3 ──────[Wk2]────────────────────────────────────────────────►
Week 4 ──────[Wk2]────────────────────────────────────────────────►
Week 5 ─────────────[Wk3-4]──────────────────────────────────────►
Week 6 ──────────────────[Wk5]──────────────────────────────────►
Week 7 ──────────────────────[Wk6]───────────────────────────────►
Week 8 ──────────────────────────[Wk7]──────────────────────────►
Week 9 ─────────────────────────────[Wk8]──────────────────────►
Week 10 ─────────────────────────────────────[Wk9]──────────────►
GENESIS ──────────────────────────────────────────────────[Wk9]──► Jun 14
```

---

*Document version: 1.0 — 🚀 Launch-Pad | Phase 6*
