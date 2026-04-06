import { NextRequest, NextResponse } from 'next/server';

/**
 * Explorer API Route
 * Handles transaction lookups for the Chain Explorer Lite
 * 
 * Uses direct RPC calls to getTransaction for signature lookups
 */

const SDK_PATH = process.env.SDK_PATH || '../../../../aether-cli/sdk/index.js';
const DEFAULT_RPC_URL = 'http://127.0.0.1:8899';

let AetherClient: any;

try {
  const sdk = require(SDK_PATH);
  AetherClient = sdk.AetherClient;
} catch (e) {
  console.warn('[Explorer API] SDK not available:', e);
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { signature } = body;

    if (!signature) {
      return NextResponse.json(
        { error: 'Transaction signature required' },
        { status: 400 }
      );
    }

    const rpcUrl = process.env.AETHER_RPC || DEFAULT_RPC_URL;
    let txData: any = null;
    let chainConnected = false;

    // Direct RPC call for transaction lookup
    try {
      const response = await fetch(rpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'getTransaction',
          params: [
            signature,
            { encoding: 'json', maxSupportedTransactionVersion: 0 }
          ]
        })
      });

      const data = await response.json();
      
      if (data.error) {
        throw new Error(data.error.message || 'Transaction lookup failed');
      }

      chainConnected = true;
      const result = data.result;

      if (result) {
        const meta = result.meta;
        const transaction = result.transaction;
        
        // Extract accounts involved
        const accounts = transaction?.message?.accountKeys || [];
        const fromAccount = accounts[0] || null;
        const toAccount = accounts[1] || null;

        txData = {
          signature: signature,
          slot: result.slot,
          blockTime: result.blockTime,
          fee: meta?.fee || null,
          status: meta?.err ? 'failed' : 'confirmed',
          type: 'transfer',
          from: fromAccount ? `ATH${fromAccount}` : null,
          to: toAccount ? `ATH${toAccount}` : null,
          lamports: meta?.postBalances?.[1] ? 
            (meta.postBalances[1] - meta.preBalances?.[1] || 0) : null,
          blockHash: result.transaction?.message?.recentBlockhash || null,
        };
      } else {
        txData = {
          signature: signature,
          slot: null,
          blockTime: null,
          fee: null,
          status: 'not_found',
          type: 'unknown',
        };
      }
    } catch (rpcError: any) {
      console.error('[Explorer API] RPC error:', rpcError);
      
      // Return mock data so UI isn't broken
      txData = {
        signature: signature,
        slot: null,
        blockTime: null,
        fee: null,
        status: 'lookup_failed',
        type: 'unknown',
        error: rpcError.message || 'Chain lookup failed',
      };
    }

    return NextResponse.json({
      ...txData,
      chainSource: chainConnected,
      rpcUrl: chainConnected ? rpcUrl : null,
    });
  } catch (error: any) {
    console.error('[Explorer API] Error:', error);
    return NextResponse.json(
      { error: error.message || 'Transaction lookup failed' },
      { status: 500 }
    );
  }
}
