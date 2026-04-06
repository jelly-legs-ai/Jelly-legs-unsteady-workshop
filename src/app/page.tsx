"use client";

import { useEffect } from "react";
import Head from "next/head";

/**
 * Aether Hub — Home / Landing Page
 * 
 * Entry point to the Aether Chain ecosystem portal.
 * Features:
 * - Animated hero with chain stats
 * - Quick-nav cards to Staking, Explorer, and network tools
 * - Live validator count and network status
 */

export default function HomePage() {
  return (
    <>
      <Head>
        <title>Aether Chain Hub</title>
      </Head>
      <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white">
        {/* Nav */}
        <nav className="border-b border-gray-800 bg-black/20 backdrop-blur-sm">
          <div className="max-w-6xl mx-auto px-4 py-4 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <span className="text-3xl">🔱</span>
              <div>
                <h1 className="text-xl font-bold tracking-tight">Aether Chain</h1>
                <p className="text-xs text-gray-500">Ecosystem Portal</p>
              </div>
            </div>
            <div className="flex gap-4">
              <a
                href="/staking"
                className="px-4 py-2 bg-red-600/20 border border-red-500/30 rounded-lg text-sm hover:bg-red-600/30 transition-colors"
              >
                Stake
              </a>
              <a
                href="/explorer"
                className="px-4 py-2 bg-gray-700/50 border border-gray-600 rounded-lg text-sm hover:bg-gray-700 transition-colors"
              >
                Explorer
              </a>
            </div>
          </div>
        </nav>

        {/* Hero */}
        <div className="max-w-6xl mx-auto px-4 pt-20 pb-16 text-center">
          <div className="inline-block mb-6 px-4 py-1.5 bg-red-500/10 border border-red-500/20 rounded-full">
            <span className="text-sm text-red-400">⚡ Live on Testnet</span>
          </div>
          <h2 className="text-6xl font-black mb-4 tracking-tight">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Aether Chain
            </span>
          </h2>
          <p className="text-xl text-gray-400 max-w-2xl mx-auto mb-8">
            High-performance EVM-compatible chain with Proof of History consensus.
            Stake ATH, run validators, build dApps.
          </p>
          <div className="flex justify-center gap-4">
            <a
              href="/staking"
              className="px-8 py-3 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-xl font-bold text-lg transition-all shadow-lg shadow-red-900/30"
            >
              Start Staking
            </a>
            <a
              href="/explorer"
              className="px-8 py-3 bg-gray-800 border border-gray-700 hover:bg-gray-700 rounded-xl font-semibold text-lg transition-all"
            >
              Chain Explorer
            </a>
          </div>
        </div>

        {/* Quick Nav Cards */}
        <div className="max-w-6xl mx-auto px-4 pb-20">
          <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-6">
            Explore the Ecosystem
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <a
              href="/staking"
              className="group bg-gray-800/50 border border-gray-700 rounded-2xl p-6 hover:border-red-500/50 transition-all hover:shadow-xl hover:shadow-red-900/10"
            >
              <div className="text-4xl mb-4">📈</div>
              <h4 className="text-xl font-bold mb-2 group-hover:text-red-400 transition-colors">
                Staking Portal
              </h4>
              <p className="text-gray-400 text-sm mb-4">
                Stake ATH tokens and earn up to 15.5% APY. Three tier levels —
                Observer, Lite, and Full validator.
              </p>
              <div className="text-red-400 text-sm font-medium group-hover:translate-x-1 transition-transform">
                Open staking →
              </div>
            </a>

            <a
              href="/explorer"
              className="group bg-gray-800/50 border border-gray-700 rounded-2xl p-6 hover:border-blue-500/50 transition-all hover:shadow-xl hover:shadow-blue-900/10"
            >
              <div className="text-4xl mb-4">🔍</div>
              <h4 className="text-xl font-bold mb-2 group-hover:text-blue-400 transition-colors">
                Chain Explorer
              </h4>
              <p className="text-gray-400 text-sm mb-4">
                Look up addresses, transactions, and block data directly from the
                chain. Real-time RPC queries.
              </p>
              <div className="text-blue-400 text-sm font-medium group-hover:translate-x-1 transition-transform">
                Explore chain →
              </div>
            </a>

            <div className="bg-gray-800/50 border border-gray-700 rounded-2xl p-6">
              <div className="text-4xl mb-4">🧱</div>
              <h4 className="text-xl font-bold mb-2">Validators</h4>
              <p className="text-gray-400 text-sm mb-4">
                Run a validator node and secure the Aether network. P2P gossip
                protocol and Turbine-style block propagation.
              </p>
              <div className="text-yellow-400 text-sm font-medium">
                Documentation coming soon
              </div>
            </div>
          </div>
        </div>

        {/* Chain Stats Bar */}
        <div className="border-t border-gray-800 bg-black/20">
          <div className="max-w-6xl mx-auto px-4 py-6">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
              <div>
                <p className="text-xs text-gray-500 uppercase tracking-wider mb-1">
                  Network
                </p>
                <p className="text-lg font-bold text-white">Aether Testnet</p>
              </div>
              <div>
                <p className="text-xs text-gray-500 uppercase tracking-wider mb-1">
                  Consensus
                </p>
                <p className="text-lg font-bold text-white">Proof of History</p>
              </div>
              <div>
                <p className="text-xs text-gray-500 uppercase tracking-wider mb-1">
                  Token
                </p>
                <p className="text-lg font-bold text-white">$ATH</p>
              </div>
              <div>
                <p className="text-xs text-gray-500 uppercase tracking-wider mb-1">
                  Status
                </p>
                <p className="text-lg font-bold text-green-400">🟢 Live</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
