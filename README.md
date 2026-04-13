# @jellylegsai/aether-cli

Aether Blockchain Command Line Interface - Validator Onboarding, Staking, and Node Management.

## Installation

```bash
# Install globally via npm
npm install -g @jellylegsai/aether-cli

# Or use npx (no install)
npx @jellylegsai/aether-cli <command>
```

## Quick Start

```bash
# Check system requirements
aether doctor

# Start onboarding wizard
aether init

# Create a wallet
aether wallet create

# Check network status
aether network

# Query blockchain data
aether slot
aether balance <address>
aether epoch
```

## Commands

### Wallet & Accounts
| Command | Description |
|---------|-------------|
| `aether wallet` | Wallet management (create, import, list, balance) |
| `aether balance [address]` | Query account balance |
| `aether transfer` | Send AETH to another address |
| `aether account --address <addr>` | Query on-chain account data |
| `aether stats` | Wallet stats dashboard |
| `aether tx <signature>` | Look up transaction by signature |
| `aether tx-history [address]` | Transaction history |

### Staking
| Command | Description |
|---------|-------------|
| `aether stake` | Stake AETH to a validator |
| `aether unstake` | Unstake AETH from a validator |
| `aether claim` | Claim staking rewards |
| `aether stake-positions` | View current stake positions |
| `aether delegations` | List stake delegations |
| `aether rewards` | View and claim staking rewards |

### Validator Management
| Command | Description |
|---------|-------------|
| `aether doctor` | System requirements check |
| `aether init` | Validator onboarding wizard |
| `aether validator` | Validator lifecycle management |
| `aether validator start` | Start validator node |
| `aether validator stop` | Stop validator node |
| `aether validator status` | Check validator status |
| `aether validator register` | Register as validator |
| `aether validators` | List network validators |
| `aether monitor` | Real-time validator dashboard |
| `aether logs` | Tail validator logs |

### Network & Blockchain
| Command | Description |
|---------|-------------|
| `aether network` | Network status and health |
| `aether slot` | Current blockchain slot |
| `aether blockheight` | Current block height |
| `aether epoch` | Current epoch info |
| `aether supply` | Token supply metrics |
| `aether tps` | Transactions per second |
| `aether fees` | Network fee estimates |
| `aether version` | Node version info |

### Advanced
| Command | Description |
|---------|-------------|
| `aether sdk` | SDK tools and direct RPC access |
| `aether sdk-test` | Test SDK with real RPC calls |
| `aether deploy` | Deploy smart contracts |
| `aether call` | Call smart contract functions |
| `aether nft` | NFT management |
| `aether multisig` | Multi-signature wallet |
| `aether emergency` | Emergency response tools |

## Features

- **Real Blockchain RPC**: All commands make actual HTTP calls to Aether nodes
- **Retry Logic**: Exponential backoff with circuit breaker pattern
- **Rate Limiting**: Token bucket algorithm for API protection
- **BIP39 Wallets**: Mnemonic-based wallet generation
- **ASCII Art Branding**: Cosmic-themed CLI interface
- **Interactive Mode**: Menu-driven interface when run without args

## Environment Variables

```bash
AETHER_RPC=http://127.0.0.1:8899  # RPC endpoint
```

## SDK Integration

All commands use `@jellylegsai/aether-sdk` for blockchain operations:

```javascript
const { AetherClient } = require('@jellylegsai/aether-sdk');
const client = new AetherClient({ rpcUrl: 'http://127.0.0.1:8899' });

// Real RPC calls
const slot = await client.getSlot();
const balance = await client.getBalance(address);
const epoch = await client.getEpochInfo();
```

## System Requirements

For validator nodes:
- **CPU**: 8+ cores
- **RAM**: 32GB+ total
- **Disk**: 512GB+ SSD
- **Network**: 100Mbps+ connection
- **Ports**: 3030 (P2P), 8899 (RPC), 22 (SSH)

## Example Output

```
╔══════════════════════════════════════════════════════════════════════════╗
║                                                                          ║
║  ╔═╗AETHER╔═╗  BLOCKCHAIN CLI                            v2.1.0          ║
║  ╠═╣NETWORK╠═╣  ◆ Decentralized Infrastructure for the Future ◆          ║
║  ╚═╝        ╚═╝                                                          ║
║                                                                          ║
╚══════════════════════════════════════════════════════════════════════════╝

◆ AETHER CLI v2.1.0 ─ Decentralized Infrastructure ◆

Wallet & Accounts
  slot                Query current slot
  balance [address]   Query account balance
  transfer           Send AETH to another address
  ...

Run "aether help" for full command list.
```

## Development

```bash
# Clone repository
git clone https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop.git
cd Jelly-legs-unsteady-workshop/aether-cli

# Install dependencies
npm install

# Test commands
node index.js doctor
node index.js network
node index.js sdk getSlot
```

## License

MIT - Jelly-legs AI Team

## Links

- Repository: https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop
- Package: https://www.npmjs.com/package/@jellylegsai/aether-cli
