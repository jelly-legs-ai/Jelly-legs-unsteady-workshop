import { NextRequest, NextResponse } from 'next/server';
import { AetherClient, DEFAULT_RPC_URL } from '@/lib/aether-sdk';

/**
 * Explorer API Route
 * Unified handler for all Chain Explorer lookups:
 * - address: getAccountInfo + getSlot
 * - transaction: getTransaction
 * - block: getBlock
 * 
 * All calls go through server-side RPC — no CORS issues, no RPC URL exposure
 */

const DEFAULT_RPC = DEFAULT_RPC_URL || 'http://127.0.0.1:8899';

/**
 * Make RPC call to Aether RPC
 */
async function rpcCall(rpcUrl: string, method: string, params: any[] = []) {
  const response = await fetch(rpcUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method,
      params,
    }),
  });
  return response.json();
}

/**
 * Handle address lookup via getAccountInfo + getSlot
 */
async function handleAddressLookup(address: string, rpcUrl: string) {
  const rawAddress = address.startsWith('ATH') ? address.slice(3) : address;
  
  const [accountData, slotData] = await Promise.all([
    rpcCall(rpcUrl, 'getAccountInfo', [rawAddress, { encoding: 'json' }]),
    rpcCall(rpcUrl, 'getSlot', []),
  ]);

  if (accountData.error) {
    throw new Error(accountData.error.message || 'Account lookup failed');
  }

  const accountInfo = accountData.result?.value;
  const balanceLamports = accountInfo?.lamports || 0;
  const balanceAeth = balanceLamports / 1e9;

  return {
    type: 'address',
    address,
    balance: balanceAeth,
    balanceFormatted: `${balanceAeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 })} ATH`,
    owner: accountInfo?.owner || null,
    executable: accountInfo?.executable || false,
    rentEpoch: accountInfo?.rentEpoch || null,
    slot: slotData.result || null,
    exists: accountInfo !== null,
    chainSource: slotData.result !== null,
  };
}

/**
 * Handle transaction lookup via getTransaction
 */
async function handleTransactionLookup(signature: string, rpcUrl: string) {
  const data = await rpcCall(rpcUrl, 'getTransaction', [
    signature,
    { encoding: 'json', maxSupportedTransactionVersion: 0 },
  ]);

  if (data.error) {
    throw new Error(data.error.message || 'Transaction lookup failed');
  }

  const result = data.result;

  if (result) {
    const meta = result.meta;
    const transaction = result.transaction;
    const accounts = transaction?.message?.accountKeys || [];
    const fromAccount = accounts[0] || null;
    const toAccount = accounts[1] || null;

    return {
      type: 'transaction',
      signature,
      slot: result.slot,
      blockTime: result.blockTime,
      fee: meta?.fee || null,
      status: meta?.err ? 'failed' : 'confirmed',
      txType: 'transfer',
      from: fromAccount ? `ATH${fromAccount}` : null,
      to: toAccount ? `ATH${toAccount}` : null,
      lamports: meta?.postBalances?.[1] 
        ? (meta.postBalances[1] - meta.preBalances?.[1] || 0) 
        : null,
      blockHash: result.transaction?.message?.recentBlockhash || null,
      chainSource: true,
    };
  }

  return {
    type: 'transaction',
    signature,
    slot: null,
    blockTime: null,
    fee: null,
    status: 'not_found',
    txType: 'unknown',
    chainSource: true,
  };
}

/**
 * Handle block lookup via getBlock
 */
async function handleBlockLookup(blockHeight: number, rpcUrl: string) {
  const data = await rpcCall(rpcUrl, 'getBlock', [
    blockHeight,
    { encoding: 'json', maxSupportedTransactionVersion: 0 },
  ]);

  if (data.error) {
    throw new Error(data.error.message || 'Block lookup failed');
  }

  const result = data.result;

  return {
    type: 'block',
    blockHeight,
    slot: result?.slot || blockHeight,
    epoch: result?.epoch || null,
    blockTime: result?.blockTime || null,
    transactions: result?.transactions?.length || 0,
    blockHash: result?.blockhash || null,
    chainSource: true,
  };
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { type, address, signature, blockHeight } = body;

    if (!type) {
      return NextResponse.json(
        { error: 'Lookup type required (address, transaction, or block)' },
        { status: 400 }
      );
    }

    const rpcUrl = process.env.AETHER_RPC || DEFAULT_RPC;
    let result: any;

    try {
      switch (type) {
        case 'address':
          if (!address) {
            return NextResponse.json(
              { error: 'Address is required' },
              { status: 400 }
            );
          }
          result = await handleAddressLookup(address, rpcUrl);
          break;

        case 'transaction':
          if (!signature) {
            return NextResponse.json(
              { error: 'Transaction signature is required' },
              { status: 400 }
            );
          }
          result = await handleTransactionLookup(signature, rpcUrl);
          break;

        case 'block':
          if (blockHeight === undefined || blockHeight === null) {
            return NextResponse.json(
              { error: 'Block height is required' },
              { status: 400 }
            );
          }
          const blockNum = parseInt(String(blockHeight));
          if (isNaN(blockNum) || blockNum < 0) {
            return NextResponse.json(
              { error: 'Block height must be a non-negative number' },
              { status: 400 }
            );
          }
          result = await handleBlockLookup(blockNum, rpcUrl);
          break;

        default:
          return NextResponse.json(
            { error: 'Invalid lookup type. Must be address, transaction, or block' },
            { status: 400 }
          );
      }
    } catch (rpcError: any) {
      console.error('[Explorer API] RPC error:', rpcError);
      return NextResponse.json(
        { error: rpcError.message || 'Chain lookup failed' },
        { status: 502 }
      );
    }

    return NextResponse.json({
      ...result,
      rpcUrl,
    });
  } catch (error: any) {
    console.error('[Explorer API] Error:', error);
    return NextResponse.json(
      { error: error.message || 'Explorer lookup failed' },
      { status: 500 }
    );
  }
}
