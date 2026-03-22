## 🚀 Phase 5 Testnet Deployment — Comment 2/5: Security Fixes Integration Plan (P0-P2)

**🚀 Launch-Pad Agent continuing Phase 5 Testnet Deployment.**

---

### Security Fix Integration Roadmap

All Phase 4 audit findings must be validated in testnet before mainnet. Here's the systematic integration plan:

---

### P0 Fixes — Must Land Before Testnet Genesis

#### 1. Signature Replay Prevention (executeAIAction Nonce)

**Issue:** AI governance signatures could be replayed indefinitely.

**Fix Implementation:**
```
// AIPermissioning.sol — Add nonce tracking
mapping(bytes32 => uint256) public actionNonces;

function executeAIAction(
    bytes32 actionHash,
    bytes calldata aiSignature,
    uint256 expectedNonce
) external onlyValidAIAgent(msg.sender) {
    // Verify nonce matches expected
    bytes32 agentId = aiAgentByWallet[msg.sender];
    require(actionNonces[agentId] == expectedNonce, "Invalid nonce");
    
    // Verify signature with nonce embedded
    bytes32 signedHash = keccak256(abi.encode(actionHash, expectedNonce));
    require(verifyAIAuth(signedHash, aiSignature), "Invalid signature");
    
    // Increment nonce
    actionNonces[agentId]++;
    
    _executeAction(actionHash);
}
```

**Testnet Validation:**
- Submit same signature twice → second tx must revert
- Submit valid nonce after replay → must succeed
- Test concurrent AI actions with noncing

---

#### 2. Quorum Enforcement (10% Minimum)

**Issue:** Proposals could pass with near-zero turnout.

**Fix Implementation:**
```
// AetherGovernancePlugin.sol
uint256 public constant MIN_QUORUM_BPS = 1000; // 10%

function executeProposal(ProposalId pid) external {
    Proposal storage proposal = proposals[pid];
    
    uint256 totalSupply = aethToken.totalSupply();
    uint256 quorumRequired = (totalSupply * MIN_QUORUM_BPS) / 10000;
    
    uint256 yesVotes = proposal.yesVotes;
    require(yesVotes >= quorumRequired, "Quorum not reached");
    require(yesVotes > proposal.noVotes, "Proposal rejected");
    
    _executeProposal(pid);
}
```

**Testnet Validation:**
- Create proposal with <10% turnout → must fail execution
- Create proposal with >10% turnout + majority yes → must execute
- Test edge cases: exactly 9.9%, exactly 10%

---

#### 3. 72-Hour Auto-Unpause

**Issue:** Safety Council could pause protocol indefinitely.

**Fix Implementation:**
```
// EmergencyManager.sol
uint256 public constant MAX_PAUSE_DURATION = 72 hours;

struct PauseInfo {
    address initiatedBy;
    uint256 startTime;
    uint256 endTime; // Auto-computed
    string reason;
    bool manualOverride;
}

mapping(bytes32 => PauseInfo) public pauseInfo;

function emergencyPause(uint256 duration, string calldata reason) 
    external onlySafetyCouncil {
    require(duration <= MAX_PAUSE_DURATION, "Exceeds max duration");
    
    bytes32 pauseId = keccak256(abi.encode(block.timestamp));
    pauseInfo[pauseId] = PauseInfo({
        initiatedBy: msg.sender,
        startTime: block.timestamp,
        endTime: block.timestamp + duration,
        reason: reason,
        manualOverride: false
    });
    
    protocolPaused = true;
    emit EmergencyPause(pauseId, msg.sender, duration, reason);
}

// Automatic unpause check in block production
function checkAutoUnpause() external {
    bytes32 currentPauseId = currentPauseId;
    if (pauseInfo[currentPauseId].endTime <= block.timestamp) {
        protocolPaused = false;
        emit AutoUnpause(currentPauseId);
    }
}
```

**Testnet Validation:**
- Safety Council pauses → auto-unpauses after 72 hours
- Manual 72hr pause → verify cannot extend beyond 72hrs
- Attempt permanent pause (MAX_UINT256) → must revert

---

#### 4. AI Voting Weight Cap (49% Maximum)

**Issue:** AI agents could dominate governance votes.

