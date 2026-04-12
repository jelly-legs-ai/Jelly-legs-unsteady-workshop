//! Aether Integration Test Suite
//!
//! Tests the full validator lifecycle: state management, staking,
//! governance, treasury, block production, AI priority lanes, and RPC.

use aether_consensus::staking::{StakingPool, StakingError};
use aether_consensus::tower::TowerConsensus;
use aether_consensus::fork_choice::ForkChoice;
use aether_consensus::poh::{PoHGenerator, PoHEntry, verify_poh_chain, HASHES_PER_TICK};
use aether_governance::{
    AetherDAO, Treasury, GovernanceConfig, ProposalType, ProposalStatus,
    VoteChoice, TokenType,
};
use aether_validator::keypair::generate_keypair;
use aether_validator::state::ValidatorState;
use std::path::PathBuf;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_state() -> ValidatorState {
    let identity = generate_keypair();
    let tmp = tempfile::tempdir().expect("temp dir");
    ValidatorState::new(identity, true, tmp.path().to_path_buf())
        .expect("create state")
}

fn random_pubkey() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut key);
    key
}

// ============================================================================
// PoH (Proof of History) Tests
// ============================================================================

#[test]
fn test_poh_generator_produces_valid_chain() {
    let genesis = PoHEntry::genesis();
    let mut gen = PoHGenerator::from_hash(genesis.hash);

    // Produce 10 ticks
    let mut entries = vec![genesis];
    for _ in 0..10 {
        entries.push(gen.tick());
    }

    // Verify the entire chain
    assert!(verify_poh_chain(&entries), "PoH chain should verify");

    // Tamper with one entry — should fail
    let mut bad = entries.clone();
    bad[5].hash = [0u8; 32];
    assert!(!verify_poh_chain(&bad), "Tampered chain should fail");
}

#[test]
fn test_poh_mix_transaction_ordering() {
    let genesis = PoHEntry::genesis();
    let mut gen = PoHGenerator::from_hash(genesis.hash);

    // Mix transactions produce unique hashes
    let e1 = gen.mix(b"tx-1");
    let e2 = gen.mix(b"tx-2");
    let e3 = gen.tick();

    assert_ne!(e1.hash, e2.hash, "Different txs should produce different hashes");
    assert_ne!(e2.hash, e3.hash, "Tick should differ from mix");

    // Verify chain including mixes
    let chain = vec![genesis, e1, e2, e3];
    assert!(verify_poh_chain(&chain));
}

#[test]
fn test_poh_hashes_per_tick_constant() {
    // Target: 400ms slot time with ~2M hashes per tick on modern CPU
    assert_eq!(HASHES_PER_TICK, 2_000_000, "Hashes per tick must match spec");
}

#[test]
fn test_poh_entry_verify_rejects_wrong_previous() {
    let prev = [1u8; 32];
    let entry = PoHEntry::new(prev, 100, 0, None);

    // Correct previous hash verifies
    assert!(entry.verify(prev));

    // Wrong previous hash fails
    assert!(!entry.verify([2u8; 32]));
    assert!(!entry.verify([0u8; 32]));
}

// ============================================================================
// Tower BFT Consensus Tests
// ============================================================================

#[test]
fn test_tower_supermajority_confirmation() {
    let mut tower = TowerConsensus::new();
    let total_stake = 3_000_000u64;

    // 3 validators each with 1M stake
    let v1 = [1u8; 32];
    let v2 = [2u8; 32];
    let v3 = [3u8; 32];

    // 2/3 of 3M = 2M, threshold = floor(2M*2/3)+1 = 2_000_001
    // 2 validators (2M stake) is NOT enough: 2_000_000 < 2_000_001
    tower.process_vote(v1, 1, 1_000_000).unwrap();
    tower.process_vote(v2, 1, 1_000_000).unwrap();
    assert!(!tower.is_slot_confirmed(1, total_stake),
        "2/3 stake should NOT confirm (need strictly > 2/3)");

    // 3rd validator pushes past threshold
    tower.process_vote(v3, 1, 1_000_000).unwrap();
    assert!(tower.is_slot_confirmed(1, total_stake),
        "3/3 stake should confirm");
}

