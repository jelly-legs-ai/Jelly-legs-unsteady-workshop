# aether-cli

AeTHer Validator Command Line Interface for system validation, onboarding, and node management.

## Installation

```bash
# Install globally via npm
npm install -g aether-cli

# Or install from repo
git clone https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop.git
cd Jelly-legs-unsteady-workshop/aether-cli
npm install
npm link
```

## Quick Start

```bash
# Check system requirements
aether-cli doctor

# Start onboarding wizard
aether-cli init

# Generate KYC link
aether-cli kyc generate

# Manage validator
aether-cli validator start
aether-cli validator status
```

## Commands

| Command | Description |
|---------|-------------|
| `aether-cli doctor` | Run system requirements checks (CPU/RAM/Disk/Network/Firewall) |
| `aether-cli init` | Start onboarding wizard |
| `aether-cli kyc generate` | Generate pre-filled KYC link with pubkey, node ID, signature |
| `aether-cli validator start` | Start validator node |
| `aether-cli validator stop` | Stop validator node |
| `aether-cli validator status` | Check validator status |
| `aether-cli help` | Show help message |
| `aether-cli --version` | Show version number |

## System Requirements

The `doctor` command validates:

- **CPU**: 8+ cores (physical)
- **RAM**: 32GB+ total, 28GB+ available
- **Disk**: 512GB+ SSD with 340GB+ free
- **Network**: 100Mbps+ connection
- **Firewall**: Ports 3030 (P2P), 8899 (RPC), 22 (SSH) accessible

## Example Output

```
$ aether-cli doctor

███╗   ███╗██╗███████╗███████╗██╗ ██████╗ ███╗   ██╗
████╗ ████║██║██╔════╝██╔════╝██║██╔═══██╗████╗  ██║
...

Validator System Check
  v1.0.0
  2026-03-24

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Running system checks...

CPU
  Model: Intel Xeon E5-2680 v4
  Cores: 16 (8 physical)
  Frequency: 2.4 GHz
  ✅ PASS (required: 4+ cores minimum)

Memory
  Total: 32 GB DDR4
  Available: 28 GB
  ✅ PASS (required: 8 GB minimum)

...

SUMMARY: All checks passed! ✅

Your system is ready to run an AeTHer validator.
```

## Development

```bash
# Run doctor command
npm run doctor

# Run tests
npm test
```

## License

MIT - Jelly-legs AI Team

## Links

- Repository: https://github.com/jelly-legs-ai/Jelly-legs-unsteady-workshop
- Documentation: docs/MINING_VALIDATOR_TOOLS.md
