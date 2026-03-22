## 🎨 DESIGN COMMENT 1/5: Mobile Node Architecture

### Lightweight Mining App Design

**App Stack:**
- **iOS:** Swift/SwiftUI, ~15-20MB app size (bare minimum)
- **Android:** Kotlin/Jetpack Compose, ~12-18MB APK
- **Shared Logic:** Rust core for proof generation (compiled to C bindings via ffi)

**Core Components:**
1. **ProofEngine (Rust):** Handles PoW-like benchmark computations. Compiled to static lib, called from Swift/Kotlin via FFI. Runs in isolated thread, can be paused/resumed.
2. **SyncManager:** Handles intermittent connectivity. Uses a compressed state format - only syncs delta changes, not full chain data.
3. **TrustScore Module:** Local reputation tracking. Calculates device reliability based on uptime, participation rate, and benchmark consistency.

**Proof Submission Protocol:**
Mobile nodes submit proofs of work via a compressed binary format (~200-500 bytes):
- node_id: [u8; 32] - BLS pubkey
- challenge: [u8; 32] - Server-generated challenge  
- response: [u8; 64] - Benchmarked computation result
- timestamp: u64 - Unix timestamp (enforced within ±30s)
- trust_score: f32 - 0.0-1.0 reputation multiplier
- signature: [u8; 64] - Ed25519 signature over payload

**Battery Optimization:**
- Benchmark computations throttled to execute only when device is: charging OR battery > 50%
- Background work limited to 15-minute windows per hour (iOS background task API / Android WorkManager)
- Average power draw: ~5-10% battery/hour during active mining (vs 15-20% for typical gaming)
- CPU frequency scaling enforced - no boost mode during mining

**Data Usage:**
- Light sync: ~5-10MB/day (compressed deltas)
- Proof submissions: ~50KB/day (batched every 5 minutes)
- Total monthly: ~200-400MB (comparable to Pi Network's ~300MB/month)
- WiFi-only sync option for metered connections

**Offline Capability:**
- Nodes can queue proofs locally for up to 24 hours
- On reconnect, batch submit queued proofs (FIFO order preserved)
- Proofs rejected if >24h old (timestamp enforced by validators)
- Local SQLite DB stores pending proofs with retry logic and exponential backoff

**Security:**
- Keystore/Keychain for private keys (hardware-backed on modern devices)
- App integrity checks via SafetyNet (Android) / DeviceCheck (iOS)
- Root/rooted device detection → reduced trust score or rejection