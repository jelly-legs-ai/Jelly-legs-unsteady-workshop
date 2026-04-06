'use client';

import WalletProvider from '@/lib/wallet';
import Nav from '@/components/Nav';

export default function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WalletProvider>
      <Nav />
      {children}
    </WalletProvider>
  );
}
