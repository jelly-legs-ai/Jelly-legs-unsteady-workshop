"use client";

import React, { useState, useEffect, useCallback } from "react";
import Head from "next/head";
import { useWallet } from '@solana/wallet-adapter-react';
import { useWalletModal } from '@solana/wallet-adapter-react-ui';

/**
 * Staking Page - REAL WALLET INTEGRATION
 * 
 * FIX: Now uses the real Solana wallet adapter from the nav instead of
 * asking users to manually type wallet addresses. The connected wallet
 * from Phantom, Solflare, or Backpack is used automatically.
 * 
 * Wiring:
 * - /api/stake (GET positions, POST stake, DELETE unstake)
 * - /api/claim (POST claim rewards, GET pending rewards)
 * - /api/wallet/verify (POST verify wallet)
 */

interface StakePosition {
  id: string;
  poolId: string;
  amount: number;
  startDate: string;
  rewardsClaimed: number;
  pendingRewards: number;
  canUnstake: boolean;
  lockEndDate?: string;
}

interface WalletInfo {
  address: string;
  verified: boolean;
  canStake: boolean;
  balance: number;
  balanceAeth: string;
}

interface PoolStat {
  poolId: string;
  name: string;
  apy: number;
  tvl: number;
  tvlLamports: number;
  activeAccounts: number;
  avgStakeSize: number;
  lastUpdatedSlot: number;
  chainConnected: boolean;
  trend: 'up' | 'down' | 'stable';
  color: string;
  colorAccent: string;
  icon: string;
  minStake: number;
  lockDays: number;
}

const POOL_CONFIGS: Record<string, PoolStat> = {
  aeth_staking:  { poolId: 'aeth_staking', name: 'AETH Staking',    apy: 8.5,  tvl: 0, tvlLamports: 0, activeAccounts: 0, avgStakeSize: 0, lastUpdatedSlot: 0, chainConnected: false, trend: 'stable', color: 'from-green-500/20 to-green-600/10 border-green-500/30',   colorAccent: 'text-green-400',   icon: '💰', minStake: 10,   lockDays: 7  },
  flux_staking:  { poolId: 'flux_staking',  name: 'FLUX Staking',     apy: 12.0, tvl: 0, tvlLamports: 0, activeAccounts: 0, avgStakeSize: 0, lastUpdatedSlot: 0, chainConnected: false, trend: 'stable', color: 'from-orange-500/20 to-orange-600/10 border-orange-500/30', colorAccent: 'text-orange-400',  icon: '⚡', minStake: 100,  lockDays: 14 },
  ath_staking:   { poolId: 'ath_staking',   name: 'ATH Governance',   apy: 15.5, tvl: 0, tvlLamports: 0, activeAccounts: 0, avgStakeSize: 0, lastUpdatedSlot: 0, chainConnected: false, trend: 'stable', color: 'from-red-500/20 to-red-600/10 border-red-500/30',          colorAccent: 'text-red-400',     icon: '🏛️', minStake: 1000, lockDays: 30 },
};

