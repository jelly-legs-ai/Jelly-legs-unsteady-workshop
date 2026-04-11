//! Integration tests for Aether validator core components.
//!
//! Covers: block production, state DB, executor, staking via ValidatorState,
//! AI priority lanes, persistence, and security hardening.

use aether_common::types::AIPriorityLane;
use aether_core::{Account, AetherTransaction, TransactionPayload, TransactionType};
use aether_validator::state::ValidatorState;
use aether_validator::state_db::StateDB;
use aether_validator::executor::Executor;
use aether_validator::block_producer::BlockProducer;
use aether_validator::persistence::{PersistenceManager, PersistedAccount, PersistedBlock, ValidatorSnapshot};
use aether_validator::keypair;
use aether_validator::genesis;
use aether_validator::sync::SyncConfig;
use sha2::Digest;
use std::path::PathBuf;
use std::sync::Arc;

// Helper: create a ValidatorState for testing
fn create_test_state() -> ValidatorState {
    let identity = keypair::generate_keypair();
    ValidatorState::new(identity, true, PathBuf::from("test_ledger")).unwrap()
}

// Helper: create a funded StateDB with a test account
fn create_funded_state_db() -> StateDB {
    let db = StateDB::new();
    let addr = [1u8; 32];
    let account = Account {
        lamports: 1_000_000_000_000,
        owner: [0u8; 32],
        data: vec![],
        rent_epoch: 0,
    };
    db.set_account_sync(&addr, account);
    db
}

// Helper: create a simple transfer transaction
fn create_transfer_tx(signer: [u8; 32], recipient: &str, amount: u64, fee: u64, slot: u64) -> AetherTransaction {
    AetherTransaction {
        signer,
        signature: [0u8; 64],
        tx_type: TransactionType::Transfer,
        payload: TransactionPayload::Transfer {
            recipient: recipient.to_string(),
            amount,
            nonce: 0,
        },
        fee,
        slot,
        timestamp: 0,
    }
}

// ============================================================================
// StateDB Tests
// ============================================================================

#[test]
fn test_state_db_basic_operations() {
    let db = StateDB::new();
    let addr = [42u8; 32];
    
    // Account doesn't exist yet
    assert!(db.get_account_sync(&addr).is_none());
    
    // Create account
    let account = Account {
        lamports: 5000,
        owner: [0u8; 32],
        data: vec![1, 2, 3],
        rent_epoch: 1,
    };
    db.set_account_sync(&addr, account.clone());
    
    // Retrieve account
    let retrieved = db.get_account_sync(&addr).unwrap();
    assert_eq!(retrieved.lamports, 5000);
    assert_eq!(retrieved.data, vec![1, 2, 3]);
    assert_eq!(retrieved.rent_epoch, 1);
}

#[test]
fn test_state_db_credit_debit() {
    let db = create_funded_state_db();
    let addr = [1u8; 32];
    
    // Credit
    db.credit_sync(&addr, 1000).unwrap();
    let acc = db.get_account_sync(&addr).unwrap();
    assert_eq!(acc.lamports, 1_000_000_000_000 + 1000);
    
    // Debit
    db.debit_sync(&addr, 500).unwrap();
    let acc = db.get_account_sync(&addr).unwrap();
    assert_eq!(acc.lamports, 1_000_000_000_000 + 500);
}

#[test]
fn test_state_db_insufficient_funds() {
    let db = StateDB::new();
    let addr = [1u8; 32];
    
    let account = Account { lamports: 100, owner: [0u8; 32], data: vec![], rent_epoch: 0 };
    db.set_account_sync(&addr, account);
    
    let result = db.debit_sync(&addr, 200);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Insufficient lamports"));
}

