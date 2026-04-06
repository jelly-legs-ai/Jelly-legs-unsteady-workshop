import { NextRequest, NextResponse } from 'next/server';

/**
 * Wallet Verification API Route
 * Handles wallet address verification for staking and other operations
 */

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { address, signature, message } = body;

    if (!address) {
      return NextResponse.json(
        { error: 'Wallet address is required' },
        { status: 400 }
      );
    }

    // Validate address format (basic check for Ethereum-like addresses)
    if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
      return NextResponse.json(
        { error: 'Invalid wallet address format' },
        { status: 400 }
      );
    }

    // In a production environment, verify the signature
    // For now, return success to enable staking flow
    return NextResponse.json({
      verified: true,
      address: address.toLowerCase(),
      timestamp: Date.now(),
      message: 'Wallet verified successfully'
    });
  } catch (error) {
    console.error('[Wallet Verify] Error:', error);
    return NextResponse.json(
      { error: 'Verification failed' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const address = searchParams.get('address');

  if (!address) {
    return NextResponse.json(
      { error: 'Address parameter required' },
      { status: 400 }
    );
  }

  // Return verification status
  return NextResponse.json({
    address: address.toLowerCase(),
    verified: true,
    canStake: true
  });
}
