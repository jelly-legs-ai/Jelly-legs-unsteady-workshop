# Immunefi Bug Bounty Setup — 5-Minute Guide

## What to Set Up at Immunefi.com

Immunefi is the standard bug bounty platform for web3. Setting up takes ~5 minutes once you have:
- Your audit scope (see `docs/AUDIT_SCOPE.md`)
- Severity/reward matrix
- Contract addresses (can add after mainnet)

---

## Step 1: Create Account
1. Go to https://immunefi.com
2. Click "Add Project" or sign up as a protocol team
3. Select "Smart Contract" as the project type

## Step 2: Project Settings

**Project Name:** AETHER  
**Tagline:** Solana-fork blockchain optimized for AI workloads  
**Website:** (your website)  
**Documentation:** (link to docs)  
**Repository:** https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop  

## Step 3: Scope

Import from `docs/AUDIT_SCOPE.md` — the "In Scope" contracts table.

**Critical contracts to declare:**
- Staking contract (tiered rewards, auto-compound)
- Mining rewards contract (streak/geo/reputation bonuses)
- FLUX/ATH token contracts (mint/burn/transfer)
- Governance treasury (multi-sig)
- Bridge contract

## Step 4: Severity Matrix

| Severity | Reward |
|----------|--------|
| Critical | $5,000 |
| High | $2,000 |
| Medium | $500 |
| Low | $100 |

## Step 5: Program Rules

**Out of scope:**
- Frontend/UI
- Testnet contracts
- Third-party dependencies (libp2p, tokio, etc.)
- Social engineering

**Disclosure policy:** Coordinated disclosure within 24 hours of fix

---

## After Setup

Once your project is live on Immunefi:
1. Share the URL in your Discord/Twitter
2. Add to your website
3. Monitor submissions via Immunefi dashboard

---

## Alternative: Audit Competition

If you want faster coverage, Immunefi also offers "Audit Competitions" where multiple researchers review simultaneously. Cost: typically $5-10K (above our budget), but faster turnaround. Worth revisiting if the bug bounty doesn't attract researchers.