#[test]
fn test_state_db_transfer() {
    let db = StateDB::new();
    let from = [1u8; 32];
    let to = [2u8; 32];
    
    let from_account = Account { lamports: 10_000, owner: [0u8; 32], data: vec![], rent_epoch: 0 };
    db.set_account_sync(&from, from_account);
    let to_account = Account { lamports: 0, owner: [0u8; 32], data: vec![], rent_epoch: 0 };
    db.set_account_sync(&to, to_account);
    
    db.transfer_sync(&from, &to, 3000).unwrap();
    
    let from_acc = db.get_account_sync(&from).unwrap();
    let to_acc = db.get_account_sync(&to).unwrap();
    assert_eq!(from_acc.lamports, 7_000);
    assert_eq!(to_acc.lamports, 3_000);
}

#[test]
fn test_state_db_transfer_insufficient_funds() {
    let db = StateDB::new();
    let from = [1u8; 32];
    let to = [2u8; 32];
    
    let from_account = Account { lamports: 100, owner: [0u8; 32], data: vec![], rent_epoch: 0 };
    db.set_account_sync(&from, from_account);
    let to_account = Account { lamports: 0, owner: [0u8; 32], data: vec![], rent_epoch: 0 };
    db.set_account_sync(&to, to_account);
    
    let result = db.transfer_sync(&from, &to, 200);
    assert!(result.is_err());
}

