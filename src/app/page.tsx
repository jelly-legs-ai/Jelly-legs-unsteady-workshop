import type { Metadata } from 'next'
import Link from 'next/link'
import NetworkStatus from '../components/NetworkStatus'
import WalletHeroSection from '../components/WalletHeroSection'

export const metadata: Metadata = {
  title: 'Aether Chain — Layer 1 for AI Workloads',
  description: 'Stake, build, and operate on the Aether blockchain. 400ms slot time. 65,000+ TPS. AI Priority Lanes.',
}

export default function HomePage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 text-white">
      {/* Hero Section */}
      <div className="relative overflow-hidden">
        {/* Animated background glow */}
        <div className="absolute inset-0">
          <div className="absolute top-[-20%] left-1/2 -translate-x-1/2 w-[800px] h-[800px] bg-red-500/10 rounded-full blur-[120px]" />
          <div className="absolute bottom-[-10%] right-[-10%] w-[400px] h-[400px] bg-red-600/5 rounded-full blur-[80px]" />
        </div>

        <div className="relative max-w-6xl mx-auto px-4 pt-20 pb-16">
          {/* Logo / Brand */}
          <div className="flex items-center justify-center gap-3 mb-8">
            <div className="w-14 h-14 rounded-xl bg-gradient-to-br from-red-500 to-red-700 flex items-center justify-center text-2xl font-bold shadow-lg shadow-red-500/30">
              🜰
            </div>
            <span className="text-3xl font-bold tracking-tight">Aether</span>
          </div>

          <h1 className="text-5xl md:text-7xl font-bold text-center mb-6 leading-tight">
            <span className="bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent">
              Layer 1 for
            </span>
            <br />
            <span className="bg-gradient-to-r from-red-400 via-red-500 to-red-600 bg-clip-text text-transparent">
              AI Workloads
            </span>
          </h1>

          <p className="text-center text-gray-400 text-lg md:text-xl max-w-2xl mx-auto mb-12 leading-relaxed">
            A high-performance blockchain built for AI operators. 400ms slot time.
            65,000+ TPS. AI Priority Lanes for mission-critical workloads.
          </p>

          {/* Wallet-connected CTA — real Phantom/Solflare/Backpack connect */}
          <WalletHeroSection />

          {/* Live Network Stats */}
          <NetworkStatus />

          {/* Feature Cards */}
          <div className="grid md:grid-cols-3 gap-6 mb-20">
            {[
              {
                icon: '💰',
                title: 'Staking Dashboard',
                description: 'Stake ATH tokens and earn rewards. Monitor your positions, claim rewards, and manage your stake across multiple pools.',
                href: '/staking',
                cta: 'Open Staking',
                color: 'from-green-500/20 to-green-600/10 border-green-500/30',
              },
              {
                icon: '🔍',
                title: 'Chain Explorer',
                description: 'Look up addresses, transactions, and blocks directly on-chain. No third-party middleware — real RPC data.',
                href: '/explorer',
                cta: 'Explore Chain',
                color: 'from-blue-500/20 to-blue-600/10 border-blue-500/30',
              },
              {
                icon: '🤖',
                title: 'AI Operator Portal',
                description: 'AI agents and operators connect wallets, pay premium gas for priority lane access, and monitor their positions.',
                href: '/ai-portal',
                cta: 'AI Dashboard',
                color: 'from-purple-500/20 to-purple-600/10 border-purple-500/30',
              },
            ].map((card) => (
              <Link
                key={card.title}
                href={card.href}
                className={`bg-gradient-to-br ${card.color} rounded-xl p-6 hover:scale-[1.02] transition-transform block border`}
              >
                <div className="text-3xl mb-3">{card.icon}</div>
                <h3 className="text-xl font-semibold text-white mb-2">{card.title}</h3>
                <p className="text-gray-400 text-sm mb-4 leading-relaxed">{card.description}</p>
                <span className="text-sm font-medium text-white/80 hover:text-white transition-colors">
                  {card.cta} →
                </span>
              </Link>
            ))}
          </div>

          {/* Wallets Section */}
          <div className="bg-gray-800/30 border border-gray-700/50 rounded-xl p-8 mb-20">
            <h2 className="text-2xl font-bold text-center mb-2 text-white">Wallet Integration</h2>
            <p className="text-gray-400 text-center mb-8 text-sm">
              Aether is Solana-compatible. Connect with your favorite Solana wallet.
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              {['Phantom', 'Solflare', 'Backpack'].map((wallet) => (
                <div
                  key={wallet}
                  className="px-5 py-3 bg-gray-900/60 border border-gray-700 rounded-lg text-sm text-gray-300 hover:border-gray-500 hover:text-white transition-all cursor-pointer flex items-center gap-2"
                >
                  <span className="text-base">💳</span>
                  <span>{wallet}</span>
                  {wallet === 'Phantom' && <span className="text-xs text-green-400 ml-1">Active</span>}
                </div>
              ))}
            </div>
          </div>

          {/* Validator Tiers */}
          <div className="mb-20">
            <h2 className="text-2xl font-bold text-center mb-8 text-white">Validator Tiers</h2>
            <div className="grid md:grid-cols-3 gap-6">
              {[
                { tier: 'Full', stake: '10,000 ATH', color: 'from-red-500/20 to-red-600/10 border-red-500/40', badge: 'Full Validator' },
                { tier: 'Lite', stake: '1,000 ATH', color: 'from-orange-500/20 to-orange-600/10 border-orange-500/40', badge: 'Lite Validator' },
                { tier: 'Observer', stake: '0 ATH', color: 'from-gray-500/20 to-gray-600/10 border-gray-500/40', badge: 'Observer' },
              ].map((v) => (
                <div key={v.tier} className={`bg-gradient-to-br ${v.color} rounded-xl p-6 border`}>
                  <div className="text-xs uppercase tracking-wider text-gray-400 mb-2">{v.badge}</div>
                  <div className="text-3xl font-bold text-white mb-1">{v.tier}</div>
                  <div className="text-gray-400 text-sm">Min stake: <span className="text-white font-medium">{v.stake}</span></div>
                </div>
              ))}
            </div>
          </div>

          <footer className="text-center text-gray-600 text-sm mt-20 pb-10">
            <p>© 2026 Aether Chain — Powered by AetherFlow Consensus</p>
            <p className="mt-1">AI Priority Lanes · Hybrid PoH + PoS · Tower BFT</p>
          </footer>
        </div>
      </div>
    </div>
  )
}
