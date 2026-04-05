// Script to update dashboard-state.json with enhanced network stats
const fs = require('fs');

const dashboardPath = './data/dashboard-state.json';
const data = JSON.parse(fs.readFileSync(dashboardPath, 'utf-8'));

// Add/Update networkStats section
data.networkStats = {
  totalNodes: 12848,
  activeValidators: 2847,
  totalStake: "2.4B AETH",
  tps: 4521,
  blockTime: "0.4s",
  uptime: "99.98%",
  gasPrice: "12 gwei",
  epoch: 48291,
  nextRewardDistribution: "2h 34m",
  bridgeVolume24h: "14.2M FLUX",
  totalBridgeVolume: "892M FLUX",
  activeAgents: 12047,
  registeredAgents: 15892,
  totalRewardsDistributed: "4.2M FLUX",
  stakingAPR: "12.5%",
  lastUpdated: new Date().toISOString()
};

// Add chainInfo section
data.chainInfo = {
  chainId: "aether-mainnet-1",
  protocolVersion: "2.1.0",
  consensusProtocol: "PoH + Proof of Stake",
  blockReward: "8.5 AETH",
  epochLength: 3600,
  miningRewardPerEpoch: "2,400 FLUX",
  maxTPS: 10000,
  currentShards: 4,
  targetShards: 16
};

fs.writeFileSync(dashboardPath, JSON.stringify(data, null, 2));
console.log('✅ Dashboard state updated with network stats and chain info');