#[test]
fn test_state_db_total_supply() {
    let db = StateDB::new();
    assert_eq!(db.total_supply_sync(), 0);
    
    let addr1 = [1u8; 32];
    let addr2 = [2u8; 32];
    db.set_account_sync(&addr1, Account { lamports: 5000, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    db.set_account_sync(&addr2, Account { lamports: 3000, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    assert_eq!(db.total_supply_sync(), 8000);
}

#[test]
fn test_state_db_state_root_deterministic() {
    let db = StateDB::new();
    let addr = [1u8; 32];
    db.set_account_sync(&addr, Account { lamports: 100, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    let root1 = db.compute_state_root();
    let root2 = db.compute_state_root();
    assert_eq!(root1, root2, "State root must be deterministic");
}

#[test]
fn test_state_db_state_root_changes_on_mutation() {
    let db = StateDB::new();
    let addr = [1u8; 32];
    db.set_account_sync(&addr, Account { lamports: 100, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    let root1 = db.compute_state_root();
    db.credit_sync(&addr, 50).unwrap();
    let root2 = db.compute_state_root();
    
    assert_ne!(root1, root2, "State root must change after mutation");
}

#[test]
fn test_state_db_get_all_accounts() {
    let db = StateDB::new();
    assert!(db.get_all_accounts_sync().is_empty());
    
    for i in 1..=3u8 {
        let mut addr = [0u8; 32];
        addr[0] = i;
        db.set_account_sync(&addr, Account { lamports: i as u64 * 100, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    }
    
    let accounts = db.get_all_accounts_sync();
    assert_eq!(accounts.len(), 3);
}

// ============================================================================
// Executor Tests
// ============================================================================

#[test]
fn test_executor_transfer() {
    let db = create_funded_state_db();
    let executor = Executor::new(db.clone());
    
    let from = [1u8; 32];
    let to = [2u8; 32];
    
    // Create recipient account
    db.set_account_sync(&to, Account { lamports: 0, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    let tx = create_transfer_tx(from, &bs58::encode(to).into_string(), 500, 100, 1);
    let result = executor.execute(&tx);
    
    assert!(result.success, "Transfer should succeed: {:?}", result.error);
}

#[test]
fn test_executor_transfer_insufficient_funds() {
    let db = StateDB::new();
    let from = [99u8; 32]; // Account doesn't exist
    
    let executor = Executor::new(db);
    let tx = create_transfer_tx(from, "nonexistent_recipient", 500, 100, 1);
    
    let result = executor.execute(&tx);
    assert!(!result.success);
}

#[test]
fn test_executor_stake() {
    let db = create_funded_state_db();
    let from = [1u8; 32];
    let validator = [2u8; 32];
    
    let executor = Executor::new(db);
    
    let tx = AetherTransaction {
        signer: from,
        signature: [0u8; 64],
        tx_type: TransactionType::Stake,
        payload: TransactionPayload::Stake {
            validator: bs58::encode(validator).into_string(),
            amount: 1000,
        },
        fee: 100,
        slot: 1,
        timestamp: 0,
    };
    
    let result = executor.execute(&tx);
    assert!(result.success, "Stake should succeed: {:?}", result.error);
}

#[test]
fn test_executor_stake_insufficient_funds() {
    let db = StateDB::new();
    let from = [1u8; 32];
    db.set_account_sync(&from, Account { lamports: 0, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    let executor = Executor::new(db);
    let tx = AetherTransaction {
        signer: from,
        signature: [0u8; 64],
        tx_type: TransactionType::Stake,
        payload: TransactionPayload::Stake {
            validator: bs58::encode([2u8; 32]).into_string(),
            amount: 1000,
        },
        fee: 100,
        slot: 1,
        timestamp: 0,
    };
    
    let result = executor.execute(&tx);
    assert!(!result.success);
}

#[test]
fn test_executor_signature_verification_fails() {
    let db = create_funded_state_db();
    let executor = Executor::new(db);
    
    // All-zeros signature is not valid Ed25519
    let from = [1u8; 32];
    let tx = create_transfer_tx(from, &bs58::encode([2u8; 32]).into_string(), 100, 10, 1);
    
    let result = executor.execute(&tx);
    assert!(!result.success, "Transaction with invalid signature should fail");
}

// ============================================================================
// BlockProducer Tests
// ============================================================================

#[tokio::test]
async fn test_block_producer_submit_transaction() {
    let state = create_test_state();
    let state_db = StateDB::new();
    let bp = Arc::new(BlockProducer::new(state, state_db));
    
    let from = [1u8; 32];
    bp.set_account_sync(&from, Account { lamports: 1_000_000, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    let tx = create_transfer_tx(from, &bs58::encode([2u8; 32]).into_string(), 1000, 500_000, 1);
    
    let sig = bp.submit_transaction(tx).await.unwrap();
    assert!(!sig.is_empty(), "Transaction signature should not be empty");
}

#[tokio::test]
async fn test_block_producer_pool_stats_empty() {
    let state = create_test_state();
    let state_db = StateDB::new();
    let bp = Arc::new(BlockProducer::new(state, state_db));
    
    let (critical, high, standard, fees) = bp.pool_stats().await;
    assert_eq!(critical, 0);
    assert_eq!(high, 0);
    assert_eq!(standard, 0);
    assert_eq!(fees, 0);
}

#[tokio::test]
async fn test_block_producer_get_block_none() {
    let state = create_test_state();
    let state_db = StateDB::new();
    let bp = Arc::new(BlockProducer::new(state, state_db));
    
    let block: Option<aether_validator::block_producer::Block> = bp.get_block(999).await;
    assert!(block.is_none());
}

// ============================================================================
// ValidatorState Tests
// ============================================================================

#[test]
fn test_validator_state_slot_tracking() {
    let state = create_test_state();
    assert_eq!(state.current_slot(), 0);
    
    state.set_current_slot(42);
    assert_eq!(state.current_slot(), 42);
}

#[test]
fn test_validator_state_epoch_info() {
    let state = create_test_state();
    let epoch_info = state.epoch_info();
    assert_eq!(epoch_info.epoch, 0);
    assert_eq!(epoch_info.slot_index, 0);
}

#[test]
fn test_validator_state_tier_management() {
    let state = create_test_state();
    assert!(state.is_full());
    assert!(!state.is_lite());
    assert!(!state.is_observer());
    
    state.set_tier(aether_validator::genesis::ValidatorTier::Observer, None);
    assert!(state.is_observer());
    assert!(!state.is_full());
}

#[test]
fn test_validator_state_staking() {
    let state = create_test_state();
    let owner = [1u8; 32];
    
    let stake_id = state.create_stake(owner, 100_000_000_000, None).unwrap();
    assert_eq!(stake_id, 0);
    
    let positions = state.get_staking_positions(&bs58::encode(owner).into_string());
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0]["amount"], 100_000_000_000);
}

#[test]
fn test_validator_state_staking_pool_info() {
    let state = create_test_state();
    let info = state.get_staking_pool_info();
    assert_eq!(info["current_epoch"], 0);
    assert_eq!(info["total_staked"], 0);
}

#[test]
fn test_validator_state_peer_management() {
    let state = create_test_state();
    assert_eq!(state.peer_count(), 0);
    
    state.add_peer("peer-1".to_string());
    assert_eq!(state.peer_count(), 1);
    
    state.add_peer("peer-2".to_string());
    assert_eq!(state.peer_count(), 2);
    
    // Duplicate shouldn't increase count
    state.add_peer("peer-1".to_string());
    assert_eq!(state.peer_count(), 2);
    
    state.remove_peer("peer-1");
    assert_eq!(state.peer_count(), 1);
}

// ============================================================================
// Persistence Tests
// ============================================================================

#[test]
fn test_persistence_snapshot_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let pm = PersistenceManager::new(dir.path()).unwrap();
    
    let snapshot = ValidatorSnapshot {
        current_slot: 12345,
        block_hash: "test-hash-abc".to_string(),
        blocks_produced: 100,
        transaction_count: 5000,
        genesis_hash: "genesis-hash".to_string(),
        chain_id: "test-chain".to_string(),
        peers: vec!["peer-1".to_string(), "peer-2".to_string()],
        timestamp: 1700000000,
    };
    
    pm.save_snapshot(&snapshot).unwrap();
    let loaded = pm.load_snapshot().unwrap().unwrap();
    
    assert_eq!(loaded.current_slot, 12345);
    assert_eq!(loaded.block_hash, "test-hash-abc");
    assert_eq!(loaded.blocks_produced, 100);
    assert_eq!(loaded.transaction_count, 5000);
}

#[test]
fn test_persistence_accounts_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let pm = PersistenceManager::new(dir.path()).unwrap();
    
    let accounts = vec![
        PersistedAccount {
            address: [1u8; 32],
            lamports: 5000,
            owner: [0u8; 32],
            data: vec![1, 2, 3],
            rent_epoch: 1,
        },
        PersistedAccount {
            address: [2u8; 32],
            lamports: 10_000,
            owner: [0u8; 32],
            data: vec![],
            rent_epoch: 0,
        },
    ];
    
    pm.save_accounts(&accounts).unwrap();
    let loaded = pm.load_accounts().unwrap();
    
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].lamports, 5000);
    assert_eq!(loaded[1].lamports, 10_000);
}

#[test]
fn test_persistence_block_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let pm = PersistenceManager::new(dir.path()).unwrap();
    
    let block = PersistedBlock {
        slot: 100,
        timestamp: 1700000000,
        previous_block_hash: "prev-hash".to_string(),
        block_hash: "block-hash-100".to_string(),
        transactions: vec!["tx1".to_string(), "tx2".to_string()],
        poh_seed: "poh-seed-100".to_string(),
        state_root: "state-root-100".to_string(),
    };
    
    pm.save_block(&block).unwrap();
    let loaded = pm.load_block(100).unwrap().unwrap();
    
    assert_eq!(loaded.slot, 100);
    assert_eq!(loaded.block_hash, "block-hash-100");
    assert_eq!(loaded.transactions.len(), 2);
}

#[test]
fn test_persistence_no_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let pm = PersistenceManager::new(dir.path()).unwrap();
    let result = pm.load_snapshot().unwrap();
    assert!(result.is_none(), "Should return None when no snapshot exists");
}

// ============================================================================
// AI Priority Lane Tests
// ============================================================================

#[test]
fn test_ai_priority_lane_derivation() {
    // Critical: >= 1_000_000 lamports
    assert_eq!(BlockProducer::derive_priority_lane_test(1_000_000), AIPriorityLane::Critical);
    assert_eq!(BlockProducer::derive_priority_lane_test(5_000_000), AIPriorityLane::Critical);
    
    // High: >= 500_000 but < 1_000_000
    assert_eq!(BlockProducer::derive_priority_lane_test(500_000), AIPriorityLane::High);
    assert_eq!(BlockProducer::derive_priority_lane_test(999_999), AIPriorityLane::High);
    
    // Standard: < 500_000
    assert_eq!(BlockProducer::derive_priority_lane_test(100), AIPriorityLane::Standard);
    assert_eq!(BlockProducer::derive_priority_lane_test(499_999), AIPriorityLane::Standard);
}

// ============================================================================
// Keypair Tests
// ============================================================================

#[test]
fn test_keypair_generation() {
    let kp1 = keypair::generate_keypair();
    let kp2 = keypair::generate_keypair();
    assert_ne!(kp1.pubkey(), kp2.pubkey());
    
    let pubkey = kp1.pubkey();
    assert!(!pubkey.is_empty());
    assert!(bs58::decode(&pubkey).into_vec().is_ok());
}

#[test]
fn test_keypair_save_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test-keypair.json");
    
    let kp = keypair::generate_keypair();
    let original_pubkey = kp.pubkey();
    
    keypair::save_identity(&path, &kp).unwrap();
    assert!(path.exists());
    
    let loaded = keypair::load_identity(&path).unwrap();
    assert_eq!(loaded.pubkey(), original_pubkey);
}

// ============================================================================
// Genesis Tests
// ============================================================================

#[test]
fn test_genesis_creation() {
    let chain_id = "aether-testnet-1".to_string();
    let validators = vec![aether_validator::genesis::GenesisValidator {
        identity_pubkey: "test-validator".to_string(),
        stake: 10_000_000,
        commission: 10,
        active: true,
    }];
    
    let genesis = genesis::create_genesis_with(&chain_id, validators);
    
    assert_eq!(genesis.chain_id, "aether-testnet-1");
    assert!(!genesis.genesis_hash.is_empty());
    assert_eq!(genesis.bootstrap_validators.len(), 1);
    assert_eq!(genesis.bootstrap_validators[0].stake, 10_000_000);
}

// ============================================================================
// Security / Edge Case Tests
// ============================================================================

#[test]
fn test_state_db_debit_to_zero() {
    let db = StateDB::new();
    let addr = [1u8; 32];
    
    db.set_account_sync(&addr, Account { lamports: 1000, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    db.debit_sync(&addr, 1000).unwrap();
    let acc = db.get_account_sync(&addr).unwrap();
    assert_eq!(acc.lamports, 0);
    
    let result = db.debit_sync(&addr, 1);
    assert!(result.is_err());
}

#[test]
fn test_state_db_concurrent_access() {
    use std::thread;
    
    let db = Arc::new(StateDB::new());
    let addr = [1u8; 32];
    
    db.set_account_sync(&addr, Account { lamports: 1_000_000, owner: [0u8; 32], data: vec![], rent_epoch: 0 });
    
    let mut handles = vec![];
    for i in 0..10 {
        let db_clone = db.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                if i % 2 == 0 {
                    let _ = db_clone.credit_sync(&addr, 100);
                } else {
                    let _ = db_clone.debit_sync(&addr, 50);
                }
            }
        }));
    }
    
    for h in handles {
        h.join().unwrap();
    }
    
    let acc = db.get_account_sync(&addr);
    assert!(acc.is_some(), "Account should still exist after concurrent operations");
}

#[test]
fn test_executor_nonexistent_recipient_transfer_fails() {
    let db = create_funded_state_db();
    let from = [1u8; 32];
    let to = [99u8; 32];
    
    assert!(db.get_account_sync(&to).is_none());
    
    let executor = Executor::new(db);
    let tx = create_transfer_tx(from, &bs58::encode(to).into_string(), 100, 10, 1);
    let result = executor.execute(&tx);
    assert!(!result.success, "Transfer to nonexistent account should fail");
}

// ============================================================================
// Sync Manager Tests
// ============================================================================

#[tokio::test]
async fn test_sync_manager_following_state() {
    let state = create_test_state();
    let state_db = StateDB::new();
    let bp = Arc::new(BlockProducer::new(state.clone(), state_db));
    
    let sync_manager = aether_validator::sync::SyncManager::new(SyncConfig::default(), state, bp, None);
    
    assert!(!sync_manager.is_syncing().await);
    let sync_state = sync_manager.get_state().await;
    assert!(matches!(sync_state, aether_validator::sync::SyncState::Following));
}

#[tokio::test]
async fn test_sync_manager_start_sync() {
    let state = create_test_state();
    let state_db = StateDB::new();
    let bp = Arc::new(BlockProducer::new(state.clone(), state_db));
    
    let sync_manager = aether_validator::sync::SyncManager::new(SyncConfig::default(), state, bp, None);
    
    sync_manager.start_sync("peer-1", 100).await;
    assert!(sync_manager.is_syncing().await);
}

// ============================================================================
// Network GossipMessage Tests
// ============================================================================

#[test]
fn test_gossip_message_serialization_all_types() {
    use aether_validator::network::GossipMessage;
    
    // SlotUpdate
    let msg = GossipMessage::SlotUpdate {
        slot: 12345,
        peer_id: "peer-1".to_string(),
        block_hash: "hash-abc".to_string(),
    };
    let bytes = msg.to_bytes().unwrap();
    let decoded = GossipMessage::from_bytes(&bytes).unwrap();
    match decoded {
        GossipMessage::SlotUpdate { slot, .. } => assert_eq!(slot, 12345),
        _ => panic!("Wrong message type"),
    }
    
    // BlockAnnounce
    let msg = GossipMessage::BlockAnnounce {
        slot: 100,
        block_hash: "block-100".to_string(),
        prev_hash: "prev-99".to_string(),
        poh_seed: "seed-100".to_string(),
        state_root: "root-100".to_string(),
        tx_count: 42,
        peer_id: "validator-1".to_string(),
        block_data: Some("{\"slot\":100}".to_string()),
    };
    let bytes = msg.to_bytes().unwrap();
    let decoded = GossipMessage::from_bytes(&bytes).unwrap();
    match decoded {
        GossipMessage::BlockAnnounce { slot, tx_count, .. } => {
            assert_eq!(slot, 100);
            assert_eq!(tx_count, 42);
        }
        _ => panic!("Wrong message type"),
    }
    
    // Vote
    let msg = GossipMessage::Vote {
        slot: 50,
        block_hash: "block-50".to_string(),
        validator: "validator-1".to_string(),
        signature: "sig-123".to_string(),
    };
    let bytes = msg.to_bytes().unwrap();
    let decoded = GossipMessage::from_bytes(&bytes).unwrap();
    match decoded {
        GossipMessage::Vote { slot, .. } => assert_eq!(slot, 50),
        _ => panic!("Wrong message type"),
    }
    
    // Handshake
    let msg = GossipMessage::Handshake {
        protocol_version: "1.0.0".to_string(),
        genesis_hash: "genesis-hash".to_string(),
        chain_id: "aether-testnet-1".to_string(),
        peer_id: "peer-123".to_string(),
        current_slot: 500,
    };
    let bytes = msg.to_bytes().unwrap();
    let decoded = GossipMessage::from_bytes(&bytes).unwrap();
    match decoded {
        GossipMessage::Handshake { protocol_version, chain_id, .. } => {
            assert_eq!(protocol_version, "1.0.0");
            assert_eq!(chain_id, "aether-testnet-1");
        }
        _ => panic!("Wrong message type"),
    }
    
    // Ping/Pong
    let msg = GossipMessage::Ping { nonce: 42 };
    let bytes = msg.to_bytes().unwrap();
    let decoded = GossipMessage::from_bytes(&bytes).unwrap();
    match decoded {
        GossipMessage::Ping { nonce } => assert_eq!(nonce, 42),
        _ => panic!("Wrong message type"),
    }
    
    let msg = GossipMessage::Pong { nonce: 99 };
    let bytes = msg.to_bytes().unwrap();
    let decoded = GossipMessage::from_bytes(&bytes).unwrap();
    match decoded {
        GossipMessage::Pong { nonce } => assert_eq!(nonce, 99),
        _ => panic!("Wrong message type"),
    }
}