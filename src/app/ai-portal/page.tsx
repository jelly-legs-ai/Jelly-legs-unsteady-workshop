'use client';

import React, { useState, useEffect, useCallback } from "react";
import Head from "next/head";
import { useWallet } from "@solana/wallet-adapter-react";
import { useWalletModal } from "@solana/wallet-adapter-react-ui";

/**
 * AI Operator Portal - Priority Lane Management for AI Agents
 *
 * Features:
 * - Priority Lane Dashboard (Critical / High / Standard)
 * - Submit AI Transactions to specific lanes
 * - Live wallet connection status
 * - Network health monitoring
 */

interface PriorityLaneStats {
  lane: "critical" | "high" | "standard";
  label: string;
  color: string;
  glowColor: string;
  queueDepth: number;
  avgWaitTime: number;
  tpsLimit: number;
  currentTps: number;
  premium: number; // extra fee %
}

interface NetworkStats {
  slot: number;
  tps: number;
  epoch: number;
  epochProgress: string;
  blockHeight: number;
  chainVersion: string;
  connected: boolean;
}

interface SubmittedTx {
  id: string;
  lane: string;
  amount: number;
  status: "pending" | "confirmed" | "failed";
  signature?: string;
  timestamp: Date;
}

const PRIORITY_LANES: PriorityLaneStats[] = [
  {
    lane: "critical",
    label: "Critical",
    color: "text-red-400",
    glowColor: "shadow-red-500/30",
    queueDepth: 0,
    avgWaitTime: 0,
    tpsLimit: 50000,
    currentTps: 0,
    premium: 100,
  },
  {
    lane: "high",
    label: "High",
    color: "text-orange-400",
    glowColor: "shadow-orange-500/30",
    queueDepth: 0,
    avgWaitTime: 0,
    tpsLimit: 30000,
    currentTps: 0,
    premium: 50,
  },
  {
    lane: "standard",
    label: "Standard",
    color: "text-blue-400",
    glowColor: "shadow-blue-500/30",
    queueDepth: 0,
    avgWaitTime: 0,
    tpsLimit: 15000,
    currentTps: 0,
    premium: 0,
  },
];

function LaneCard({
  lane,
  stats,
}: {
  lane: PriorityLaneStats;
  stats?: NetworkStats;
}) {
  const utilization = stats?.tps
    ? Math.min(100, (stats.tps / lane.tpsLimit) * 100)
    : 0;

  return (
    <div
      className={`bg-gray-800/50 backdrop-blur-sm border border-gray-700/50 rounded-xl p-6 hover:border-${lane.color.split('-')[1]}-500/40 transition-all`}
    >
      <div className="flex items-center justify-between mb-4">
        <div className={`text-xl font-bold ${lane.color}`}>{lane.label}</div>
        <div className="text-xs text-gray-500 uppercase tracking-wider">
          Priority Lane
        </div>
      </div>

      <div className="space-y-3">
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Queue Depth</span>
          <span className="text-white font-mono font-medium">
            {lane.queueDepth} tx
          </span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Avg Wait Time</span>
          <span className="text-white font-mono font-medium">
            {lane.avgWaitTime}ms
          </span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">TPS Limit</span>
          <span className="text-white font-mono font-medium">
            {lane.tpsLimit.toLocaleString()}
          </span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Current TPS</span>
          <span className={`font-mono font-medium ${stats?.tps ? "text-green-400" : "text-gray-500"}`}>
            {stats?.tps ? stats.tps.toLocaleString() : "—"}
          </span>
        </div>

        {/* Utilization bar */}
        <div className="mt-4">
          <div className="flex justify-between text-xs mb-1">
            <span className="text-gray-500">Network Utilization</span>
            <span className="text-gray-400">{utilization.toFixed(1)}%</span>
          </div>
          <div className="h-2 bg-gray-900 rounded-full overflow-hidden">
            <div
              className={`h-full ${lane.lane === "critical" ? "bg-red-500" : lane.lane === "high" ? "bg-orange-500" : "bg-blue-500"} rounded-full transition-all`}
              style={{ width: `${Math.min(100, utilization)}%` }}
            />
          </div>
        </div>

        <div className="flex justify-between items-center pt-2 border-t border-gray-700/50">
          <span className="text-sm text-gray-400">Premium Fee</span>
          <span className={`font-bold ${lane.premium > 0 ? "text-yellow-400" : "text-gray-500"}`}>
            {lane.premium > 0 ? `+${lane.premium}%` : "Base"}
          </span>
        </div>
      </div>
    </div>
  );
}

