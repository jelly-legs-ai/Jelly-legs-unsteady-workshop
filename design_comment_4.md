## 🎨 DESIGN COMMENT 4/5: Dual-Token Economics

### $AETH - Governance Token

**Token Specifications:**
- **Type:** SPL Token (Solana-compatible)
- **Total Supply:** 1,000,000,000 (1B) initial, max 2B over 20 years via inflation
- **Decimals:** 9
- **Contract Address:** [TBD on mainnet launch]
- **Blockchain:** AETHER native chain (Solana fork)

**Initial Distribution (TGE):**
- Community Mining Rewards: 40% (400M $AETH) - distributed over 20 years
- Founding Team: 15% (150M) - 4-year vest, 1-year cliff
- Early Investors: 10% (100M) - 2-year vest, 6-month cliff
- Treasury: 20% (200M) - governance controlled
- Ecosystem Fund: 10% (100M) - grants, partnerships, liquidity
- Public Sale: 5% (50M) - initial DEX offering

**Utility:**
- Staking for validators (required for block rewards)
- Governance voting (1 token = 1 vote)
- Fee discount (up to 50% when paying fees in $AETH)
- Collateral for $COMPUTE purchases

**Inflation Schedule:**
- Year 1: 5% annual inflation
- Year 2-5: Inflation decreases by 0.5% per year (floor 2%)
- Year 6+: 2% fixed (sustains validator ecosystem)
- Inflation adjusts quarterly based on staking participation rate

### $COMPUTE - Gas/Credits Token

**Token Specifications:**
- **Type:** SPL Token (separate mint)
- **Total Supply:** Elastic (mined/purchased, not pre-mined)
- **Decimals:** 6
- **Purpose:** Payment for AI compute resources on the network

**Emission Model:**
- Earned by: Mobile/laptop/desktop miners (secondary reward)
- Purchased by: AI agents needing compute, DApps needing gas
- Burned on: AI task execution, smart contract calls, data storage

**Pricing Mechanism:**
- $COMPUTE/$AETH price discovered via bonding curve
- Initial price: 1 $COMPUTE = 0.001 $AETH
- Bonding curve ensures price stability during growth
- Market makers provide liquidity on DEX

**Use Cases:**
1. AI Task Payment: Agent pays $COMPUTE to submit tasks to mobile miners
2. Gas Fees: All transactions burn small amount of $COMPUTE
3. Storage: Per-byte $COMPUTE burn for on-chain data storage
4. Priority: Higher $COMPUTE tips = faster task processing

### Reward Distribution by Token

**Mobile Miners:**
- 80% $AETH, 20% $COMPUTE (by value)
- $AETH encourages long-term holding (governance participation)
- $COMPUTE provides immediate utility (can sell or use for gas)

**Validators:**
- 100% $AETH (block rewards + fees)
- No $COMPUTE (they are infrastructure, not compute providers)
- Staking yields denominated in $AETH only

**AI Agents / DApp Users:**
- Pay for compute in $COMPUTE
- Gas can be paid in $AETH or $COMPUTE (user choice)
- $AETH holders get 50% gas discount

**Example Transaction Flow:**
1. AI Agent submits task, deposits 1000 $COMPUTE into escrow
2. Mobile miners complete task, earn 800 $AETH + 200 $COMPUTE total
3. AI Agent receives results, releases escrow
4. 50 $COMPUTE burned as network fee (5%)
5. Remaining 950 $COMPUTE returned or distributed based on task quality