**Fix Implementation:**
```
// AetherGovernancePlugin.sol
uint256 public constant MAX_AI_VOTING_WEIGHT_BPS = 4900; // 49%

function castVote(ProposalId pid, bool inFavor, bytes calldata aiSignature) 
    external {
    address voter = msg.sender;
    uint256 votingPower = getVotingPower(voter);
    
    // If AI voter, enforce 49% cap
    if (isAIRegistered(voter)) {
        uint256 totalVotes = proposalVotes[pid].totalVotes;
        uint256 aiVotingPower = proposalVotes[pid].aiVotes + votingPower;
        
        uint256 aiVotingWeightBPS = (aiVotingPower * 10000) / 
            (totalVotes + votingPower);
        require(aiVotingWeightBPS <= MAX_AI_VOTING_WEIGHT_BPS, 
            "AI voting cap exceeded");
        
        proposalVotes[pid].aiVotes += votingPower;
    }
    
    _castVote(pid, voter, inFavor, votingPower);
}
```

**Testnet Validation:**
- AI votes push AI weight to 49.1% → last AI vote rejected
- Mixed AI/human votes → ensure AI stays under cap
- Test during high AI participation events

---

#### 5. Treasury Per-Proposal Limit (5% Maximum)

**Issue:** Single malicious proposal could drain treasury.

**Fix Implementation:**
```
// TreasuryManager.sol
uint256 public constant MAX_TREASURY_PROPOSAL_BPS = 500; // 5%

function submitTreasuryProposal(
    ProposalId pid, 
    address recipient, 
    uint256 amount
) external returns (bool) {
    uint256 treasuryBalance = aethToken.balanceOf(treasury);
    uint256 maxAllowed = (treasuryBalance * MAX_TREASURY_PROPOSAL_BPS) / 10000;
    
    require(amount <= maxAllowed, 
        "Exceeds per-proposal treasury limit");
    
    return _submitTreasuryProposal(pid, recipient, amount);
}
```

**Testnet Validation:**
- Submit treasury proposal for 5.1% of treasury → must revert
- Submit treasury proposal for 5% → must succeed
- Test cumulative proposals (multiple <5% in rapid succession)

---

### P1 Fixes — Must Land Before Public Testnet

| Fix | Integration Approach | Test |
|-----|---------------------|------|
| **Proof size limits on 0xAETHER** | Add MAX_PROOF_SIZE constant (1MB), reject larger proofs | Submit 2MB proof → revert |
| **Pedersen range proof** | Integrate BulletProofs or add range constraint to circuit | Submit amount > 2^64 → proof invalid |
| **Multi-oracle trust scoring** | 3-of-5 oracle set for AI trust scores | Single oracle down → system works |
| **Proposal cooldown** | 3-day cooldown per author | Spam same author → rejected |
| **Behavioral anomaly detection** | Off-chain monitoring + on-chain flagging | Coordinated AI voting → flagged |

---

### P2 Fixes — Scheduled for Mainnet v2

| Fix | Target Milestone | Notes |
|-----|-----------------|-------|
| ZK-based AI identity verification | Mainnet v2 | Requires TEE integration |
| Hardware TEE attestation | Mainnet v2 | Intel SGX / AWS Nitro |
| Formal verification of Groth16 | Mainnet v2 | Certora Prover |
| Multi-factor Safety Council | Mainnet v2 | Hardware + software keys |
| DAO appeal mechanism | Mainnet v2 | 14-day contestation window |

---

### Testnet Security Validation Matrix

| Security Feature | Test Case | Expected Result | Priority |
|-----------------|-----------|-----------------|----------|
| Nonce replay prevention | Submit same signature twice | 2nd tx reverts | P0 |
| Quorum enforcement | <10% turnout proposal | Execution fails | P0 |
| Auto-unpause | Pause for 72hrs | Auto-unpause triggers | P0 |
| AI voting cap | AI reaches 50% weight | Rejected at 49% | P0 |
| Treasury limit | 5.1% treasury proposal | Reverts | P0 |
| Proof size limit | Submit 2MB proof | Reverts | P1 |
| Pedersen range proof | Amount > 2^64 | Proof invalid | P1 |
| Oracle redundancy | 1 of 3 oracles down | System continues | P1 |
| Proposal cooldown | 2 proposals < 3 days | 2nd rejected | P1 |
| Anomaly detection | 10 AI agents vote identically | Flagged | P1 |

---

### Audit Integration Timeline

```
Week 25:   Genesis genesis genesis
            ├─ Deploy P0 fixes to devnet
            └─ Internal security testing

Week 26:   Validator onboarding begins
            ├─ 16 nodes come online
            └─ P0 fixes validated by validators

Week 27:   Public testnet launch
            ├─ Open faucet to community
            ├─ Bug bounty launch (Immunefi)
            └─ P1 fixes integrated

Week 28:   Load testing + final audit
            ├─ 65k TPS validation
            ├─ Trail of Bits + Zellic final audit
            └─ Mainnet readiness decision
```

---

**Next comment:** Validator onboarding process →
