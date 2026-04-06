import { NextRequest, NextResponse } from 'next/server';

/**
 * Staking API Route
 * Handles staking operations: GET positions, POST stake, DELETE unstake
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

// In-memory store for staking positions (replace with database in production)
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

    const positions = stakeStore.get(address.toLowerCase()) || [];

    // Filter by poolId if provided
    const filteredPositions = poolId
      ? positions.filter(p => p.poolId === poolId)
      : positions;

    return NextResponse.json({
      stakes: filteredPositions,
      chainSource: false // Indicates we're using local storage, not chain
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