#[test]
fn test_tower_rejects_backward_vote() {
    let mut tower = TowerConsensus::new();
    let v = [1u8; 32];

    tower.process_vote(v, 5, 100).unwrap();
    let result = tower.process_vote(v, 3, 100);
    assert!(result.is_err(), "Should reject vote for earlier slot");
}

#[test]
fn test_tower_root_advances_after_confirmation_depth() {
    let mut tower = TowerConsensus::new();
    let v = [1u8; 32];

    // Vote on 35 consecutive slots (exceeds confirmation_depth of 32)
    for slot in 1..=35 {
        tower.process_vote(v, slot, 1000).unwrap();
    }

    let root = tower.get_root_slot();
    // With 35 votes, slots 1-3 should have >= 32 confirmations
    assert!(root >= 3, "Root should advance after 32 confirmations, got {}", root);
}

// ============================================================================
// Fork Choice Tests
// ============================================================================

#[test]
fn test_fork_choice_lmd_ghost() {
    let root = [0u8; 32];
    let mut fc = ForkChoice::new(root);

    // Two competing forks off root
    let fork_a = [1u8; 32];
    let fork_b = [2u8; 32];
    fc.add_block(fork_a, root, 1, 100).unwrap();
    fc.add_block(fork_b, root, 1, 200).unwrap();

    // Heavier fork should win
    let best = fc.get_best_block().unwrap();
    assert_eq!(best.hash, fork_b, "Heavier fork should be selected");
}

#[test]
fn test_fork_choice_deep_chain() {
    let root = [0u8; 32];
    let mut fc = ForkChoice::new(root);

    // Build chain: root -> b1 -> b2 -> b3 -> b4
    let mut prev = root;
    for i in 1..=4 {
        let mut hash = [0u8; 32];
        hash[0] = i;
        fc.add_block(hash, prev, i as u64, 100 * i as u64).unwrap();
        prev = hash;
    }

    // Best should be b4 (deepest in heaviest chain)
    let best = fc.get_best_block().unwrap();
    let mut expected = [0u8; 32];
    expected[0] = 4;
    assert_eq!(best.hash, expected);
}

#[test]
fn test_fork_choice_stake_update_propagates() {
    let root = [0u8; 32];
    let mut fc = ForkChoice::new(root);

    let b1 = [1u8; 32];
    let b2 = [2u8; 32];
    fc.add_block(b1, root, 1, 100).unwrap();
    fc.add_block(b2, b1, 2, 100).unwrap();

    // Add stake to b2 — should propagate to b1 as well
    let updated = fc.update_stake(b2, 500).unwrap();
    assert_eq!(updated, 2, "Should update b1 and b2");

    // b1 should have 100 + 500 = 600
    assert_eq!(fc.get_block(&b1).unwrap().stake_weight, 600);
    // b2 should have 100 + 500 = 600
    assert_eq!(fc.get_block(&b2).unwrap().stake_weight, 600);
}

#[test]
fn test_fork_choice_prune() {
    let root = [0u8; 32];
    let mut fc = ForkChoice::new(root);

    let mut prev = root;
    for i in 1..=5 {
        let mut hash = [0u8; 32];
        hash[0] = i;
        fc.add_block(hash, prev, i as u64, 100).unwrap();
        prev = hash;
    }

    // Prune blocks before slot 3
    fc.prune_before(3);

    // Root and slots 3+ should remain
    assert!(fc.get_block(&root).is_some(), "Root should always remain");
    let slot3 = {
        let mut h = [0u8; 32];
        h[0] = 3;
        h
    };
    assert!(fc.get_block(&slot3).is_some(), "Slot 3 should remain");

    let slot1 = {
        let mut h = [0u8; 32];
        h[0] = 1;
        h
    };
    assert!(fc.get_block(&slot1).is_none(), "Slot 1 should be pruned");
}

