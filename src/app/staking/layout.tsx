import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Staking - Aether Chain',
  description: 'Stake ATH tokens to earn rewards and secure the network',
}

export default function StakingLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
