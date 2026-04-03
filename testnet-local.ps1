# Aether Testnet - Local 2-Node Network
# Run this from the target/release directory

# ============================================
# TERMINAL 1 - Bootstrap Node (Genesis Seed)
# ============================================
# This creates the genesis block and starts the first validator
Write-Host "=== TERMINAL 1: Starting Bootstrap Node ===" -ForegroundColor Green

# Generate genesis if not exists
if (!(Test-Path "genesis.json")) {
    .\aether-validator.exe create-genesis
    Write-Host "Genesis created. Check genesis.json and bootstrap-validator-identity.json" -ForegroundColor Yellow
}

# Start bootstrap validator
# --testnet: use testnet mode
# --genesis genesis.json: load the shared genesis
# --port 8001: P2P gossip port
.\aether-validator.exe start --testnet --genesis genesis.json --p2p-addr "0.0.0.0:8001"

# ============================================
# TERMINAL 2 - Connecting Node
# ============================================
# Connect to the bootstrap node using --bootstrap
Write-Host "=== TERMINAL 2: Starting Connecting Node ===" -ForegroundColor Cyan

# Start second validator, connect to bootstrap
# --bootstrap localhost:8001: connect to first node's P2P port
.\aether-validator.exe start --testnet --genesis genesis.json --p2p-addr "0.0.0.0:8002" --bootstrap "localhost:8001"

# ============================================
# TERMINAL 3 - Query the Network
# ============================================
Write-Host "=== TERMINAL 3: Querying Chain ===" -ForegroundColor Yellow

# Check health
Write-Host "`n--- Health Check ---" -ForegroundColor White
curl http://localhost:8899/health

# Check current slot
Write-Host "`n--- Current Slot ---" -ForegroundColor White
curl http://localhost:8899/v1/slot

# Check genesis info
Write-Host "`n--- Genesis Info ---" -ForegroundColor White
curl http://localhost:8899/v1/genesis

# Check epoch
Write-Host "`n--- Epoch Info ---" -ForegroundColor White
curl http://localhost:8899/v1/epoch

# Check validators
Write-Host "`n--- Validators ---" -ForegroundColor White
curl http://localhost:8899/v1/validators

# Check block production
Write-Host "`n--- Block Production ---" -ForegroundColor White
curl http://localhost:8899/v1/block_production

# Get a specific block
Write-Host "`n--- Block at Slot 10 ---" -ForegroundColor White
curl "http://localhost:8899/v1/block?slot=10"

Write-Host "`nDone!" -ForegroundColor Green
