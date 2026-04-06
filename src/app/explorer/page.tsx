"use client";

import React, { useState, useCallback } from "react";
import Head from "next/head";

/**
 * Chain Explorer Lite
 * Look up addresses, transactions, and block data without leaving the portal.
 * Full RPC integration — no third-party middleware.
 */

type LookupType = "address" | "transaction" | "block";

interface AddressInfo {
  address: string;
  balance: number;
  balanceFormatted: string;
  owner: string | null;
  executable: boolean;
  rentEpoch: number | null;
  slot: number | null;
  exists: boolean;
}

interface TxInfo {
  signature: string;
  slot: number | null;
  blockTime: number | null;
  fee: number | null;
  status: string;
  type: string;
}

interface BlockInfo {
  blockHeight: number | null;
  slot: number | null;
  epoch: number | null;
  blockTime: number | null;
  transactions: number | null;
  blockHash: string | null;
}

export default function ExplorerPage() {
  const [lookupType, setLookupType] = useState<LookupType>("address");
  const [searchInput, setSearchInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const [addressData, setAddressData] = useState<AddressInfo | null>(null);
  const [txData, setTxData] = useState<TxInfo | null>(null);
  const [blockData, setBlockData] = useState<BlockInfo | null>(null);

  const handleLookup = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchInput.trim()) return;

    setIsLoading(true);
    setError("");
    setAddressData(null);
    setTxData(null);
    setBlockData(null);

    try {
      if (lookupType === "address") {
        // Direct RPC call for address lookup (consistent with block lookup pattern)
        const rpcUrl = process.env.AETHER_RPC || "http://127.0.0.1:8899";
        const rawAddress = searchInput.trim().startsWith("ATH")
          ? searchInput.trim().slice(3)
          : searchInput.trim();

        const [accountRes, slotRes] = await Promise.all([
          fetch(rpcUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "getAccountInfo",
              params: [rawAddress, { encoding: "json" }],
            }),
          }),
          fetch(rpcUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "getSlot",
              params: [],
            }),
          }),
        ]);

        const [accountData, slotData] = await Promise.all([
          accountRes.json(),
          slotRes.json(),
        ]);

        if (accountData.error) throw new Error(accountData.error.message || "Account lookup failed");
        if (slotData.error) throw new Error(slotData.error.message || "Slot lookup failed");

        const accountInfo = accountData.result?.value;
        const balanceLamports = accountInfo?.lamports || 0;
        const balanceAeth = balanceLamports / 1e9;

        setAddressData({
          address: searchInput.trim(),
          balance: balanceAeth,
          balanceFormatted: `${balanceAeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 })} ATH`,
          owner: accountInfo?.owner || null,
          executable: accountInfo?.executable || false,
          rentEpoch: accountInfo?.rentEpoch || null,
          slot: slotData.result,
          exists: accountInfo !== null,
        });
      } else if (lookupType === "transaction") {
        // getTransaction RPC via /api/explorer
        const res = await fetch("/api/explorer", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            signature: searchInput.trim(),
          }),
        });
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || "Lookup failed");
        setTxData({
          signature: data.signature || searchInput.trim(),
          slot: data.slot || null,
          blockTime: data.blockTime || null,
          fee: data.fee || null,
          status: data.status || "unknown",
          type: data.type || "transfer",
        });
      } else if (lookupType === "block") {
        // getBlock RPC
        const blockNum = parseInt(searchInput.trim());
        if (isNaN(blockNum) || blockNum < 0) {
          throw new Error("Block height must be a non-negative number");
        }
        const rpcUrl = process.env.AETHER_RPC || "http://127.0.0.1:8899";
        const rpcRes = await fetch(rpcUrl, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            jsonrpc: "2.0",
            id: 1,
            method: "getBlock",
            params: [blockNum, { encoding: "json", maxSupportedTransactionVersion: 0 }],
          }),
        });
        const rpcData = await rpcRes.json();
        if (rpcData.error) throw new Error(rpcData.error.message || "Block lookup failed");
        const result = rpcData.result;
        setBlockData({
          blockHeight: blockNum,
          slot: result?.slot || blockNum,
          epoch: result?.epoch || null,
          blockTime: result?.blockTime || null,
          transactions: result?.transactions?.length || 0,
          blockHash: result?.blockhash || null,
        });
      }
    } catch (err: any) {
      setError(err.message || "Lookup failed");
    } finally {
      setIsLoading(false);
    }
  }, [lookupType, searchInput]);

  return (
    <>
      <Head>
        <title>Chain Explorer - Aether Chain</title>
      </Head>
      <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white">
        <div className="max-w-4xl mx-auto px-4 py-8">
          {/* Header */}
          <div className="text-center mb-8">
            <h1 className="text-4xl font-bold mb-2 bg-gradient-to-r from-red-500 to-red-600 bg-clip-text text-transparent">
              Chain Explorer
            </h1>
            <p className="text-gray-400">
              Look up addresses, transactions, and blocks — direct from the chain
            </p>
          </div>

          {/* Search Form */}
          <div className="bg-gray-800/50 backdrop-blur-sm border border-gray-700 rounded-xl p-6 mb-6">
            {/* Type Selector */}
            <div className="flex gap-2 mb-4">
              {(["address", "transaction", "block"] as LookupType[]).map((type) => (
                <button
                  key={type}
                  onClick={() => {
                    setLookupType(type);
                    setSearchInput("");
                    setAddressData(null);
                    setTxData(null);
                    setBlockData(null);
                    setError("");
                  }}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-all capitalize ${
                    lookupType === type
                      ? "bg-red-600 text-white"
                      : "bg-gray-700 text-gray-300 hover:bg-gray-600"
                  }`}
                >
                  {type}
                </button>
              ))}
            </div>

            <form onSubmit={handleLookup}>
              <div className="flex gap-3">
                <input
                  type="text"
                  value={searchInput}
                  onChange={(e) => setSearchInput(e.target.value)}
                  placeholder={
                    lookupType === "address"
                      ? "ATH..."
                      : lookupType === "transaction"
                      ? "signature or TX hash..."
                      : "block height (number)..."
                  }
                  className="flex-1 px-4 py-3 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:border-red-500 transition-colors"
                />
                <button
                  type="submit"
                  disabled={isLoading || !searchInput.trim()}
                  className="px-6 py-3 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-lg font-semibold transition-all disabled:opacity-50"
                >
                  {isLoading ? "Looking up..." : "Lookup"}
                </button>
              </div>
            </form>

            {error && (
              <div className="mt-4 p-3 bg-red-900/50 border border-red-500 rounded-lg text-red-200 text-sm">
                {error}
              </div>
            )}
          </div>

          {/* Results */}
          {addressData && (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-green-500/30 rounded-xl p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 rounded-full bg-green-500/20 flex items-center justify-center text-green-400 text-xl">
                  👤
                </div>
                <h2 className="text-xl font-semibold text-green-400">Account Found</h2>
              </div>
              <div className="space-y-3">
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Address</span>
                  <span className="font-mono text-sm text-white">{addressData.address}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Balance</span>
                  <span className="text-2xl font-bold text-white">{addressData.balanceFormatted}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Owner</span>
                  <span className="font-mono text-sm text-gray-300">
                    {addressData.owner ? `${addressData.owner.slice(0, 8)}...` : "N/A"}
                  </span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Executable</span>
                  <span className={addressData.executable ? "text-yellow-400" : "text-gray-300"}>
                    {addressData.executable ? "Yes (program)" : "No (account)"}
                  </span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Slot</span>
                  <span className="text-gray-300">{addressData.slot?.toLocaleString() || "N/A"}</span>
                </div>
                <div className="flex justify-between items-center py-2">
                  <span className="text-gray-400">Status</span>
                  <span className="text-green-400">✓ On-chain</span>
                </div>
              </div>
            </div>
          )}

          {txData && (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-blue-500/30 rounded-xl p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center text-blue-400 text-xl">
                  📝
                </div>
                <h2 className="text-xl font-semibold text-blue-400">Transaction</h2>
              </div>
              <div className="space-y-3">
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Signature</span>
                  <span className="font-mono text-sm text-white break-all">{txData.signature}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Type</span>
                  <span className="text-gray-300 capitalize">{txData.type}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Slot</span>
                  <span className="text-gray-300">{txData.slot?.toLocaleString() || "N/A"}</span>
                </div>
                <div className="flex justify-between items-center py-2">
                  <span className="text-gray-400">Status</span>
                  <span className="text-blue-400">{txData.status}</span>
                </div>
              </div>
            </div>
          )}

          {blockData && (
            <div className="bg-gray-800/50 backdrop-blur-sm border border-purple-500/30 rounded-xl p-6">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-400 text-xl">
                  🧱
                </div>
                <h2 className="text-xl font-semibold text-purple-400">Block</h2>
              </div>
              <div className="space-y-3">
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Block Height</span>
                  <span className="text-xl font-bold text-white">{blockData.blockHeight?.toLocaleString()}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Slot</span>
                  <span className="text-gray-300">{blockData.slot?.toLocaleString() || "N/A"}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Epoch</span>
                  <span className="text-gray-300">{blockData.epoch?.toString() || "N/A"}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Block Hash</span>
                  <span className="font-mono text-sm text-gray-300 break-all">
                    {blockData.blockHash || "N/A"}
                  </span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-gray-700">
                  <span className="text-gray-400">Transactions</span>
                  <span className="text-gray-300">{blockData.transactions?.toLocaleString() || 0}</span>
                </div>
                <div className="flex justify-between items-center py-2">
                  <span className="text-gray-400">Block Time</span>
                  <span className="text-gray-300">
                    {blockData.blockTime
                      ? new Date(blockData.blockTime * 1000).toLocaleString()
                      : "N/A"}
                  </span>
                </div>
              </div>
            </div>
          )}

          {/* Empty State */}
          {!isLoading && !addressData && !txData && !blockData && !error && (
            <div className="text-center py-12 text-gray-500">
              <div className="text-4xl mb-4">🔍</div>
              <p>Enter an address, transaction signature, or block height to look it up on-chain</p>
            </div>
          )}
        </div>
      </div>
    </>
  );
}