export default function StakingPage() {
  // REAL WALLET: use the adapter connected to Phantom/Solflare/Backpack
  const { connected, publicKey, connecting } = useWallet();
  const { setVisible } = useWalletModal();

  // Wallet State (derived from real adapter)
  const walletAddress = publicKey ? publicKey.toBase58() : '';
  const [walletInfo, setWalletInfo] = useState<WalletInfo | null>(null);
  const [isVerifying, setIsVerifying] = useState(false);

  // Staking State
  const [positions, setPositions] = useState<StakePosition[]>([]);
  const [isLoadingPositions, setIsLoadingPositions] = useState(false);
  const [poolStats, setPoolStats] = useState<PoolStat[]>([]);
  const [isLoadingPoolStats, setIsLoadingPoolStats] = useState(true);

  // Action States
  const [stakeAmount, setStakeAmount] = useState<string>("");
  const [selectedPool, setSelectedPool] = useState<string>("aeth_staking");
  const [isStaking, setIsStaking] = useState(false);
  const [isUnstaking, setIsUnstaking] = useState<string | null>(null);
  const [isClaiming, setIsClaiming] = useState<string | null>(null);

  // UI State
  const [error, setError] = useState<string>("");
  const [success, setSuccess] = useState<string>("");

  /**
   * INTEGRATION: Verify wallet via /api/wallet/verify using REAL address
   */
  const verifyWallet = useCallback(async () => {
    if (!walletAddress) return;

    setIsVerifying(true);
    setError("");
    setSuccess("");

    try {
      const response = await fetch("/api/wallet/verify", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ address: walletAddress }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || "Verification failed");
      }

      const data = await response.json();
      setWalletInfo({
        address: walletAddress,
        verified: data.verified,
        canStake: data.canStake || false,
        balance: data.chainData?.balance || 0,
        balanceAeth: data.chainData?.balanceAeth || "0",
      });
      setSuccess("Wallet verified successfully");
    } catch (err: any) {
      setError(err.message || "Wallet verification failed");
      setWalletInfo(null);
    } finally {
      setIsVerifying(false);
    }
  }, [walletAddress]);

  /**
   * Fetch live pool statistics from /api/pool-stats
   */
  const fetchPoolStats = useCallback(async () => {
    setIsLoadingPoolStats(true);
    try {
      const res = await fetch("/api/pool-stats");
      if (res.ok) {
        const data = await res.json();
        if (data.pools && Array.isArray(data.pools)) {
          setPoolStats(data.pools.map((p: any) => ({
            ...POOL_CONFIGS[p.poolId] || POOL_CONFIGS.aeth_staking,
            ...p,
          })));
        }
      }
    } catch {
      // Keep empty on failure
    } finally {
      setIsLoadingPoolStats(false);
    }
  }, []);

  /**
   * Auto-verify when a real wallet connects
   */
  useEffect(() => {
    if (connected && publicKey && !walletInfo?.verified) {
      verifyWallet();
    }
    // If disconnected, clear wallet info
    if (!connected) {
      setWalletInfo(null);
      setPositions([]);
    }
  }, [connected, publicKey, walletInfo?.verified, verifyWallet]);

  /**
   * INTEGRATION: Fetch stake positions from /api/stake
   */
  const fetchPositions = useCallback(async () => {
    if (!walletInfo?.verified || !walletInfo.address) return;

    setIsLoadingPositions(true);
    try {
      const response = await fetch(
        `/api/stake?address=${encodeURIComponent(walletInfo.address)}`
      );
      if (!response.ok) throw new Error("Failed to fetch positions");
      const data = await response.json();
      setPositions(data.stakes || []);
    } catch (err) {
      console.error("Error fetching positions:", err);
      setError("Failed to load stake positions");
    } finally {
      setIsLoadingPositions(false);
    }
  }, [walletInfo?.address, walletInfo?.verified]);

  /**
   * INTEGRATION: Stake via POST /api/stake
   */
  const handleStake = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!walletInfo?.verified || !walletInfo.canStake) {
      setError("Please verify your wallet first");
      return;
    }

    const amount = parseFloat(stakeAmount);
    if (isNaN(amount) || amount <= 0) {
      setError("Please enter a valid amount");
      return;
    }

    const pool = Object.values(POOL_CONFIGS).find((p) => p.poolId === selectedPool);
    if (pool && amount < pool.minStake) {
      setError(`Minimum stake for ${pool.name} is ${pool.minStake} ATH`);
      return;
    }

    setIsStaking(true);
    setError("");
    setSuccess("");

    try {
      const response = await fetch("/api/stake", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          address: walletInfo.address,
          amount: amount,
          poolId: selectedPool,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || "Staking failed");
      }

      const data = await response.json();
      setSuccess(`Successfully staked ${amount} ATH in ${pool?.name}! TX: ${data.txHash?.slice(0, 16)}...`);
      setStakeAmount("");
      
      // Refresh positions after successful stake
      await fetchPositions();
    } catch (err: any) {
      setError(err.message || "Staking transaction failed");
    } finally {
      setIsStaking(false);
    }
  };

  /**
   * INTEGRATION: Claim rewards via POST /api/claim
   */
  const handleClaimRewards = async (poolId: string) => {
    if (!walletInfo?.verified) {
      setError("Please verify your wallet first");
      return;
    }

    setIsClaiming(poolId);
    setError("");
    setSuccess("");

    try {
      const response = await fetch("/api/claim", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          address: walletInfo.address,
          poolId: poolId,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || "Claim failed");
      }

      const data = await response.json();
      setSuccess(
        `Claimed ${data.amountClaimed?.toFixed(4) || "0"} ATH rewards! TX: ${data.txHash?.slice(0, 16)}...`
      );
      
      // Refresh positions after claim
      await fetchPositions();
    } catch (err: any) {
      setError(err.message || "Claim failed");
    } finally {
      setIsClaiming(null);
    }
  };

  /**
   * INTEGRATION: Unstake via DELETE /api/stake
   */
  const handleUnstake = async (poolId: string) => {
    if (!walletInfo?.verified) {
      setError("Please verify your wallet first");
      return;
    }

    setIsUnstaking(poolId);
    setError("");
    setSuccess("");

    try {
      const response = await fetch(
        `/api/stake?address=${encodeURIComponent(walletInfo.address)}&poolId=${encodeURIComponent(poolId)}`,
        { method: "DELETE" }
      );

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || "Unstake failed");
      }

      const data = await response.json();
      setSuccess(
        `Unstaked ${data.amountUnstaked?.toFixed(4) || "0"} ATH! TX: ${data.txHash?.slice(0, 16)}...`
      );
      
      // Refresh positions after unstake
      await fetchPositions();
    } catch (err: any) {
      setError(err.message || "Unstake failed");
    } finally {
      setIsUnstaking(null);
    }
  };

  // Load positions when wallet is verified
  useEffect(() => {
    if (walletInfo?.verified) {
      fetchPositions();
    }
  }, [walletInfo?.verified, fetchPositions]);

  // Fetch live pool stats on mount and every 30s
  useEffect(() => {
    fetchPoolStats();
    const interval = setInterval(fetchPoolStats, 30000);
    return () => clearInterval(interval);
  }, [fetchPoolStats]);

  // Auto-clear messages
  useEffect(() => {
    if (error || success) {
      const timer = setTimeout(() => {
        setError("");
        setSuccess("");
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error, success]);

  const totalStaked = positions.reduce((sum, p) => sum + p.amount, 0);
  const totalPendingRewards = positions.reduce(
    (sum, p) => sum + p.pendingRewards,
    0
  );

  return (
    <>
      <Head>
        <title>Staking - Aether Chain</title>
      </Head>
      <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white">
        <div className="max-w-6xl mx-auto px-4 py-8">
          <h1 className="text-4xl font-bold text-center mb-2 bg-gradient-to-r from-red-500 to-red-600 bg-clip-text text-transparent">
            Staking
          </h1>
          <p className="text-center text-gray-400 mb-8">
            Stake ATH tokens to earn rewards and secure the network
          </p>

          {/* Live Pool Stats Dashboard */}
          <div className="mb-8">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold text-white flex items-center gap-2">
                <span>📊</span> Live Pool Stats
              </h2>
              <div className="flex items-center gap-2 text-xs text-gray-500">
                {poolStats[0]?.chainConnected ? (
                  <>
                    <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                    <span>Live from chain · Slot {poolStats[0]?.lastUpdatedSlot?.toLocaleString() || '—'}</span>
                  </>
                ) : (
                  <>
                    <div className="w-2 h-2 bg-yellow-400 rounded-full animate-pulse" />
                    <span>Demo mode · pool stats unavailable</span>
                  </>
                )}
              </div>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {(poolStats.length > 0 ? poolStats : Object.values(POOL_CONFIGS)).map((pool) => {
                const trendArrow = pool.trend === 'up' ? '↑' : pool.trend === 'down' ? '↓' : '→';
                const trendColor = pool.trend === 'up' ? 'text-green-400' : pool.trend === 'down' ? 'text-red-400' : 'text-gray-400';
                return (
                  <div
                    key={pool.poolId}
                    className={`bg-gradient-to-br ${pool.color} rounded-xl p-5 border`}
                  >
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex items-center gap-2">
                        <span className="text-2xl">{pool.icon}</span>
                        <div>
                          <div className="font-semibold text-white">{pool.name}</div>
                          <div className="text-xs text-gray-400">Min: {pool.minStake} ATH · Lock: {pool.lockDays}d</div>
                        </div>
                      </div>
                      <span className={`text-xs font-medium ${trendColor}`}>{trendArrow}</span>
                    </div>
                    <div className="space-y-2">
                      <div className="flex justify-between items-center">
                        <span className="text-sm text-gray-400">APY</span>
                        <span className={`text-xl font-bold ${pool.colorAccent}`}>
                          {isLoadingPoolStats && poolStats.length === 0 ? '—' : `${pool.apy.toFixed(1)}%`}
                        </span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-sm text-gray-400">TVL</span>
                        <span className="text-white font-medium font-mono text-sm">
                          {isLoadingPoolStats && poolStats.length === 0 ? '—' :
                            pool.tvl > 0 ? `${pool.tvl.toLocaleString(undefined, { maximumFractionDigits: 0 })} ATH` : '—'}
                        </span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-sm text-gray-400">Active Stakers</span>
                        <span className="text-white font-medium text-sm">
                          {isLoadingPoolStats && poolStats.length === 0 ? '—' :
                            pool.activeAccounts > 0 ? pool.activeAccounts.toLocaleString() : '—'}
                        </span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Error/Success Messages */}
          {error && (
            <div className="mb-4 p-4 bg-red-900/50 border border-red-500 rounded-lg text-red-200">
              {error}
            </div>
          )}
          {success && (
            <div className="mb-4 p-4 bg-green-900/50 border border-green-500 rounded-lg text-green-200">
              {success}
            </div>
          )}

          {/* Wallet Connection Section */}
          {!connected ? (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-red-500/30 rounded-xl p-6 mb-8">
              <h2 className="text-xl font-semibold mb-4 text-red-400">
                Connect Your Wallet
              </h2>
              <p className="text-gray-400 text-sm mb-4">
                Connect your Phantom, Solflare, or Backpack wallet to stake ATH and earn rewards.
              </p>
              <button
                onClick={() => setVisible(true)}
                disabled={connecting}
                className="px-6 py-3 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-lg font-semibold transition-all disabled:opacity-50"
              >
                {connecting ? "Connecting..." : "Connect Wallet"}
              </button>
            </div>
          ) : !walletInfo?.verified ? (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-red-500/30 rounded-xl p-6 mb-8">
              <h2 className="text-xl font-semibold mb-4 text-red-400">
                Verifying Wallet...
              </h2>
              <div className="flex items-center gap-2">
                <div className="w-4 h-4 border-2 border-red-500 border-t-transparent rounded-full animate-spin" />
                <span className="text-gray-400">Checking balance on chain</span>
              </div>
            </div>
          ) : (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-green-500/30 rounded-xl p-6 mb-8">
              <div className="flex justify-between items-center">
                <div>
                  <h2 className="text-xl font-semibold text-green-400">
                    ✅ Wallet Connected
                  </h2>
                  <p className="text-gray-400 font-mono text-sm mt-1">
                    {walletInfo.address}
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-gray-400 text-sm">Balance</p>
                  <p className="text-2xl font-bold text-white">
                    {walletInfo.balanceAeth} ATH
                  </p>
                </div>
              </div>
              <div className="mt-4 pt-4 border-t border-gray-700 flex gap-8">
                <div>
                  <p className="text-gray-400 text-sm">Total Staked</p>
                  <p className="text-xl font-semibold text-red-400">
                    {totalStaked.toFixed(4)} ATH
                  </p>
                </div>
                <div>
                  <p className="text-gray-400 text-sm">Pending Rewards</p>
                  <p className="text-xl font-semibold text-green-400">
                    {totalPendingRewards.toFixed(4)} ATH
                  </p>
                </div>
              </div>
            </div>
          )}

          {walletInfo?.verified && (
            <>
              {/* Stake Form */}
              <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6 mb-8">
                <h2 className="text-xl font-semibold mb-4 text-red-400">
                  Stake ATH
                </h2>
                <form onSubmit={handleStake} className="space-y-4">
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm text-gray-400 mb-2">
                        Select Pool
                      </label>
                      <select
                        value={selectedPool}
                        onChange={(e) => setSelectedPool(e.target.value)}
                        className="w-full px-4 py-3 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:border-red-500 transition-colors"
                      >
                        {Object.values(POOL_CONFIGS).map((pool) => (
                          <option key={pool.poolId} value={pool.poolId}>
                            {pool.name} - {pool.apy}% APY (min: {pool.minStake} ATH, {pool.lockDays} days)
                          </option>
                        ))}
                      </select>
                    </div>
                    <div>
                      <label className="block text-sm text-gray-400 mb-2">
                        Amount (ATH)
                      </label>
                      <input
                        type="number"
                        value={stakeAmount}
                        onChange={(e) => setStakeAmount(e.target.value)}
                        placeholder="Enter amount to stake"
                        min="0"
                        step="0.01"
                        className="w-full px-4 py-3 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:border-red-500 transition-colors"
                      />
                    </div>
                  </div>
                  <button
                    type="submit"
                    disabled={isStaking || !stakeAmount}
                    className="w-full py-3 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-lg font-semibold transition-all disabled:opacity-50"
                  >
                    {isStaking ? "Staking..." : "Stake ATH"}
                  </button>
                </form>
              </div>

              {/* Active Positions */}
              <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6">
                <h2 className="text-xl font-semibold mb-4 text-red-400">
                  Your Stake Positions
                </h2>
                {isLoadingPositions ? (
                  <div className="text-center py-8 text-gray-400">
                    Loading positions...
                  </div>
                ) : positions.length === 0 ? (
                  <div className="text-center py-8 text-gray-500">
                    No active stake positions. Start staking to earn rewards!
                  </div>
                ) : (
                  <div className="space-y-4">
                    {positions.map((position) => (
                      <div
                        key={position.id}
                        className="bg-gray-900/50 border border-gray-700 rounded-lg p-4"
                      >
                        <div className="flex justify-between items-start mb-3">
                          <div>
                            <h3 className="font-semibold text-white">
                              {Object.values(POOL_CONFIGS).find((p) => p.poolId === position.poolId)?.name ||
                                position.poolId}
                            </h3>
                            <p className="text-sm text-gray-400">
                              Staked: {new Date(position.startDate).toLocaleDateString()}
                            </p>
                          </div>
                          <div className="text-right">
                            <p className="text-2xl font-bold text-white">
                              {position.amount.toFixed(4)} ATH
                            </p>
                          </div>
                        </div>
                        <div className="grid grid-cols-2 gap-4 mb-4">
                          <div>
                            <p className="text-sm text-gray-400">Pending Rewards</p>
                            <p className="text-lg font-semibold text-green-400">
                              {position.pendingRewards.toFixed(4)} ATH
                            </p>
                          </div>
                          <div>
                            <p className="text-sm text-gray-400">Can Unstake</p>
                            <p
                              className={`text-lg font-semibold ${
                                position.canUnstake
                                  ? "text-green-400"
                                  : "text-yellow-400"
                              }`}
                            >
                              {position.canUnstake ? "Yes" : "Locked"}
                              {position.lockEndDate && (
                                <span className="text-sm text-gray-500 ml-2">
                                  (until{" "}
                                  {new Date(position.lockEndDate).toLocaleDateString()})
                                </span>
                              )}
                            </p>
                          </div>
                        </div>
                        <div className="flex gap-3">
                          <button
                            onClick={() => handleClaimRewards(position.poolId)}
                            disabled={
                              isClaiming === position.poolId ||
                              position.pendingRewards <= 0
                            }
                            className="flex-1 py-2 bg-gradient-to-r from-green-600 to-green-700 hover:from-green-500 hover:to-green-600 rounded-lg font-medium transition-all disabled:opacity-50 text-sm"
                          >
                            {isClaiming === position.poolId
                              ? "Claiming..."
                              : "Claim Rewards"}
                          </button>
                          <button
                            onClick={() => handleUnstake(position.poolId)}
                            disabled={
                              isUnstaking === position.poolId || !position.canUnstake
                            }
                            className="flex-1 py-2 bg-gradient-to-r from-gray-600 to-gray-700 hover:from-gray-500 hover:to-gray-600 rounded-lg font-medium transition-all disabled:opacity-50 text-sm"
                          >
                            {isUnstaking === position.poolId
                              ? "Unstaking..."
                              : "Unstake"}
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </>
  );
}