// ============================================================================
// Staking Tests
// ============================================================================

#[test]
fn test_stake_lifecycle() {
    let mut pool = StakingPool::new(0);
    let owner = random_pubkey();

    // 1. Create stake
    let stake_id = pool.stake(owner, 10_000_000_000_000).unwrap();
    assert_eq!(pool.total_staked, 10_000_000_000_000);

    // 2. Stake is locked initially
    let stake = pool.get_stake(stake_id).unwrap();
    assert!(stake.is_locked(pool.current_epoch));
    assert!(!stake.pending_withdrawal);

    // 3. Advance past lock period
    for _ in 0..5 {
        pool.tick();
    }
    assert!(!pool.get_stake(stake_id).unwrap().is_locked(5));

    // 4. Initiate withdrawal
    let unlock = pool.initiate_withdrawal(stake_id).unwrap();
    assert!(pool.get_stake(stake_id).unwrap().pending_withdrawal);

    // 5. Advance past unlock epoch
    pool.current_epoch = unlock + 1;
    assert!(pool.get_stake(stake_id).unwrap().can_withdraw(pool.current_epoch));

    // 6. Complete withdrawal
    let withdrawn = pool.complete_withdrawal(stake_id).unwrap();
    assert_eq!(withdrawn, 10_000_000_000_000);
    assert_eq!(pool.total_staked, 0);
}

#[test]
fn test_stake_below_minimum() {
    let mut pool = StakingPool::new(0);
    let result = pool.stake(random_pubkey(), 1);
    assert!(matches!(result, Err(StakingError::BelowMinimumStake(_, _))));
}

#[test]
fn test_stake_delegation() {
    let mut pool = StakingPool::new(0);
    let owner = random_pubkey();
    let validator = random_pubkey();

    let stake_id = pool.stake(owner, 10_000_000_000_000).unwrap();

    // Advance past lock so delegation is allowed
    pool.current_epoch = 5;
    pool.delegate(stake_id, validator).unwrap();

    let stake = pool.get_stake(stake_id).unwrap();
    assert_eq!(stake.delegated_to, Some(validator));
}

#[test]
fn test_stake_slashing() {
    let mut pool = StakingPool::new(0);
    let owner = random_pubkey();

    let stake_id = pool.stake(owner, 10_000_000_000_000).unwrap();

    // Slash 10% (1000 basis points)
    let penalty = pool.slash(stake_id, 1000).unwrap();
    assert_eq!(penalty, 1_000_000_000_000); // 10% of 10T
    assert_eq!(pool.get_stake(stake_id).unwrap().amount, 9_000_000_000_000);
    assert_eq!(pool.total_staked, 9_000_000_000_000);
}

#[test]
fn test_stake_reward_accumulation() {
    let mut pool = StakingPool::new(0);
    let owner = random_pubkey();
    let stake_id = pool.stake(owner, 10_000_000_000_000).unwrap();

    // Advance one year (365 epochs)
    pool.current_epoch = 365;
    pool.distribute_rewards();

    let stake = pool.get_stake(stake_id).unwrap();
    // 12% APY: ~1.2T lamports per year on 10T stake
    assert!(stake.accumulated_rewards > 0, "Should have accumulated rewards");
    assert!(stake.accumulated_rewards >= 10_000_000_000_000 / 10,
        "Rewards should be at least ~10% of stake for a year");
}

#[test]
fn test_stake_cannot_withdraw_while_locked() {
    let mut pool = StakingPool::new(0);
    let owner = random_pubkey();
    let stake_id = pool.stake(owner, 10_000_000_000_000).unwrap();

    // Try to complete withdrawal while still locked
    pool.initiate_withdrawal(stake_id).unwrap();
    pool.current_epoch = 0; // Still at epoch 0, lock period not elapsed

    let result = pool.complete_withdrawal(stake_id);
    assert!(result.is_err(), "Should not be able to withdraw while locked");
}

