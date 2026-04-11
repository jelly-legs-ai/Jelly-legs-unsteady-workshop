'use client';

import React from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import Link from 'next/link';

/**
 * WalletHeroSection
 * Client component for wallet connection in the homepage hero.
 * Shows connected wallet status or connect button.
 * Used in the main landing page CTA area.
 */
export default function WalletHeroSection() {
  const { connected, publicKey, connecting } = useWallet();
  const { connection } = useConnection();

  const shortAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : null;

  return (
    <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-16">
      {connected && publicKey ? (
        <>
          {/* Connected state — show address + quick actions */}
          <div className="flex items-center gap-3 px-5 py-3 bg-green-500/15 border border-green-500/40 rounded-xl">
            <div className="w-2.5 h-2.5 bg-green-400 rounded-full animate-pulse" />
            <span className="text-green-400 font-mono font-medium text-sm">
              {shortAddress}
            </span>
          </div>
          <Link
            href="/staking"
            className="px-8 py-4 bg-gradient-to-r from-red-600 to-red-700 hover:from-red-500 hover:to-red-600 rounded-xl font-semibold text-lg transition-all shadow-lg shadow-red-500/30 hover:shadow-red-500/50 text-center"
          >
            Stake ATH
          </Link>
          <Link
            href="/ai-portal"
            className="px-8 py-4 bg-purple-600/20 hover:bg-purple-600/30 border border-purple-500/40 hover:border-purple-400/60 rounded-xl font-semibold text-lg transition-all text-center"
          >
            AI Portal
          </Link>
        </>
      ) : (
        <>
          {/* Not connected — show connect + explore */}
          <WalletMultiButton className="!bg-gradient-to-r !from-green-600 !to-green-700 !hover:from-green-500 !hover:to-green-600 !text-white !font-semibold !px-8 !py-4 !rounded-xl !text-lg !transition-all !shadow-lg !shadow-green-500/30 !border-0" />
          <Link
            href="/explorer"
            className="px-8 py-4 bg-gray-800 hover:bg-gray-700 border border-gray-700 hover:border-gray-600 rounded-xl font-semibold text-lg transition-all text-center"
          >
            Chain Explorer
          </Link>
        </>
      )}
    </div>
  );
}
