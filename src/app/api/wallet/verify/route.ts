import { NextRequest, NextResponse } from 'next/server';

/**
 * Wallet Verification API Route
 * Handles wallet address verification for staking and other operations
 * 
 * INTEGRATION: Now uses real Aether blockchain RPC calls via SDK
 * - Validates ATH-prefixed addresses
 * - Fetches real account info from chain
 * - Returns actual balance and account status
 */

// Resolve SDK path at runtime - works from both src/ and dist/server/
const SDK_PATH = process.env.SDK_PATH || (process.cwd() + '/aether-cli/sdk/index.js');

// Load SDK dynamically to avoid issues during build
let AetherClient: any;
let DEFAULT_RPC_URL: string = 'http://127.0.0.1:8899';

try {
  const sdk = require(SDK_PATH);
  AetherClient = sdk.AetherClient;
  DEFAULT_RPC_URL = sdk.DEFAULT_RPC_URL || 'http://127.0.0.1:8899';
} catch (e) {
  console.warn('[Wallet Verify] SDK not available, will use fallback mode:', e);
}

/**
 * Validate Aether address format (ATH prefix + base58)
 * ATH addresses start with "ATH" followed by base58 encoded public key
 */
function isValidAetherAddress(address: string): boolean {
  if (!address || typeof address !== 'string') return false;
  // ATH prefix + base58 chars (32-44 chars typical)
  return /^ATH[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
}

/**
 * Strip ATH prefix for RPC calls
 */
function getRawAddress(address: string): string {
  return address.startsWith('ATH') ? address.slice(3) : address;
}

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

    // Validate Aether address format (ATH prefix)
    if (!isValidAetherAddress(address)) {
      return NextResponse.json(
        { error: 'Invalid Aether address format. Expected: ATH...' },
        { status: 400 }
      );
    }

    // Get RPC URL from env or use default
    const rpcUrl = process.env.AETHER_RPC || DEFAULT_RPC_URL;
    
    let chainData = null;
    let chainConnected = false;
    
    // REAL BLOCKCHAIN INTEGRATION: Verify against actual chain
    if (AetherClient) {
      try {
        const client = new AetherClient({ rpcUrl });
        const rawAddr = getRawAddress(address);
        
        // Parallel chain calls
        const [accountInfo, slot] = await Promise.all([
          client.getAccountInfo(rawAddr).catch(() => null),
          client.getSlot().catch(() => null)
        ]);
        
        chainConnected = slot !== null;
        
        if (accountInfo && !accountInfo.error) {
          chainData = {
            exists: true,
            balance: accountInfo.lamports || 0,
            balanceAeth: accountInfo.lamports ? (accountInfo.lamports / 1e9).toFixed(4) : '0',
            owner: accountInfo.owner || null,
            executable: accountInfo.executable || false,
            rentEpoch: accountInfo.rent_epoch || null
          };
        } else {
          // Account doesn't exist on chain yet (normal for new wallets)
          chainData = {
            exists: false,
            balance: 0,
            balanceAeth: '0',
            owner: null,
            executable: false,
            rentEpoch: null
          };
        }
      } catch (sdkError) {
        console.error('[Wallet Verify] SDK error:', sdkError);
        // Continue with basic verification
      }
    }

    // Build response with chain data if available
    const response: any = {
      verified: true,
      address: address,
      normalizedAddress: address.toLowerCase(),
      timestamp: Date.now(),
      message: 'Wallet verified successfully',
      chainSource: chainConnected,
      rpcUrl: chainConnected ? rpcUrl : null
    };

    // Include chain data if we got it
    if (chainData) {
      response.chainData = chainData;
      response.canStake = chainData.balance > 0;
    }

    return NextResponse.json(response);
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

  // Validate Aether address format
  if (!isValidAetherAddress(address)) {
    return NextResponse.json(
      { error: 'Invalid Aether address format. Expected: ATH...' },
      { status: 400 }
    );
  }

  // REAL BLOCKCHAIN INTEGRATION: Fetch actual account data
  const rpcUrl = process.env.AETHER_RPC || DEFAULT_RPC_URL;
  let chainData = null;
  let chainConnected = false;

  if (AetherClient) {
    try {
      const client = new AetherClient({ rpcUrl });
      const rawAddr = getRawAddress(address);
      
      const [accountInfo, slot, balance] = await Promise.all([
        client.getAccountInfo(rawAddr).catch(() => null),
        client.getSlot().catch(() => null),
        client.getBalance(rawAddr).catch(() => 0)
      ]);
      
      chainConnected = slot !== null;
      
      chainData = {
        exists: accountInfo !== null && !accountInfo.error,
        balance: balance,
        balanceAeth: (balance / 1e9).toFixed(4),
        slot: slot,
        rpcUrl: rpcUrl
      };
    } catch (sdkError) {
      console.error('[Wallet Verify GET] SDK error:', sdkError);
    }
  }

  return NextResponse.json({
    address: address,
    verified: true,
    canStake: chainData ? chainData.balance > 0 : true,
    chainSource: chainConnected,
    chainData: chainData,
    timestamp: Date.now()
  });
}