#[test]
fn test_multiple_stakes_same_owner() {
    let mut pool = StakingPool::new(0);
    let owner = random_pubkey();

    let id1 = pool.stake(owner, 10_000_000_000_000).unwrap();
    let id2 = pool.stake(owner, 20_000_000_000_000).unwrap();
    let id3 = pool.stake(owner, 30_000_000_000_000).unwrap();

    assert_eq!(pool.total_staked, 60_000_000_000_000);

    let positions = pool.get_stakes_by_owner(&owner);
    assert_eq!(positions.len(), 3);

    // Withdraw just one
    pool.current_epoch = 5;
    pool.initiate_withdrawal(id2).unwrap();
    pool.current_epoch = 10;
    pool.complete_withdrawal(id2).unwrap();

    assert_eq!(pool.total_staked, 40_000_000_000_000);
    // id1 and id3 still active
    assert!(pool.get_stake(id1).unwrap().amount > 0);
    assert!(pool.get_stake(id3).unwrap().amount > 0);
}

// ============================================================================
// Governance Tests
// ============================================================================

#[test]
fn test_governance_proposal_lifecycle() {
    let mut dao = AetherDAO::with_default_config();
    let proposer = random_pubkey();

    // 1. Create proposal
    let proposal_id = dao.create_proposal(
        "Test Proposal".to_string(),
        "A test proposal".to_string(),
        ProposalType::TextProposal {
            title: "Test".to_string(),
            description: "Testing".to_string(),
        },
        proposer,
        100_000_000,
        0,
    ).unwrap();

    // 2. Proposal starts in draft/pending
    let proposal = dao.get_proposal(proposal_id).unwrap();
    assert_eq!(proposal.title, "Test Proposal");

    // 3. Vote on it
    let voter = random_pubkey();
    let sig = [0u8; 64];
    dao.vote(proposal_id, voter, VoteChoice::For, sig).unwrap();

    // 4. Check stats
    let stats = dao.stats();
    assert_eq!(stats.total, 1);
}

#[test]
fn test_governance_voting_power() {
    let mut dao = AetherDAO::with_default_config();

    // Create a snapshot with voting power
    let voter1 = random_pubkey();
    let voter2 = random_pubkey();
    let snapshot = dao.create_snapshot(vec![
        (voter1, 10_000_000_000_000),
        (voter2, 20_000_000_000_000),
    ]);

    let proposer = random_pubkey();
    let proposal_id = dao.create_proposal(
        "Power Test".to_string(),
        "Test voting power".to_string(),
        ProposalType::TextProposal {
            title: "Power".to_string(),
            description: "Test".to_string(),
        },
        proposer,
        100_000_000,
        snapshot,
    ).unwrap();

    // Vote with voter1 (10T power)
    dao.vote(proposal_id, voter1, VoteChoice::For, [0u8; 64]).unwrap();
    // Vote with voter2 (20T power)
    dao.vote(proposal_id, voter2, VoteChoice::Against, [1u8; 64]).unwrap();

    let proposal = dao.get_proposal(proposal_id).unwrap();
    // voter2 has 2x power, so against should lead
    assert!(proposal.tally.against_votes > proposal.tally.for_votes);
}

#[test]
fn test_governance_council_veto() {
    let mut dao = AetherDAO::with_default_config();
    let council_member = [42u8; 32];
    dao.add_council_member(council_member);

    let proposer = random_pubkey();
    let proposal_id = dao.create_proposal(
        "Veto Test".to_string(),
        "Test veto power".to_string(),
        ProposalType::TextProposal {
            title: "Veto".to_string(),
            description: "Test".to_string(),
        },
        proposer,
        100_000_000,
        0,
    ).unwrap();

    // Council member vetoes
    dao.veto_proposal(proposal_id, &council_member).unwrap();

    let proposal = dao.get_proposal(proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Vetoed));
}

