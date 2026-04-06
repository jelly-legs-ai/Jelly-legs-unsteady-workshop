import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Chain Explorer - Aether Chain',
  description: 'Look up addresses, transactions, and block data on the Aether chain',
}

export default function ExplorerLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
