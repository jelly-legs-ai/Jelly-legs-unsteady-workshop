# AETHER Mobile App Architecture

## 1. Overview

**AETHER Mobile** is a React Native (Expo) application for iOS and Android that enables participants to contribute Proof-of-Work (PoW) from mobile devices to the AETHER blockchain network. The app wraps native Rust-based mining libraries via a JSI (JavaScript Interface) bridge for performance-critical operations, while providing a clean, battery-conscious UI for monitoring mining activity, rewards, and network status.

- **Framework:** React Native with Expo (managed workflow, with custom native modules via `expo prebuild`)
- **Languages:** TypeScript (primary), Rust (native PoW engine), Kotlin/Swift (native bindings)
- **State Management:** Zustand (lightweight, minimal re-renders)
- **Navigation:** React Navigation v7 (bottom tabs + stack)
- **Target Platforms:** iOS 14+, Android API 24+

---

## 2. Project Structure

```
aether-mobile/
├── src/
│   ├── proof_engine.ts       # TypeScript wrapper around native Rust PoW library
│   ├── api.ts                # Backend API calls (validators, rewards, sync)
│   ├── storage.ts            # Local persistence (SecureStore, AsyncStorage)
│   ├── hooks/                # Custom React hooks
│   │   ├── useMining.ts      # Mining state & controls
│   │   ├── useNetwork.ts     # Validator connectivity
│   │   └── useBattery.ts     # Battery-aware mining throttle
│   ├── screens/
│   │   ├── LoginScreen.tsx
│   │   ├── DashboardScreen.tsx
│   │   ├── MiningScreen.tsx
│   │   ├── RewardsScreen.tsx
│   │   └── SettingsScreen.tsx
│   ├── components/
│   │   ├── MiningCard.tsx
│   │   ├── RewardBadge.tsx
│   │   ├── ValidatorStatus.tsx
│   │   └── OfflineBanner.tsx
│   ├── store/
│   │   └── miningStore.ts    # Zustand store for global mining state
│   ├── types/
│   │   └── index.ts          # Shared TypeScript types
│   └── utils/
│       ├── battery.ts        # Battery monitoring utilities
│       └── network.ts        # Network reachability helpers
├── android/                  # Native Android project (expo prebuild)
├── ios/                      # Native iOS project (expo prebuild)
├── native/
│   └── aether-pow/           # Rust crate for PoW engine
│       ├── src/
│       │   └── lib.rs
│       ├── Cargo.toml
│       └── android/
│       └── ios/
└── app.json
```

### Key Modules

#### `src/proof_engine.ts`
Wraps the native Rust PoW library via JSI. Provides an async API for:
- `startMining(threads: number): Promise<void>`
- `stopMining(): Promise<void>`
- `getProgress(): Promise<MiningProgress>`
- `submitProof(work: WorkUnit): Promise<ProofResult>`

The Rust library handles SHA-3/Keccak hashing, difficulty adjustment, and nonce search. All heavy computation stays off the JS thread.

#### `src/api.ts`
Handles all communication with the AETHER validator network:
- `POST /v1/miner/register` — Register mobile miner with a validator
- `POST /v1/miner/submit` — Submit proof of work
- `GET /v1/miner/rewards` — Fetch accumulated rewards
- `GET /v1/validators/status` — Health check for connected validators
- `GET /v1/sync/block` — Sync latest block headers

#### `src/storage.ts`
Local persistence layer:
- **SecureStore** (expo-secure-store): Private keys, auth tokens
- **AsyncStorage** (non-sensitive config: mining prefs, UI state)

---

## 3. Validator Communication

Mobile clients maintain a **long-poll HTTP/2 connection** to a nearest validator node. Communication flow:

```
┌──────────────┐    HTTPS/WSS    ┌────────────────┐
│  Mobile App  │ ◄─────────────► │  Validator Node │
│  (React Native)│               │  (Rust + Axum)   │
└──────┬───────┘                └────────┬────────┘
       │                                  │
       │  1. Register with validator      │
       │  2. Receive WorkUnit (block hdr)  │
       │  3. Run PoW locally (Rust engine) │
       │  4. Submit ProofResult            │
       │  5. Receive rewards + next unit   │
       └──────────────────────────────────┘
```

- **Protocol:** HTTPS REST for submissions; WebSocket for real-time work dispatch
- **Failover:** On validator disconnect, app auto-reconnects to next-nearest validator from a hardcoded seed list
- **Compression:** Proof submissions are gzipped to minimize data usage

---

## 4. Battery & Data Usage Considerations

Mining is intentionally **resource-conservative** on mobile:

| Setting      | CPU Threads | Battery Impact  | Hash Rate    |
|--------------|-------------|-----------------|--------------|
| Low          | 1           | ~5%/hr          | ~500 H/s     |
| Medium       | 2           | ~12%/hr         | ~1,200 H/s   |
| High         | 4           | ~25%/hr         | ~3,000 H/s   |

- **Adaptive Throttling:** The `useBattery` hook monitors device battery level via the Battery Status API. Mining automatically pauses when:
  - Battery < 20% (user-configurable threshold)
  - Device is on cellular and "Data Saver" mode is enabled
- **Work Scheduling:** Heavy mining sessions are deferred to Wi-Fi and >50% battery by default
- **Data Budget:** Default cap of 100 MB/day for validator communications; configurable in Settings
- **Chunked Work Units:** Block headers (~200 bytes) delivered as work units, keeping per-submission payloads tiny

---

## 5. Offline Capability

The app is designed to remain functional during network outages:

### Offline Mining
- Work units are pre-fetched and cached (up to 10 pending units) when online
- PoW runs entirely locally — no network required for computation
- Completed proofs are queued in **SQLite** (via `expo-sqlite`) for later submission
- Queue is auto-flushed when connectivity is restored

### Offline UI
- Dashboard shows last known rewards, cached block height, and mining statistics
- `OfflineBanner` component appears when no network is detected
- All screens render correctly from local state — no skeleton loaders blocking on network

### Sync Strategy
```
Online → Cache work units + Submit pending proofs → Update local state
Offline → Mine cached work units → Queue proofs in SQLite
Reconnect → Flush proof queue → Fetch updated rewards → Refresh UI
```

- **Conflict Resolution:** Each work unit has an expiration timestamp. Expired proofs are discarded; the validator issues a fresh unit on reconnect
- **Persistence:** SQLite stores the pending proof queue across app restarts, so proofs are never lost

---

## 6. Security Notes

- Private keys are stored in the OS keychain (iOS Keychain / Android Keystore) via `expo-secure-store`
- All API calls use certificate pinning
- Proof submissions are signed with the miner's private key before leaving the device
- The Rust PoW engine is sandboxed — it cannot access storage, network, or device sensors

---

## 7. Stack Summary

| Layer             | Technology                              |
|-------------------|-----------------------------------------|
| UI Framework      | React Native (Expo)                    |
| Language          | TypeScript                              |
| State             | Zustand                                 |
| Navigation        | React Navigation v7                     |
| Native Mining     | Rust (via react-native-rust-bridge/JSI) |
| Local DB          | SQLite (expo-sqlite)                    |
| Secure Storage    | expo-secure-store                       |
| HTTP Client       | Axios + HTTP/2                          |
| WebSocket         | expo-websockets                          |
| Battery Monitoring| expo Battery API                        |
| Build Targets     | iOS 14+, Android API 24+                |