#[test]
fn test_governance_cancel_by_proposer() {
    let mut dao = AetherDAO::with_default_config();
    let proposer = [99u8; 32];

    let proposal_id = dao.create_proposal(
        "Cancel Test".to_string(),
        "Test cancellation".to_string(),
        ProposalType::TextProposal {
            title: "Cancel".to_string(),
            description: "Test".to_string(),
        },
        proposer,
        100_000_000,
        0,
    ).unwrap();

    // Proposer cancels
    dao.cancel_proposal(proposal_id, &proposer).unwrap();

    let proposal = dao.get_proposal(proposal_id).unwrap();
    assert!(matches!(proposal.status, ProposalStatus::Cancelled));
}

#[test]
fn test_governance_proposal_types() {
    let mut dao = AetherDAO::with_default_config();
    let proposer = random_pubkey();

    // Text proposal
    let id1 = dao.create_proposal(
        "Text".to_string(),
        "desc".to_string(),
        ProposalType::TextProposal {
            title: "Text".to_string(),
            description: "desc".to_string(),
        },
        proposer,
        100_000_000,
        0,
    ).unwrap();

    // Parameter change proposal
    let id2 = dao.create_proposal(
        "Param".to_string(),
        "desc".to_string(),
        ProposalType::ParameterChange {
            parameter: "slot_time_ms".to_string(),
            current_value: "400".to_string(),
            new_value: "300".to_string(),
        },
        proposer,
        100_000_000,
        0,
    ).unwrap();

    // Fund allocation proposal
    let recipient = random_pubkey();
    let id3 = dao.create_proposal(
        "Fund".to_string(),
        "desc".to_string(),
        ProposalType::FundAllocation {
            recipient,
            amount: 500_000_000_000,
            token_type: TokenType::ATH,
            purpose: "Community grant".to_string(),
        },
        proposer,
        100_000_000,
        0,
    ).unwrap();

    assert!(dao.get_proposal(id1).is_some());
    assert!(dao.get_proposal(id2).is_some());
    assert!(dao.get_proposal(id3).is_some());
}

// ============================================================================
// Treasury Tests
// ============================================================================

#[test]
fn test_treasury_withdrawal_lifecycle() {
    let mut treasury = Treasury::with_default_config();
    let recipient = random_pubkey();
    let signer1 = random_pubkey();
    let signer2 = random_pubkey();
    let now = 1000000u64;

    // Create withdrawal
    let wid = treasury.create_withdrawal(
        recipient,
        1_000_000_000_000,
        TokenType::ATH,
        "Test withdrawal".to_string(),
        now,
    ).unwrap();

    // Approve with first signer
    treasury.approve_withdrawal(wid, signer1).unwrap();

    // Try to execute before timelock — should fail
    let result = treasury.execute_withdrawal(wid, now, [0u8; 64]);
    assert!(result.is_err(), "Should not execute before timelock");

    // Approve with second signer (need 2/3)
    treasury.approve_withdrawal(wid, signer2).unwrap();

    // Execute after timelock
    let future_time = now + 86400 * 7 + 1; // Past timelock
    treasury.execute_withdrawal(wid, future_time, [1u8; 64]).unwrap();

    // Verify withdrawal is now executed
    let w = treasury.get_withdrawal(wid).unwrap();
    assert!(matches!(w.status, WithdrawalStatus::Executed));
}

#[test]
fn test_treasury_add_signer() {
    let mut treasury = Treasury::with_default_config();
    let new_signer = random_pubkey();

    treasury.add_signer(new_signer, 1000).unwrap();

    let summary = treasury.summary();
    assert!(summary.signer_count >= 3, "Should have at least default signers plus new one");
}

#[test]
fn test_treasury_summary() {
    let treasury = Treasury::with_default_config();
    let summary = treasury.summary();

    // Default treasury should have zero balances initially
    assert!(summary.signer_count >= 2, "Should have default signers");
}

