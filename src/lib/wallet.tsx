'use client';

import React, { useMemo } from 'react';
import {
  ConnectionProvider,
  WalletProvider as SolanaWalletProvider,
} from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom';
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare';
import { BackpackWalletAdapter } from '@solana/wallet-adapter-backpack';

// Import wallet adapter styles
import '@solana/wallet-adapter-react-ui/styles.css';

const RPC_URL = process.env.NEXT_PUBLIC_AETHER_RPC || 'http://127.0.0.1:8899';

interface WalletProviderProps {
  children: React.ReactNode;
}

export default function WalletProvider({ children }: WalletProviderProps) {
  // Solana-compatible wallets: Phantom, Solflare, and Backpack
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
      new BackpackWalletAdapter(),
    ],
    []
  );

  return (
    <ConnectionProvider endpoint={RPC_URL}>
      <SolanaWalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>{children}</WalletModalProvider>
      </SolanaWalletProvider>
    </ConnectionProvider>
  );
}

export { RPC_URL };
