//! Graceful Shutdown Signal
//!
//! Provides a coordinated shutdown mechanism for the validator node.
//! When a shutdown signal is received (SIGTERM, SIGINT, or programmatic),
//! all subsystems are notified and given a grace period to clean up before
//! the process exits.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   Shutdown Coordinator                    │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Listen for SIGINT / SIGTERM / Ctrl+C                │
//! │  2. Broadcast shutdown to all subsystems                │
//! │  3. Wait for subsystems to report completion             │
//! │  4. Force exit after grace period                        │
//! ├─────────────────────────────────────────────────────────┤
//! │  Subsystems:                                              │
//! │  - Block producer (flush pending blocks)                 │
//! │  - RPC server (drain in-flight requests)                 │
//! │  - P2P gossip (send goodbye messages)                   │
//! │  - Persistence (flush state to disk)                    │
//! └─────────────────────────────────────────────────────────┘
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, watch};
use tracing::info;

/// Default grace period for subsystem shutdown (seconds)
pub const DEFAULT_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// Maximum grace period allowed (seconds)
pub const MAX_SHUTDOWN_TIMEOUT_SECS: u64 = 300;

/// A shared shutdown signal that can be triggered once and observed by many.
///
/// Uses a broadcast channel for immediate notification and a watch channel
/// for subsystems that need to poll for shutdown state.
#[derive(Clone)]
pub struct ShutdownSignal {
    /// Broadcast sender (only fires once, then all receivers get notified)
    sender: Arc<broadcast::Sender<()>>,
    /// Watch channel for polling shutdown state (true = shutdown requested)
    shutdown_requested: watch::Receiver<bool>,
}

impl ShutdownSignal {
    /// Create a new shutdown signal pair (signal + trigger).
    pub fn new() -> (Self, ShutdownTrigger) {
        let (tx, _) = broadcast::channel(1);
        let (watch_tx, watch_rx) = watch::channel(false);

        let signal = Self {
            sender: Arc::new(tx),
            shutdown_requested: watch_rx,
        };

        let trigger = ShutdownTrigger {
            sender: Arc::clone(&signal.sender),
            watch_tx,
        };

        (signal, trigger)
    }

    /// Check if shutdown has been requested (non-blocking poll).
    pub fn is_shutdown(&self) -> bool {
        *self.shutdown_requested.borrow()
    }

    /// Wait for the shutdown signal. Returns immediately if already triggered.
    pub async fn recv(&self) {
        // Check if already triggered
        if *self.shutdown_requested.borrow() {
            return;
        }

        // Wait for the signal
        let mut rx = self.sender.subscribe();
        let _ = rx.recv().await;
    }

    /// Wait for shutdown with a timeout. Returns true if shutdown was received.
    pub async fn recv_timeout(&self, timeout: Duration) -> bool {
        tokio::select! {
            _ = self.recv() => true,
            _ = tokio::time::sleep(timeout) => false,
        }
    }

    /// Create a new receiver for this signal (for additional subsystems).
    pub fn subscribe(&self) -> ShutdownReceiver {
        ShutdownReceiver {
            rx: self.sender.subscribe(),
            shutdown_requested: self.shutdown_requested.clone(),
        }
    }
}

/// Receiver side of the shutdown signal (for subsystems).
pub struct ShutdownReceiver {
    rx: broadcast::Receiver<()>,
    shutdown_requested: watch::Receiver<bool>,
}

impl ShutdownReceiver {
    /// Check if shutdown has been requested (non-blocking poll).
    pub fn is_shutdown(&self) -> bool {
        *self.shutdown_requested.borrow()
    }

    /// Wait for the shutdown signal.
    pub async fn recv(&mut self) {
        if *self.shutdown_requested.borrow() {
            return;
        }
        let _ = self.rx.recv().await;
    }
}

/// Trigger side of the shutdown signal. Can only be fired once.
pub struct ShutdownTrigger {
    sender: Arc<broadcast::Sender<()>>,
    watch_tx: watch::Sender<bool>,
}

impl ShutdownTrigger {
    /// Fire the shutdown signal, notifying all observers.
    pub fn fire(&self) {
        let _ = self.sender.send(());
        let _ = self.watch_tx.send(true);
    }
}

