"use client";

import React, { useState, useEffect } from "react";

/**
 * NetworkStatus Component
 * Fetches real-time network stats from the Aether blockchain via /api/network-stats
 */

interface NetworkStats {
  slot: number;
  tps: number;
  epoch: number;
  epochProgress: string;
  blockHeight: number;
  chainVersion: string;
  rpcUrl: string;
  connected: boolean;
}

function StatCard({ label, value, subvalue, loading }: { label: string; value: string; subvalue?: string; loading?: boolean }) {
  if (loading) {
    return (
      <div className="bg-gray-800/40 backdrop-blur-sm border border-gray-700/50 rounded-xl p-4 text-center animate-pulse">
        <div className="h-8 bg-gray-700 rounded mb-2" />
        <div className="h-4 bg-gray-700 rounded w-20 mx-auto" />
      </div>
    );
  }

  return (
    <div className="bg-gray-800/40 backdrop-blur-sm border border-gray-700/50 rounded-xl p-4 text-center">
      <div className="text-2xl font-bold text-white mb-1">{value}</div>
      <div className="text-sm text-gray-400">{label}</div>
      {subvalue && <div className="text-xs text-gray-500 mt-1">{subvalue}</div>}
    </div>
  );
}

export default function NetworkStatus() {
  const [stats, setStats] = useState<NetworkStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchStats() {
      try {
        const res = await fetch("/api/network-stats");
        if (res.ok) {
          const data = await res.json();
          setStats(data);
        } else {
          // Fallback to demo values
          setStats({
            slot: 0,
            tps: 65000,
            epoch: 0,
            epochProgress: "0.0",
            blockHeight: 0,
            chainVersion: "1.0.0",
            rpcUrl: "",
            connected: false,
          });
        }
      } catch {
        setStats({
          slot: 0,
          tps: 65000,
          epoch: 0,
          epochProgress: "0.0",
          blockHeight: 0,
          chainVersion: "1.0.0",
          rpcUrl: "",
          connected: false,
        });
      } finally {
        setLoading(false);
      }
    }

    fetchStats();
    // Refresh every 30 seconds
    const interval = setInterval(fetchStats, 30000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="mb-12">
      {/* Network Status Header */}
      <div className="flex items-center justify-center gap-2 mb-6">
        <div className={`w-2 h-2 rounded-full ${stats?.connected ? 'bg-green-500 animate-pulse' : 'bg-red-500'}`} />
        <span className="text-sm text-gray-400">
          {stats?.connected ? 'Live from chain' : 'Demo mode (chain unavailable)'}
        </span>
        {stats?.connected && (
          <span className="text-[10px] px-1.5 py-0.5 bg-green-500/20 text-green-400 rounded-full border border-green-500/30 ml-1">
            SYNCED
          </span>
        )}
      </div>

      {/* Live Stats Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
        <StatCard 
          label="Current Slot" 
          value={stats?.slot && stats.slot > 0 ? stats.slot.toLocaleString() : '—'} 
        />
        <StatCard 
          label="Throughput" 
          value={stats?.tps && stats.tps > 0 ? `${stats.tps.toLocaleString()} TPS` : '65K+ TPS'}
          subvalue="AI Workloads"
        />
        <StatCard 
          label="Epoch" 
          value={stats?.epoch && stats.epoch > 0 ? stats.epoch.toString() : '—'}
          subvalue={stats?.epochProgress && stats.epochProgress !== '0.0' ? `${stats.epochProgress}% complete` : undefined}
        />
        <StatCard 
          label="Block Height" 
          value={stats?.blockHeight && stats.blockHeight > 0 ? stats.blockHeight.toLocaleString() : '—'} 
        />
      </div>

      {/* Chain Info Footer */}
      {stats?.connected && stats.rpcUrl && (
        <div className="mt-4 text-center text-xs text-gray-600">
          Connected to {stats.rpcUrl} · v{stats.chainVersion}
        </div>
      )}
    </div>
  );
}
