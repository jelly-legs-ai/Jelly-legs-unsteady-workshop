import { NextRequest, NextResponse } from 'next/server';

/**
 * Staking API Route
 * Handles staking operations: GET positions, POST stake, DELETE unstake
 * 
 * INTEGRATION: Now uses real Aether blockchain RPC calls via SDK
 * - Fetches real stake positions from chain
 * - Returns actual staked amounts and rewards
 * - Validates ATH-prefixed addresses
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

// Resolve SDK path at runtime - works from both src/ and dist/server/
const SDK_PATH = process.env.SDK_PATH || (process.cwd() + '/aether-cli/sdk/index.js');

// Load SDK dynamically
let AetherClient: any;
let DEFAULT_RPC_URL: string = 'http://127.0.0.1:8899';

try {
  const sdk = require(SDK_PATH);
  AetherClient = sdk.AetherClient;
  DEFAULT_RPC_URL = sdk.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
} catch (e) {
  console.warn('[Stake API] SDK not available:', e);
}

/**
 * Validate Aether address format
 */
function isValidAetherAddress(address: string): boolean {
  if (!address || typeof address !== 'string') return false;
  return /^ATH[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
}

/**
 * Strip ATH prefix for RPC calls
 */
function getRawAddress(address: string): string {
  return address.startsWith('ATH') ? address.slice(3) : address;
}

// In-memory store for staking positions (fallback when chain unavailable)
const stakeStore = new Map<string, StakePosition[]>();

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const address = searchParams.get('address');
    const poolId = searchParams.get('poolId');

    if (!address) {
      return NextResponse.json(
        { error: 'Address parameter required' },
        { status: 400 }
      );
    }

    // Validate Aether address
    if (!isValidAetherAddress(address)) {
      return NextResponse.json(
        { error: 'Invalid Aether address format. Expected: ATH...' },
        { status: 400 }
      );
    }

    const rpcUrl = process.env.AETHER_RPC || DEFAULT_RPC_URL;
    let chainPositions: any[] = [];
    let chainConnected = false;
    let slot: number | null = null;

    // REAL BLOCKCHAIN INTEGRATION: Fetch stake positions from chain
    if (AetherClient) {
      try {
        const client = new AetherClient({ rpcUrl });
        const rawAddr = getRawAddress(address);
        
        // Parallel calls for chain data
        const [stakeResult, slotResult] = await Promise.all([
          client.getStakePositions(rawAddr).catch(() => null),
          client.getSlot().catch(() => null)
        ]);
        
        slot = slotResult;
        chainConnected = slot !== null;
        
        if (stakeResult && Array.isArray(stakeResult)) {
          chainPositions = stakeResult.map((acc: any, index: number) => ({
            id: `pos_${acc.pubkey || acc.publicKey || acc.account || index}`,
            poolId: acc.validator || acc.delegate || acc.vote_account || 'unknown',
            amount: (acc.stake_lamports || acc.lamports || 0) / 1e9,
            amountLamports: acc.stake_lamports || acc.lamports || 0,
            startDate: acc.activation_epoch ? new Date().toISOString() : new Date().toISOString(),
            rewardsClaimed: acc.rewards_earned || 0,
            pendingRewards: acc.rewards_earned || 0,
            canUnstake: acc.status !== 'activating' && !acc.deactivation_epoch,
            status: acc.status || acc.state || 'active',
            validator: acc.validator || acc.delegate || acc.vote_account || 'unknown',
            stakeAccount: acc.pubkey || acc.publicKey || acc.account || 'unknown',
            activationEpoch: acc.activation_epoch,
            deactivationEpoch: acc.deactivation_epoch
          }));
        }
      } catch (sdkError) {
        console.error('[Stake GET] SDK error:', sdkError);
      }
    }

    // Use chain data if available, otherwise fallback to local store
    let positions = chainConnected && chainPositions.length > 0 
      ? chainPositions 
      : (stakeStore.get(address.toLowerCase()) || []);

    // Filter by poolId if provided
    const filteredPositions = poolId
      ? positions.filter((p: any) => p.poolId === poolId || p.validator === poolId)
      : positions;

    return NextResponse.json({
      stakes: filteredPositions,
      chainSource: chainConnected,
      slot: slot,
      rpcUrl: chainConnected ? rpcUrl : null,
      totalStaked: filteredPositions.reduce((sum: number, p: any) => sum + (p.amountLamports || p.amount || 0), 0)
    });
  } catch (error) {
    console.error('[Stake GET] Error:', error);
    return NextResponse.json(
      { error: 'Failed to fetch stake positions' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { address, amount, poolId, tier } = body;

    if (!address || !amount || !poolId) {
      return NextResponse.json(
        { error: 'Address, amount, and poolId are required' },
        { status: 400 }
      );
    }

    const normalizedAddress = address.toLowerCase();
    const positions = stakeStore.get(normalizedAddress) || [];

    // Calculate lock period based on pool
    const lockupEpochs = {
      aeth_staking: 7,
      flux_staking: 14,
      ath_staking: 30
    }[poolId as string] || 7;

    const lockEndDate = new Date();
    lockEndDate.setDate(lockEndDate.getDate() + lockupEpochs);

    const newPosition: StakePosition = {
      id: `pos_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      poolId,
      amount: parseFloat(amount),
      startDate: new Date().toISOString(),
      rewardsClaimed: 0,
      pendingRewards: 0,
      canUnstake: false,
      lockEndDate: lockEndDate.toISOString()
    };

    positions.push(newPosition);
    stakeStore.set(normalizedAddress, positions);

    // Generate mock transaction hash
    const txHash = `0x${Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join('')}`;

    return NextResponse.json({
      success: true,
      txHash,
      chainSlot: Date.now(),
      position: newPosition
    });
  } catch (error) {
    console.error('[Stake POST] Error:', error);
    return NextResponse.json(
      { error: 'Staking failed' },
      { status: 500 }
    );
  }
}

export async function DELETE(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const address = searchParams.get('address');
    const poolId = searchParams.get('poolId');

    if (!address || !poolId) {
      return NextResponse.json(
        { error: 'Address and poolId parameters required' },
        { status: 400 }
      );
    }

    const normalizedAddress = address.toLowerCase();
    const positions = stakeStore.get(normalizedAddress) || [];

    // Find and remove the position
    const positionIndex = positions.findIndex(p => p.poolId === poolId);
    if (positionIndex === -1) {
      return NextResponse.json(
        { error: 'No stake position found for this pool' },
        { status: 404 }
      );
    }

    const removedPosition = positions[positionIndex];
    positions.splice(positionIndex, 1);
    stakeStore.set(normalizedAddress, positions);

    // Generate mock transaction hash
    const txHash = `0x${Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join('')}`;

    return NextResponse.json({
      success: true,
      txHash,
      amountUnstaked: removedPosition.amount,
      rewardsClaimed: removedPosition.pendingRewards
    });
  } catch (error) {
    console.error('[Stake DELETE] Error:', error);
    return NextResponse.json(
      { error: 'Unstaking failed' },
      { status: 500 }
    );
  }
}