export default function AIPortalPage() {
  const { connected, publicKey, connecting, signTransaction, signAllTransactions, signMessage } = useWallet();
  const { setVisible } = useWalletModal();

  const walletAddress = publicKey ? publicKey.toBase58() : "";
  const shortAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 6)}...${publicKey.toBase58().slice(-4)}`
    : "";

  // Network stats
  const [networkStats, setNetworkStats] = useState<NetworkStats | null>(null);
  const [loadingNetwork, setLoadingNetwork] = useState(true);

  // Transaction submission
  const [selectedLane, setSelectedLane] = useState<"critical" | "high" | "standard">("high");
  const [txAmount, setTxAmount] = useState<string>("");
  const [txMemo, setTxMemo] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [txHistory, setTxHistory] = useState<SubmittedTx[]>([]);

  // UI state
  const [error, setError] = useState<string>("");
  const [success, setSuccess] = useState<string>("");

  /**
   * Fetch network stats from /api/network-stats
   */
  const fetchNetworkStats = useCallback(async () => {
    try {
      const res = await fetch("/api/network-stats");
      if (res.ok) {
        const data = await res.json();
        setNetworkStats(data);
      }
    } catch {
      // silently fail — component handles null state
    } finally {
      setLoadingNetwork(false);
    }
  }, []);

  useEffect(() => {
    fetchNetworkStats();
    const interval = setInterval(fetchNetworkStats, 30000);
    return () => clearInterval(interval);
  }, [fetchNetworkStats]);

  /**
   * Submit an AI transaction to a priority lane
   * Wires to POST /v1/ai_priority/submit via the SDK pattern
   */
  const handleSubmitTx = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!connected || !publicKey) {
      setError("Please connect your wallet first");
      return;
    }

    const amount = parseFloat(txAmount);
    if (isNaN(amount) || amount <= 0) {
      setError("Please enter a valid amount");
      return;
    }

    setIsSubmitting(true);
    setError("");
    setSuccess("");

    const txId = `tx_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    const newTx: SubmittedTx = {
      id: txId,
      lane: selectedLane,
      amount,
      status: "pending",
      timestamp: new Date(),
    };

    try {
      // Attempt POST to /v1/ai_priority/submit
      // This will fail gracefully in demo mode (no real SDK endpoint)
      const response = await fetch("/v1/ai_priority/submit", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          wallet: walletAddress,
          lane: selectedLane,
          amount,
          memo: txMemo,
          txId,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        setTxHistory((prev) => [
          {
            ...newTx,
            status: "confirmed",
            signature: data.signature || data.txHash,
          },
          ...prev,
        ]);
        setSuccess(
          `Transaction submitted to ${selectedLane.toUpperCase()} lane! TX: ${(data.signature || data.txHash || txId).slice(0, 16)}...`
        );
      } else {
        // Non-2xx — still record as demo
        const errorData = await response.json().catch(() => ({}));
        if (response.status === 404) {
          // Endpoint not implemented yet — record as demo pending
          setTxHistory((prev) => [{ ...newTx }, ...prev]);
          setSuccess(
            `Demo: Transaction queued to ${selectedLane.toUpperCase()} lane (endpoint not yet deployed). Amount: ${amount} ATH`
          );
        } else {
          throw new Error(errorData.error || `Server error ${response.status}`);
        }
      }
    } catch (err: any) {
      if (err.message?.includes("fetch failed") || err.message?.includes("404")) {
        // Network error — record as demo
        setTxHistory((prev) => [{ ...newTx }, ...prev]);
        setSuccess(
          `Demo: Transaction queued to ${selectedLane.toUpperCase()} lane. Amount: ${amount} ATH. Connect to real RPC to submit on-chain.`
        );
      } else {
        setError(err.message || "Transaction submission failed");
        setTxHistory((prev) => [
          { ...newTx, status: "failed" as const },
          ...prev,
        ]);
      }
    } finally {
      setIsSubmitting(false);
      setTxAmount("");
      setTxMemo("");
    }
  };

  // Auto-clear messages
  useEffect(() => {
    if (error || success) {
      const timer = setTimeout(() => {
        setError("");
        setSuccess("");
      }, 6000);
      return () => clearTimeout(timer);
    }
  }, [error, success]);

  const totalTxSubmitted = txHistory.length;
  const pendingTx = txHistory.filter((tx) => tx.status === "pending").length;

  return (
    <>
      <Head>
        <title>AI Operator Portal — Aether Chain</title>
      </Head>
      <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white">
        <div className="max-w-6xl mx-auto px-4 py-10">
          {/* Hero */}
          <div className="text-center mb-12">
            <div className="inline-flex items-center gap-2 px-4 py-2 bg-purple-500/10 border border-purple-500/30 rounded-full text-purple-300 text-sm font-medium mb-6">
              <span className="w-2 h-2 bg-purple-400 rounded-full animate-pulse" />
              AI Operator Portal
            </div>
            <h1 className="text-4xl md:text-5xl font-bold text-center mb-4">
              <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
                Priority Lane
              </span>
              <br />
              <span className="bg-gradient-to-r from-red-400 via-red-500 to-red-600 bg-clip-text text-transparent">
                Management
              </span>
            </h1>
            <p className="text-gray-400 text-lg max-w-2xl mx-auto">
              Submit AI agent transactions to Critical, High, or Standard priority
              lanes. Pay premium gas for mission-critical workloads.
            </p>
          </div>

          {/* Error/Success */}
          {error && (
            <div className="mb-6 p-4 bg-red-900/50 border border-red-500 rounded-lg text-red-200 text-sm">
              {error}
            </div>
          )}
          {success && (
            <div className="mb-6 p-4 bg-green-900/50 border border-green-500 rounded-lg text-green-200 text-sm">
              {success}
            </div>
          )}

          {/* Wallet Status */}
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700/50 rounded-xl p-5 mb-8">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div className="flex items-center gap-3">
                {connected ? (
                  <>
                    <div className="w-10 h-10 bg-green-500/20 border border-green-500/40 rounded-full flex items-center justify-center">
                      <span className="text-green-400 text-lg">✓</span>
                    </div>
                    <div>
                      <div className="text-green-400 font-semibold">
                        Wallet Connected
                      </div>
                      <div className="text-gray-400 font-mono text-sm">
                        {shortAddress}
                      </div>
                    </div>
                  </>
                ) : (
                  <>
                    <div className="w-10 h-10 bg-gray-700/50 border border-gray-600 rounded-full flex items-center justify-center">
                      <span className="text-gray-400 text-lg">◯</span>
                    </div>
                    <div>
                      <div className="text-gray-400 font-semibold">
                        Wallet Not Connected
                      </div>
                      <div className="text-gray-500 text-sm">
                        Connect to submit transactions
                      </div>
                    </div>
                  </>
                )}
              </div>

              {/* Session stats */}
              <div className="flex gap-6 text-sm">
                <div>
                  <span className="text-gray-500">Submitted: </span>
                  <span className="text-white font-medium">{totalTxSubmitted}</span>
                </div>
                <div>
                  <span className="text-gray-500">Pending: </span>
                  <span className="text-yellow-400 font-medium">{pendingTx}</span>
                </div>
              </div>

              {!connected && (
                <button
                  onClick={() => setVisible(true)}
                  disabled={connecting}
                  className="px-5 py-2.5 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-lg font-semibold text-sm transition-all disabled:opacity-50"
                >
                  {connecting ? "Connecting..." : "Connect Wallet"}
                </button>
              )}
            </div>
          </div>

          {/* Priority Lane Dashboard */}
          <div className="mb-10">
            <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-2">
              <span className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
              Priority Lanes
              {networkStats?.connected && (
                <span className="text-sm font-normal text-gray-500 ml-2">
                  Live · Epoch {networkStats.epoch} · Slot {networkStats.slot?.toLocaleString()}
                </span>
              )}
            </h2>
            <div className="grid md:grid-cols-3 gap-6">
              {PRIORITY_LANES.map((lane) => (
                <LaneCard key={lane.lane} lane={lane} stats={networkStats ?? undefined} />
              ))}
            </div>
          </div>

          {/* Submit AI Transaction */}
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700/50 rounded-xl p-6 mb-8">
            <h2 className="text-xl font-bold text-white mb-2">
              Submit AI Transaction
            </h2>
            <p className="text-gray-400 text-sm mb-6">
              Route your AI agent transaction through a priority lane. Premium
              gas fees apply for Critical and High lanes.
            </p>

            <form onSubmit={handleSubmitTx} className="space-y-5">
              {/* Lane selection */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  Priority Lane
                </label>
                <div className="grid grid-cols-3 gap-3">
                  {PRIORITY_LANES.map((lane) => (
                    <button
                      key={lane.lane}
                      type="button"
                      onClick={() => setSelectedLane(lane.lane)}
                      className={`py-3 px-4 rounded-lg border font-medium text-sm transition-all ${
                        selectedLane === lane.lane
                          ? lane.lane === "critical"
                            ? "bg-red-600/20 border-red-500 text-red-400"
                            : lane.lane === "high"
                            ? "bg-orange-600/20 border-orange-500 text-orange-400"
                            : "bg-blue-600/20 border-blue-500 text-blue-400"
                          : "bg-gray-900 border-gray-700 text-gray-400 hover:border-gray-600"
                      }`}
                    >
                      <div>{lane.label}</div>
                      <div className="text-xs mt-0.5 opacity-70">
                        {lane.premium > 0 ? `+${lane.premium}% fee` : "Base fee"}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Amount */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  Amount (ATH)
                </label>
                <input
                  type="number"
                  value={txAmount}
                  onChange={(e) => setTxAmount(e.target.value)}
                  placeholder="0.00"
                  min="0"
                  step="0.01"
                  className="w-full px-4 py-3 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:border-red-500 transition-colors text-white placeholder-gray-600"
                />
              </div>

              {/* Memo */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">
                  Memo / Task Description
                </label>
                <textarea
                  value={txMemo}
                  onChange={(e) => setTxMemo(e.target.value)}
                  placeholder="Describe the AI task or transaction purpose..."
                  rows={3}
                  className="w-full px-4 py-3 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:border-red-500 transition-colors text-white placeholder-gray-600 resize-none"
                />
              </div>

              <button
                type="submit"
                disabled={isSubmitting || !connected || !txAmount}
                className={`w-full py-3 rounded-lg font-semibold transition-all disabled:opacity-50 text-sm ${
                  selectedLane === "critical"
                    ? "bg-gradient-to-r from-red-700 to-red-800 hover:from-red-600 hover:to-red-700"
                    : selectedLane === "high"
                    ? "bg-gradient-to-r from-orange-700 to-orange-800 hover:from-orange-600 hover:to-orange-700"
                    : "bg-gradient-to-r from-blue-700 to-blue-800 hover:from-blue-600 hover:to-blue-700"
                }`}
              >
                {isSubmitting
                  ? "Submitting..."
                  : `Submit to ${selectedLane.toUpperCase()} Lane`}
              </button>
            </form>
          </div>

          {/* Transaction History */}
          {txHistory.length > 0 && (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700/50 rounded-xl p-6">
              <h2 className="text-xl font-bold text-white mb-4">
                Transaction History
              </h2>
              <div className="space-y-3">
                {txHistory.map((tx) => (
                  <div
                    key={tx.id}
                    className="flex items-center justify-between bg-gray-900/50 border border-gray-700/50 rounded-lg px-4 py-3"
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-2 h-2 rounded-full ${
                          tx.status === "confirmed"
                            ? "bg-green-400"
                            : tx.status === "failed"
                            ? "bg-red-400"
                            : "bg-yellow-400 animate-pulse"
                        }`}
                      />
                      <div>
                        <div className="text-white text-sm font-medium">
                          {tx.lane.toUpperCase()} Lane
                        </div>
                        <div className="text-gray-500 text-xs font-mono">
                          {tx.amount} ATH · {tx.timestamp.toLocaleTimeString()}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <span
                        className={`text-xs px-2 py-1 rounded-full ${
                          tx.status === "confirmed"
                            ? "bg-green-500/20 text-green-400"
                            : tx.status === "failed"
                            ? "bg-red-500/20 text-red-400"
                            : "bg-yellow-500/20 text-yellow-400"
                        }`}
                      >
                        {tx.status}
                      </span>
                      {tx.signature && (
                        <span className="text-gray-500 text-xs font-mono">
                          {tx.signature.slice(0, 12)}...
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
