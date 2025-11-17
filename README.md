# Cloak Parachain

[![Substrate version](https://img.shields.io/badge/Substrate-polkadot--stable2412-brightgreen?logo=Parity%20Substrate)](https://substrate.io/)
[![License](https://img.shields.io/badge/License-Unlicense-blue.svg)](https://unlicense.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

> **Privacy-Preserving Cross-Chain Bridge for Polkadot**
>
> Your Transactions, Your Privacy.

Cloak is a specialized parachain that enables completely private cross-chain asset transfers on Polkadot using zero-knowledge cryptography. Built with Substrate and powered by zkSNARKs, Cloak provides unprecedented privacy for cross-chain transactions while maintaining full decentralization and security.

---

## 🎯 Overview

### The Problem

Current blockchain systems expose all transaction details publicly:
- Your complete wallet balance
- Every person you transact with
- Exact amounts sent and received
- Your entire transaction history

This lack of privacy compromises users, businesses, and organizations operating on blockchain networks.

### The Solution

**Cloak** solves this by providing a privacy layer for the Polkadot ecosystem that enables:

✅ **Anonymous Transfers** - Complete privacy for senders and receivers
✅ **Hidden Amounts** - Transaction values kept confidential
✅ **Unlinkable Addresses** - No connection between deposits and withdrawals
✅ **Cross-Chain Privacy** - Private transfers across any parachains via XCM

---

## ✨ Features

### 🔐 Zero-Knowledge Cryptography
- **zkSNARKs (Groth16)**: Prove ownership without revealing identity
- **BN254 Elliptic Curve**: Efficient cryptographic operations
- **Pedersen Commitments**: Hide transaction amounts cryptographically

### 🌳 Anonymity Sets
- **Merkle Trees**: Create large anonymity pools for deposits
- **Scalable Privacy**: Privacy strength grows with pool size
- **Efficient Verification**: Logarithmic proof size

### 🌉 Cross-Chain Integration
- **XCM v5 Support**: Native Polkadot cross-chain messaging
- **Multi-Parachain**: Transfer privately between any parachains
- **Seamless UX**: No wrapped tokens or complex bridging

### 🛡️ Security & Decentralization
- **Off-Chain Proof Generation**: Users generate proofs locally
- **On-Chain Verification**: Validators verify proofs trustlessly
- **No Trusted Setup**: Transparent, auditable cryptography

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Cloak Parachain                        │
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
│  │  Commitment  │    │   Merkle     │    │   Nullifier  │ │
│  │  Generation  │───▶│    Tree      │───▶│   Registry   │ │
│  └──────────────┘    └──────────────┘    └──────────────┘ │
│                                                             │
│  ┌──────────────┐    ┌──────────────┐                     │
│  │   zkSNARK    │    │     XCM      │                     │
│  │ Verification │    │  Integration │                     │
│  └──────────────┘    └──────────────┘                     │
└─────────────────────────────────────────────────────────────┘
         ▲                                        │
         │                                        │
    Proof from                                 Assets to
    User's Device                            Other Parachains
```

### Privacy Flow

1. **Deposit Phase**: User deposits assets → Creates cryptographic commitment → Funds added to anonymity pool
2. **Mixing Phase**: Deposit combines with others → Merkle tree creates anonymity set → No individual tracking
3. **Withdrawal Phase**: User generates zero-knowledge proof off-chain → Proves ownership without revealing deposit → Withdraw to new address

---

## 🚀 Getting Started

### Prerequisites

**System Requirements:**
- Linux or macOS (Windows via WSL2)
- 8GB+ RAM recommended
- 50GB+ free disk space

**Software Dependencies:**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update

# Add WASM target
rustup target add wasm32-unknown-unknown

# Add rust-src component (required for no_std compilation)
rustup component add rust-src

# Install system dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install -y build-essential git clang curl libssl-dev llvm libudev-dev protobuf-compiler
```

### Installation

Clone the repository:

```bash
git clone https://github.com/Cross-chain-Cloak/Cloak.git
cd Cloak
```

---

## 🔨 Building

### Build the Runtime

```bash
# Build in release mode (recommended)
cargo build --release

# Or build specific package
cargo build --release -p parachain-template-runtime
```

The WASM runtime will be generated at:
```
target/release/wbuild/parachain-template-runtime/parachain_template_runtime.compact.compressed.wasm
```

### Build for Development

```bash
# Debug build (faster compilation, larger binary)
cargo build
```

---

## 🧪 Testing

### Run All Tests

```bash
# Run all tests in the workspace
cargo test

# Run tests in release mode (faster execution)
cargo test --release
```

### Test Privacy Bridge Pallet

The Privacy Bridge pallet includes **45 comprehensive tests** covering all functionality:

```bash
# Run all privacy bridge tests
cargo test -p pallet-privacy-bridge

# Run tests with detailed output
cargo test -p pallet-privacy-bridge -- --nocapture

# Run specific test
cargo test -p pallet-privacy-bridge test_commitment_generation

# Run tests in release mode
cargo test --release -p pallet-privacy-bridge
```

**Test Coverage:**

✅ **Commitment Generation** (5 tests)
✅ **Nullifier System** (5 tests)
✅ **Merkle Tree Operations** (8 tests)
✅ **zkSNARK Proofs** (12 tests)
✅ **XCM Integration** (10 tests)
✅ **Edge Cases & Security** (5 tests)

**Expected Output:**
```
running 45 tests
test commitment_tests::test_basic_commitment ... ok
test commitment_tests::test_commitment_uniqueness ... ok
test nullifier_tests::test_prevent_double_spend ... ok
test merkle_tests::test_tree_insertion ... ok
test zksnark_tests::test_proof_verification ... ok
test xcm_tests::test_cross_chain_transfer ... ok
...

test result: ok. 45 passed; 0 failed; 0 ignored
```

### Test Runtime

```bash
# Run runtime-specific tests
cargo test -p parachain-template-runtime
```

---

## 🌐 Local Development

### Option 1: Zombienet (Recommended for Testing)

**Quick Start - Launch Local Network:**

```bash
# Install Zombienet (Linux)
wget https://github.com/paritytech/zombienet/releases/latest/download/zombienet-linux-x64
chmod +x zombienet-linux-x64
sudo mv zombienet-linux-x64 /usr/local/bin/zombienet

# Install polkadot binaries (required for relay chain)
# Download from: https://github.com/paritytech/polkadot-sdk/releases
# Then add to PATH:
export PATH="$PATH:<path/to/polkadot/binaries>"

# Launch the network
zombienet --provider native spawn zombienet.toml

# Access via browser:
# Relay Chain: http://localhost:9944
# Parachain: http://localhost:9988
```

### Option 2: Omni Node (For Runtime Development)

```bash
# Install polkadot-omni-node
cargo install polkadot-omni-node --git https://github.com/paritytech/polkadot-sdk --tag polkadot-stable2412

# Install chain-spec-builder
cargo install staging-chain-spec-builder --git https://github.com/paritytech/polkadot-sdk --tag polkadot-stable2412

# Build the runtime
cargo build --release

# Create chain spec
chain-spec-builder create \
  --relay-chain "rococo-local" \
  --para-id 1000 \
  --runtime target/release/wbuild/parachain-template-runtime/parachain_template_runtime.wasm \
  named-preset development > chain_spec.json

# Run Omni Node in dev mode
polkadot-omni-node --chain chain_spec.json --dev --dev-block-time 1000
```

### Option 3: Parachain Template Node

```bash
# Build and install the node
cargo install --path node

# Use zombienet for full setup
zombienet --provider native spawn zombienet.toml
```

---

## 📦 Deployment

### Paseo Testnet Deployment

For detailed testnet deployment instructions, see our [deployment guides](https://github.com/Cross-chain-Cloak/Cloak/wiki/Deployment).

**Quick Overview:**

1. **Get PAS Tokens**: https://faucet.polkadot.io (select Paseo network)
2. **Reserve ParaID**: Via [Polkadot.js Apps](https://polkadot.js.org/apps/?rpc=wss://paseo.rpc.amforc.com#/parachains)
3. **Generate Keys**: Collator and session keys
4. **Build WASM**: `cargo build --release`
5. **Create Chain Spec**: Using `chain-spec-builder`
6. **Export Genesis**: WASM and state files
7. **Register**: Upload genesis files to Paseo
8. **Start Collator**: Run your collator node
9. **Obtain Coretime**: For block production
10. **Verify**: Check block production

**Estimated Time:** 3-6 hours (mostly sync/onboarding)

---

## 📁 Project Structure

```
Cloak/
├── Cargo.toml                    # Workspace configuration
├── Dockerfile                    # Container build
├── LICENSE                       # Unlicense
├── README.md                     # This file
│
├── pallets/
│   ├── privacy-bridge/           # Privacy Bridge pallet (main feature)
│   │   ├── src/
│   │   │   ├── lib.rs            # Pallet implementation
│   │   │   ├── zksnark.rs        # zkSNARK proof system
│   │   │   ├── merkle_tree.rs    # Merkle tree anonymity sets
│   │   │   ├── tests.rs          # Test suite (45 tests)
│   │   │   └── weights.rs        # Benchmark weights
│   │   └── Cargo.toml
│   └── template/                 # Template pallet (example)
│
├── runtime/                      # Parachain runtime
│   ├── src/
│   │   ├── lib.rs                # Runtime configuration
│   │   └── configs/              # Pallet configurations
│   ├── build.rs                  # Build script
│   └── Cargo.toml
│
├── node/                         # Optional: Custom node binary
│   ├── src/
│   │   ├── main.rs               # Node entry point
│   │   ├── chain_spec.rs         # Chain specification
│   │   ├── cli.rs                # CLI configuration
│   │   ├── command.rs            # Command handling
│   │   ├── rpc.rs                # RPC endpoints
│   │   └── service.rs            # Node service
│   └── Cargo.toml
│
├── zombienet.toml                # Local network config (with custom node)
└── zombienet-omni-node.toml      # Local network config (with Omni Node)
```

---

## 🔧 Technical Specifications

### Blockchain

| Property | Value |
|----------|-------|
| **Framework** | Polkadot SDK (stable2412) |
| **Language** | Rust |
| **Token Symbol** | CLK (Cloak) |
| **Token Decimals** | 12 |
| **Block Time** | 12 seconds |
| **Consensus** | Aura (Authority Round) |
| **Finality** | Grandpa (via Relay Chain) |

### Cryptography

| Component | Specification |
|-----------|---------------|
| **zkSNARK Scheme** | Groth16 |
| **Elliptic Curve** | BN254 (bn128) |
| **Hash Function** | Poseidon (zkSNARK-optimized) |
| **Commitment Scheme** | Pedersen commitments |
| **Merkle Tree Depth** | 20 levels (1M+ deposits) |
| **Proof Size** | ~200 bytes (constant) |

### Cross-Chain

| Feature | Details |
|---------|---------|
| **Protocol** | XCM v5 |
| **Supported Chains** | All Polkadot/Kusama parachains |
| **Asset Types** | Native tokens, fungible assets |
| **Message Format** | XCM instructions |

---

## 💡 Use Cases

### 💼 Businesses & DAOs
- Private payroll across multiple chains
- Confidential supplier payments
- Treasury management with financial privacy

### 👤 Individual Users
- Protect personal financial privacy
- Break transaction history
- Private donations to causes

### 🏦 DeFi Applications
- Private liquidity provision
- Anonymous trading strategies
- Confidential yield farming

### 🌍 Cross-Border Payments
- Private international transfers
- Sender/receiver identity protection
- Bypass financial surveillance

---

## 🗺️ Roadmap

### ✅ Phase 1: Foundation (Completed)
- [x] Privacy Bridge pallet implementation
- [x] zkSNARK integration (Groth16/BN254)
- [x] Merkle tree anonymity sets
- [x] XCM v5 cross-chain integration
- [x] Comprehensive test suite (45 tests)
- [x] WASM runtime compilation

### 🚧 Phase 2: Testnet (In Progress)
- [ ] Deploy to Paseo testnet
- [ ] Community testing and feedback
- [ ] Stress testing and optimization
- [ ] Bug fixes and improvements

### 📋 Phase 3: Integration (Planned)
- [ ] Integrate with major parachains (Acala, Moonbeam, Astar)
- [ ] User-friendly dApp interface
- [ ] Additional asset type support
- [ ] Enhanced XCM features

### 🔒 Phase 4: Security (Planned)
- [ ] Third-party security audit
- [ ] Formal verification
- [ ] Bug bounty program
- [ ] Security documentation

### 🚀 Phase 5: Mainnet (Future)
- [ ] Mainnet parachain auction
- [ ] Production launch
- [ ] Ecosystem partnerships
- [ ] Ongoing maintenance

---

## 🤝 Contributing

We welcome contributions! Here's how:

1. **Fork the repository**
2. **Create feature branch**: `git checkout -b feature/amazing-feature`
3. **Make changes**
4. **Run tests**: `cargo test -p pallet-privacy-bridge`
5. **Commit**: `git commit -m "Add amazing feature"`
6. **Push**: `git push origin feature/amazing-feature`
7. **Open Pull Request**

### Code Style

- Format code: `cargo fmt`
- Check lints: `cargo clippy`
- Add tests for new features
- Update documentation

---

## 🐛 Troubleshooting

### Build Issues

**Problem:** `Cannot compile the WASM runtime: wasm32-unknown-unknown target not installed`

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

---

**Problem:** `Cannot compile the WASM runtime: no standard library sources found`

**Solution:**
```bash
rustup component add rust-src
```

---

**Problem:** `No space left on device (os error 28)`

**Solution:**
```bash
cargo clean
df -h  # Check disk space
```

---

**Problem:** Build takes too long or runs out of memory

**Solution:**
```bash
# Reduce parallel compilation
cargo build --release -j 2
```

---

## 📚 Resources

### Documentation
- [Polkadot Wiki](https://wiki.polkadot.network/)
- [Substrate Documentation](https://docs.substrate.io/)
- [XCM Documentation](https://wiki.polkadot.network/docs/learn-xcm)
- [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)

### Development Tools
- [Polkadot.js Apps](https://polkadot.js.org/apps/)
- [Zombienet](https://github.com/paritytech/zombienet)
- [Chopsticks](https://github.com/AcalaNetwork/chopsticks) - Runtime development tool

---

## 🙏 Acknowledgments

Built with:
- [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)
- [Substrate Framework](https://substrate.io/)
- [Arkworks zkSNARK Libraries](https://github.com/arkworks-rs)

Inspired by:
- [Tornado Cash](https://tornado.cash/) - Privacy on Ethereum
- [Zcash](https://z.cash/) - Privacy-focused cryptocurrency
- [Aztec Protocol](https://aztec.network/) - Privacy on Ethereum

---

## 📄 License

This project is released into the public domain under the [Unlicense](https://unlicense.org/).

See [LICENSE](./LICENSE) for details.

---

## 📧 Contact

**GitHub**: [Cross-chain-Cloak/Cloak](https://github.com/Cross-chain-Cloak/Cloak)

---

<div align="center">

**Built with ❤️ for the Polkadot Ecosystem**

[⭐ Star us on GitHub](https://github.com/Cross-chain-Cloak/Cloak) | [🐛 Report Bug](https://github.com/Cross-chain-Cloak/Cloak/issues) | [💡 Request Feature](https://github.com/Cross-chain-Cloak/Cloak/issues)

</div>
