#!/bin/bash
# Aether Validator Local Testnet Scripts
# =======================================
# Terminal 1 (bootstrap node - first node)
# ----------------------------------------
# Run: ./testnet-local.sh
# Or manually:
# ./target/release/aether-validator create-genesis --out genesis.json
# ./target/release/aether-validator start --genesis genesis.json --port 8001

# Terminal 2 (connecting node - joins via bootstrap)
# -------------------------------------------------
# Run manually:
# ./target/release/aether-validator start --genesis genesis.json --port 8002 --bootstrap localhost:8001

echo "Aether Validator Local Testnet Setup"
echo "===================================="
echo ""
echo "TERMINAL 1 - Bootstrap Node:"
echo "  ./target/release/aether-validator create-genesis --out genesis.json"
echo "  ./target/release/aether-validator start --genesis genesis.json --port 8001"
echo ""
echo "TERMINAL 2 - Connecting Node:"
echo "  ./target/release/aether-validator start --genesis genesis.json --port 8002 --bootstrap localhost:8001"
echo ""
echo "Make sure genesis.json is copied to the Terminal 2 directory!"
