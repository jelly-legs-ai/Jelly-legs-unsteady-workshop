import { NextRequest, NextResponse } from 'next/server';

/**
 * Claim Rewards API Route
 * Handles reward claims for staked positions
 */

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { address, poolId } = body;

    if (!address) {
      return NextResponse.json(
        { error: 'Address is required' },
        { status: 400 }
      );
    }

    // Generate mock transaction hash
    const txHash = `0x${Array.from({length: 64}, () => Math.floor(Math.random() * 16).toString(16)).join('')}`;

    // Mock amount claimed
    const amountClaimed = Math.random() * 100 + 10;

    return NextResponse.json({
      success: true,
      txHash,
      amountClaimed,
      message: 'Rewards claimed successfully'
    });
  } catch (error) {
    console.error('[Claim] Error:', error);
    return NextResponse.json(
      { error: 'Claim failed' },
      { status: 500 }
    );
  }
}