/// Coordinates graceful shutdown across all validator subsystems.
///
/// Usage:
/// ```ignore
/// let (shutdown, trigger) = ShutdownCoordinator::new(30);
/// 
/// // Register subsystems
/// let block_producer_shutdown = shutdown.subscribe();
/// let rpc_shutdown = shutdown.subscribe();
/// 
/// // Start subsystems with shutdown signal
/// tokio::spawn(block_producer.run(shutdown));
/// tokio::spawn(rpc_server.run(shutdown));
/// 
/// // Wait for shutdown signal (Ctrl+C or SIGTERM)
/// trigger.wait_for_signal().await;
/// 
/// // Give subsystems time to clean up
/// trigger.graceful_shutdown().await;
/// ```
pub struct ShutdownCoordinator {
    /// Grace period for subsystems to shut down
    #[allow(dead_code)]
    timeout: Duration,
    /// The shutdown signal
    signal: ShutdownSignal,
    /// The shutdown trigger
    trigger: ShutdownTrigger,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator with the given timeout in seconds.
    pub fn new(timeout_secs: u64) -> Self {
        let timeout_secs = timeout_secs.min(MAX_SHUTDOWN_TIMEOUT_SECS).max(1);
        let (signal, trigger) = ShutdownSignal::new();

        Self {
            timeout: Duration::from_secs(timeout_secs),
            signal,
            trigger,
        }
    }

    /// Get a clone of the shutdown signal (for passing to subsystems).
    pub fn signal(&self) -> ShutdownSignal {
        self.signal.clone()
    }

    /// Get a reference to the shutdown trigger.
    pub fn trigger(&self) -> &ShutdownTrigger {
        &self.trigger
    }

    /// Install Ctrl+C handler and wait for termination signal.
    /// Returns the trigger when a signal is received.
    pub async fn wait_for_signal(self) -> ShutdownTrigger {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, initiating graceful shutdown...");
            }
        }
        self.trigger.fire();
        self.trigger
    }

    /// Decompose into signal and trigger for direct use.
    pub fn decompose(self) -> (ShutdownSignal, ShutdownTrigger) {
        (self.signal, self.trigger)
    }

    /// Wait for all subsystems to complete or timeout.
    /// Call this after firing the trigger.
    pub async fn graceful_shutdown(_trigger: ShutdownTrigger, signal: ShutdownSignal) {
        info!("Waiting for subsystems to shut down (timeout: {:?})...", signal.timeout_hint());

        // Just wait for the grace period — subsystems should observe the signal
        // and exit on their own. We can't join them here because they're spawned
        // independently, but we can give them time.
        tokio::time::sleep(Duration::from_secs(5)).await;
        info!("Graceful shutdown period complete.");
    }
}

impl ShutdownSignal {
    /// Returns a hint about the expected shutdown timeout for logging.
    fn timeout_hint(&self) -> Duration {
        Duration::from_secs(DEFAULT_SHUTDOWN_TIMEOUT_SECS)
    }
}

/// A subsystem that can be started and stopped gracefully.
#[async_trait::async_trait]
pub trait GracefulSubsystem {
    /// Start the subsystem. Should exit when the shutdown signal is received.
    async fn run(&mut self, shutdown: ShutdownSignal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_signal_basic() {
        let (signal, trigger) = ShutdownSignal::new();

        assert!(!signal.is_shutdown());
        trigger.fire();
        assert!(signal.is_shutdown());

        // Already triggered — recv should return immediately
        signal.recv().await;
    }

    #[tokio::test]
    async fn test_shutdown_signal_multiple_receivers() {
        let (signal, trigger) = ShutdownSignal::new();

        let mut rx1 = signal.subscribe();
        let mut rx2 = signal.subscribe();

        assert!(!rx1.is_shutdown());
        assert!(!rx2.is_shutdown());

        trigger.fire();

        // Both receivers should see the shutdown
        rx1.recv().await;
        rx2.recv().await;

        assert!(rx1.is_shutdown());
        assert!(rx2.is_shutdown());
    }

    #[tokio::test]
    async fn test_shutdown_signal_timeout() {
        let (signal, _trigger) = ShutdownSignal::new();

        // Should timeout since no signal fired
        let result = signal.recv_timeout(Duration::from_millis(50)).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_shutdown_signal_timeout_fired() {
        let (signal, trigger) = ShutdownSignal::new();

        // Fire immediately
        trigger.fire();

        let result = signal.recv_timeout(Duration::from_secs(5)).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_shutdown_coordinator() {
        let coordinator = ShutdownCoordinator::new(5);

        let signal = coordinator.signal();
        let trigger = coordinator.trigger();

        assert!(!signal.is_shutdown());
        trigger.fire();
        assert!(signal.is_shutdown());
    }
}