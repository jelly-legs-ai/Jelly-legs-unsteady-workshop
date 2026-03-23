-- AeTHer Chain Database Schema
-- PostgreSQL schema for on-chain data indexing and off-chain storage

-- Users and wallets
CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address VARCHAR(64) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_active TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    kyc_status VARCHAR(20) DEFAULT 'none',
    risk_score INTEGER DEFAULT 0
);

-- FLUX token mining records
CREATE TABLE mining_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id VARCHAR(128) NOT NULL,
    wallet_address VARCHAR(64) NOT NULL,
    epoch BIGINT NOT NULL,
    hashrate BIGINT NOT NULL,
    uptime_hours INTEGER NOT NULL,
    flux_mined NUMERIC(20, 8) NOT NULL,
    device_tier VARCHAR(20) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_address) REFERENCES wallets(address)
);

CREATE INDEX idx_mining_records_wallet ON mining_records(wallet_address);
CREATE INDEX idx_mining_records_epoch ON mining_records(epoch);

-- AETH staking records
CREATE TABLE staking_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(64) NOT NULL,
    token_type VARCHAR(10) NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    start_epoch BIGINT NOT NULL,
    lock_end_epoch BIGINT,
    is_active BOOLEAN DEFAULT TRUE,
    rewards_claimed NUMERIC(20, 8) DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_address) REFERENCES wallets(address)
);

CREATE INDEX idx_staking_active ON staking_records(wallet_address) WHERE is_active = TRUE;

-- Agent KYC registry
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id VARCHAR(128) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    owner_wallet VARCHAR(64) NOT NULL,
    kyc_status VARCHAR(20) DEFAULT 'pending',
    kyc_issuer VARCHAR(255),
    reputation_score DECIMAL(3, 2) DEFAULT 0.00,
    stake_bonded NUMERIC(20, 8) DEFAULT 0,
    verified_claims JSONB DEFAULT '[]',
    services JSONB DEFAULT '[]',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_verified TIMESTAMP,
    FOREIGN KEY (owner_wallet) REFERENCES wallets(address)
);

CREATE INDEX idx_agents_kyc ON agents(kyc_status);
CREATE INDEX idx_agents_owner ON agents(owner_wallet);

-- Validator nodes
CREATE TABLE validators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(64) UNIQUE NOT NULL,
    node_id VARCHAR(128) NOT NULL,
    stake_amount NUMERIC(20, 8) NOT NULL,
    commission_rate DECIMAL(5, 4) DEFAULT 0.0500,
    uptime_percentage DECIMAL(5, 2) DEFAULT 0.00,
    total_rewards NUMERIC(20, 8) DEFAULT 0,
    is_active BOOLEAN DEFAULT FALSE,
    joined_epoch BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_address) REFERENCES wallets(address)
);

CREATE INDEX idx_validators_active ON validators(is_active) WHERE is_active = TRUE;

-- Cross-chain bridge transactions
CREATE TABLE bridge_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_hash VARCHAR(128) UNIQUE NOT NULL,
    source_chain VARCHAR(32) NOT NULL,
    dest_chain VARCHAR(32) NOT NULL,
    sender VARCHAR(64) NOT NULL,
    recipient VARCHAR(64) NOT NULL,
    token_symbol VARCHAR(20) NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

CREATE INDEX idx_bridge_status ON bridge_transactions(status);
CREATE INDEX idx_bridge_sender ON bridge_transactions(sender);

-- Governance proposals
CREATE TABLE proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id BIGINT UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    proposer VARCHAR(64) NOT NULL,
    vote_start_epoch BIGINT NOT NULL,
    vote_end_epoch BIGINT NOT NULL,
    for_votes NUMERIC(20, 8) DEFAULT 0,
    against_votes NUMERIC(20, 8) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposer) REFERENCES wallets(address)
);

CREATE INDEX idx_proposals_status ON proposals(status);

-- Vote records
CREATE TABLE votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id BIGINT NOT NULL,
    voter VARCHAR(64) NOT NULL,
    vote_type VARCHAR(10) NOT NULL,
    voting_power NUMERIC(20, 8) NOT NULL,
    voted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (proposal_id) REFERENCES proposals(proposal_id),
    FOREIGN KEY (voter) REFERENCES wallets(address)
);

CREATE UNIQUE INDEX idx_votes_unique ON votes(proposal_id, voter);

-- FLUX token transfers
CREATE TABLE flux_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_hash VARCHAR(128) UNIQUE NOT NULL,
    from_address VARCHAR(64) NOT NULL,
    to_address VARCHAR(64) NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    transfer_type VARCHAR(20) DEFAULT 'transfer',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_flux_from ON flux_transfers(from_address);
CREATE INDEX idx_flux_to ON flux_transfers(to_address);