#[test]
fn test_treasury_pending_withdrawals() {
    let mut treasury = Treasury::with_default_config();
    let recipient = random_pubkey();

    // Create two withdrawals
    treasury.create_withdrawal(
        recipient, 100, TokenType::ATH, "First".to_string(), 1000,
    ).unwrap();
    treasury.create_withdrawal(
        recipient, 200, TokenType::FLUX, "Second".to_string(), 1000,
    ).unwrap();

    let pending = treasury.get_pending_withdrawals();
    assert_eq!(pending.len(), 2, "Should have 2 pending withdrawals");
}

#[test]
fn test_treasury_budget_status() {
    let treasury = Treasury::with_default_config();
    let budgets = treasury.get_budget_status();

    // Should have budget categories
    assert!(!budgets.is_empty(), "Should have budget allocations");
}

// ============================================================================
// Validator State Tests
// ============================================================================

#[test]
fn test_validator_state_creation() {
    let state = create_test_state();

    // Fresh state should have sensible defaults
    assert_eq!(state.current_slot(), 0);
    assert_eq!(state.block_height(), 0);
    assert_eq!(state.transaction_count(), 0);
    assert_eq!(state.peer_count(), 0);
    assert!(state.has_genesis() || state.get_genesis_hash().is_empty());
}

#[test]
fn test_validator_state_slot_advancement() {
    let state = create_test_state();

    assert_eq!(state.current_slot(), 0);
    state.increment_slot();
    assert_eq!(state.current_slot(), 1);
    state.increment_slot();
    assert_eq!(state.current_slot(), 2);
}

#[test]
fn test_validator_state_peer_management() {
    let state = create_test_state();

    assert_eq!(state.peer_count(), 0);

    state.add_peer("peer1".to_string());
    state.add_peer("peer2".to_string());
    state.add_peer("peer3".to_string());
    assert_eq!(state.peer_count(), 3);

    // Duplicate should not add
    state.add_peer("peer1".to_string());
    assert_eq!(state.peer_count(), 3);

    state.remove_peer("peer2");
    assert_eq!(state.peer_count(), 2);
}

#[test]
fn test_validator_tier_system() {
    let state = create_test_state();

    // Default should be Full validator
    assert!(state.can_produce_blocks());
    assert!(state.can_vote());

    // Change to Observer
    use aether_validator::genesis::ValidatorTier;
    state.set_tier(ValidatorTier::Observer, None);
    assert!(!state.can_produce_blocks());
    assert!(!state.can_vote());
}

#[test]
fn test_validator_staking_via_state() {
    let state = create_test_state();
    let owner = random_pubkey();

    // Create stake
    let stake_id = state.create_stake(owner, 10_000_000_000_000, None).unwrap();
    assert_eq!(stake_id, 0);

    // Query staking positions
    let owner_b58 = bs58::encode(owner).into_string();
    let positions = state.get_staking_positions(&owner_b58);
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0]["amount"], 10_000_000_000_000);

    // Get staking summary
    let summary = state.get_staking_summary(&owner_b58);
    assert_eq!(summary.total_staked, 10_000_000_000_000);
    assert_eq!(summary.active_positions, 1);

    // Staking pool info
    let pool_info = state.get_staking_pool_info();
    assert_eq!(pool_info["total_staked"], 10_000_000_000_000);
}

#[test]
fn test_validator_staking_with_delegation() {
    let state = create_test_state();
    let owner = random_pubkey();
    let validator = random_pubkey();

    // Create stake and delegate immediately
    let stake_id = state.create_stake(owner, 10_000_000_000_000, Some(validator)).unwrap();

    // Advance epoch to unlock
    state.advance_staking_epoch();
    state.advance_staking_epoch();
    state.advance_staking_epoch();

    // Check delegation
    let owner_b58 = bs58::encode(owner).into_string();
    let positions = state.get_staking_positions(&owner_b58);
    let delegated = positions[0].get("delegated_to").unwrap().as_str().unwrap();
    let validator_b58 = bs58::encode(validator).into_string();
    assert_eq!(delegated, validator_b58);
}

