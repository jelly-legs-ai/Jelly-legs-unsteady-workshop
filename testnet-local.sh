#!/bin/bash
# Aether Testnet - Local 2-Node Setup
# Terminal 1: Start seed node (genesis block producer)
# Terminal 2: Start second node (connects to seed)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================="
echo "  Aether Local Testnet - 2 Node Setup"
echo "=========================================="
echo ""

# Check if genesis exists, create if not
if [ ! -f "genesis.json" ]; then
    echo "📝 Genesis file not found, creating..."
    ./target/release/aether-validator create-genesis
else
    echo "✅ Using existing genesis.json"
fi

# Create identities if they don't exist
if [ ! -f "node1.json" ]; then
    echo "📝 Creating node1 identity..."
    ./target/release/aether-validator create-validator-identity --out node1.json --force
else
    echo "✅ Using existing node1.json"
fi

if [ ! -f "node2.json" ]; then
    echo "📝 Creating node2 identity..."
    ./target/release/aether-validator create-validator-identity --out node2.json --force
else
    echo "✅ Using existing node2.json"
fi

echo ""
echo "=========================================="
echo "  Starting Node 1 (Seed/Genesis Node)"
echo "=========================================="
echo "  Port: 8001"
echo "  RPC:  localhost:8899"
echo "  Genesis: genesis.json"
echo ""
echo "Run this command:"
echo "  ./target/release/aether-validator start \\"
echo "    --genesis genesis.json \\"
echo "    --port 8001 \\"
echo "    --identity node1.json"
echo ""

# For automated testing, start node 1 in background
if [ "${AUTO_START:-}" = "true" ]; then
    ./target/release/aether-validator start \
        --genesis genesis.json \
        --port 8001 \
        --identity node1.json &

    NODE1_PID=$!
    echo "Node 1 started with PID: $NODE1_PID"

    # Wait for node to start
    sleep 3

    echo ""
    echo "=========================================="
    echo "  Starting Node 2 (Bootstrap Client)"
    echo "=========================================="
    echo "  Port: 8002"
    echo "  Bootstrap: localhost:8001"
    echo ""

    ./target/release/aether-validator start \
        --genesis genesis.json \
        --port 8002 \
        --bootstrap localhost:8001 \
        --identity node2.json &

    NODE2_PID=$!
    echo "Node 2 started with PID: $NODE2_PID"

    echo ""
    echo "=========================================="
    echo "  Testnet Running"
    echo "=========================================="
    echo "  Node 1 PID: $NODE1_PID"
    echo "  Node 2 PID: $NODE2_PID"
    echo ""
    echo "Press Ctrl+C to stop both nodes"

    # Wait for interrupt
    trap "kill $NODE1_PID $NODE2_PID 2>/dev/null; exit" INT TERM
    wait
fi