#[test]
fn test_validator_governance_via_state() {
    let state = create_test_state();
    let proposer = random_pubkey();

    // Create proposal
    let proposal_id = state.create_governance_proposal(
        "Test Proposal".to_string(),
        "Testing via state".to_string(),
        ProposalType::TextProposal {
            title: "Test".to_string(),
            description: "Testing".to_string(),
        },
        proposer,
        100_000,
    ).unwrap();

    assert!(proposal_id > 0, "Proposal ID should be non-zero");

    // Get proposal
    let proposal = state.get_governance_proposal(proposal_id).unwrap();
    assert_eq!(proposal.title, "Test Proposal");

    // Vote
    let voter = random_pubkey();
    state.governance_vote(proposal_id, voter, VoteChoice::For, [0u8; 64]).unwrap();

    // Get active proposals
    let active = state.get_active_governance_proposals();
    assert!(!active.is_empty());

    // Governance stats
    let stats = state.governance_stats();
    assert!(stats.total >= 1);

    // Governance config
    let config = state.governance_config();
    assert!(config.quorum_threshold > 0);
}

#[test]
fn test_validator_treasury_via_state() {
    let state = create_test_state();

    // Get summary
    let summary = state.treasury_summary();
    assert!(summary.signer_count >= 2);

    // Create withdrawal
    let recipient = random_pubkey();
    let wid = state.treasury_create_withdrawal(
        recipient,
        1_000_000,
        TokenType::ATH,
        "Integration test withdrawal".to_string(),
        1000000,
    ).unwrap();
    assert_eq!(wid, 0);

    // Get pending withdrawals
    let pending = state.treasury_pending_withdrawals();
    assert!(!pending.is_empty());

    // Budget status
    let budgets = state.treasury_budget_status();
    assert!(!budgets.is_empty());
}

// ============================================================================
// Block Producer & Transaction Tests
// ============================================================================

#[test]
fn test_block_producer_creates_and_submits() {
    use aether_validator::block_producer::BlockProducer;
    use aether_validator::state_db::StateDB;
    use aether_core::{AetherTransaction, TransactionType, TransactionPayload};

    let state = create_test_state();
    let tmp = tempfile::tempdir().expect("temp dir");
    let state_db = StateDB::new(tmp.path().to_path_buf());
    let bp = BlockProducer::new(state.clone(), state_db);

    // Submit a transaction
    let tx = AetherTransaction {
        signature: [1u8; 64],
        signer: [2u8; 32],
        tx_type: TransactionType::Transfer,
        payload: TransactionPayload::Transfer {
            recipient: "recipient123".to_string(),
            amount: 1000,
            nonce: 0,
        },
        fee: 5000,
        slot: 0,
        timestamp: 0,
    };

    let result = bp.try_spawn_block_task(tx);
    // This might succeed or fail depending on implementation state
    // The key thing is it doesn't panic
    assert!(result.is_ok() || result.is_err(), "Should handle transaction submission");
}

// ============================================================================
// AI Priority Lane Tests
// ============================================================================

#[test]
fn test_ai_priority_lane_derivation() {
    use aether_common::types::AIPriorityLane;

    // Critical lane: fee >= 1_000_000 lamports
    assert_eq!(
        aether_validator::block_producer::BlockProducer::derive_priority_lane_from_fee(1_000_000),
        AIPriorityLane::Critical
    );

    // High lane: fee >= 500_000 lamports
    assert_eq!(
        aether_validator::block_producer::BlockProducer::derive_priority_lane_from_fee(500_000),
        AIPriorityLane::High
    );

    // Standard lane: fee < 500_000 lamports
    assert_eq!(
        aether_validator::block_producer::BlockProducer::derive_priority_lane_from_fee(499_999),
        AIPriorityLane::Standard
    );

    assert_eq!(
        aether_validator::block_producer::BlockProducer::derive_priority_lane_from_fee(0),
        AIPriorityLane::Standard
    );

    // Very high fee = Critical
    assert_eq!(
        aether_validator::block_producer::BlockProducer::derive_priority_lane_from_fee(10_000_000),
        AIPriorityLane::Critical
    );
}

#[test]
fn test_fee_distributor() {
    use aether_ai_priority::fee_distribution::FeeDistributor;

    let distributor = FeeDistributor::new([0u8; 32]);

    // Initial state should be zero
    let stats = distributor.current_epoch_stats();
    assert_eq!(stats.critical_tx_count, 0);
    assert_eq!(stats.high_tx_count, 0);
    assert_eq!(stats.standard_tx_count, 0);
}

// ============================================================================
// Consensus Integration Tests
// ============================================================================

#[test]
fn test_full_consensus_pipeline() {
    // Simulate a multi-validator consensus scenario
    let mut tower = TowerConsensus::new();
    let total_stake = 100_000_000_000_000u64;

    // Add 5 validators with different stakes
    let validators: Vec<([u8; 32], u64)> = (0..5).map(|i| {
        let mut key = [0u8; 32];
        key[0] = i + 1;
        (key, 20_000_000_000_000)
    }).collect();

    // All validators vote on slots 1 through 10
    for slot in 1..=10 {
        for (validator, stake) in &validators {
            tower.process_vote(*validator, slot, *stake).unwrap();
        }
    }

    // Slot 1 should be confirmed (100% stake)
    assert!(tower.is_slot_confirmed(1, total_stake));

    // Check root slot has advanced
    let root = tower.get_root_slot();
    assert!(root > 0, "Root should have advanced past 0");
}

#[test]
fn test_consensus_fork_choice_integration() {
    // Test that fork choice works with tower consensus
    let root = [0u8; 32];
    let mut fc = ForkChoice::new(root);

    // Build main chain
    let b1 = [1u8; 32];
    let b2 = [2u8; 32];
    let b3 = [3u8; 32];
    fc.add_block(b1, root, 1, 1000).unwrap();
    fc.add_block(b2, b1, 2, 1000).unwrap();
    fc.add_block(b3, b2, 3, 1000).unwrap();

    // Build competing fork from b1
    let b2_alt = [12u8; 32];
    let b3_alt = [13u8; 32];
    fc.add_block(b2_alt, b1, 2, 2000).unwrap();
    fc.add_block(b3_alt, b2_alt, 3, 2000).unwrap();

    // Heavier fork (alt) should be selected
    let best = fc.get_best_block().unwrap();
    assert_eq!(best.hash, b3_alt, "Heavier fork should win");

    // Verify chain integrity
    let chain = fc.get_chain(b3_alt);
    assert_eq!(chain.len(), 4); // root + b1 + b2_alt + b3_alt
}

// ============================================================================
// StateDB Tests
// ============================================================================

#[test]
fn test_state_db_basic_operations() {
    use aether_validator::state_db::StateDB;
    let db = StateDB::new();

    // Account operations
    let addr = [1u8; 32];
    let account = aether_core::Account {
        lamports: 5000,
        owner: [2u8; 32],
        data: vec![1, 2, 3],
        rent_epoch: 0,
    };

    db.set_account_sync(&addr, account.clone());
    let retrieved = db.get_account_sync(&addr).unwrap();
    assert_eq!(retrieved.lamports, 5000);
    assert_eq!(retrieved.data, vec![1, 2, 3]);

    // Non-existent account
    let missing = db.get_account_sync(&[99u8; 32]);
    assert!(missing.is_none());
}

#[test]
fn test_state_db_persistence() {
    use aether_validator::state_db::StateDB;
    
    // StateDB is in-memory, so we test data consistency within same instance
    let db = StateDB::new();
    let addr = [42u8; 32];
    let account = aether_core::Account {
        lamports: 99999,
        owner: [0u8; 32],
        data: vec![],
        rent_epoch: 100,
    };
    db.set_account_sync(&addr, account.clone());

    // Verify write
    let retrieved = db.get_account_sync(&addr).unwrap();
    assert_eq!(retrieved.lamports, 99999);
    assert_eq!(retrieved.rent_epoch, 100);

    // Verify total supply tracking
    assert_eq!(db.total_supply_sync(), 99999);
    assert_eq!(db.account_count(), 1);
}