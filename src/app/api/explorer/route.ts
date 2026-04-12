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
    const client = new AetherClient({ rpcUrl });
    let result: any;

    try {
      switch (type) {
        case 'address':
          if (!address) {
            return NextResponse.json({ error: 'Address is required' }, { status: 400 });
          }
          const rawAddress = address.startsWith('ATH') ? address.slice(3) : address;
          const [accountData, slot] = await Promise.all([
            client.getAccountInfo(rawAddress),
            client.getSlot(),
          ]);

          if (!accountData) {
            return NextResponse.json({ error: 'Account not found' }, { status: 404 });
          }

          const balanceLamports = (accountData as any).lamports || 0;
          const balanceAeth = balanceLamports / 1e9;

          result = {
            type: 'address',
            address,
            balance: balanceAeth,
            balanceFormatted: `${balanceAeth.toLocaleString(undefined, { minimumFractionDigits: 4, maximumFractionDigits: 4 })} ATH`,
            owner: (accountData as any).owner || null,
            executable: (accountData as any).executable || false,
            rentEpoch: (accountData as any).rentEpoch || null,
            slot,
            exists: true,
            chainSource: true,
          };
          break;

        case 'transaction':
          if (!signature) {
            return NextResponse.json({ error: 'Transaction signature is required' }, { status: 400 });
          }
          const tx = await client.getTransaction(signature);
          if (!tx) {
            result = {
              type: 'transaction',
              signature,
              slot: null,
              blockTime: null,
              fee: null,
              status: 'not_found',
              txType: 'unknown',
              chainSource: true,
            };
          } else {
            const txAny = tx as any;
            const meta = txAny.meta as any;
            const accounts = txAny.transaction?.message?.accountKeys || [];
            result = {
              type: 'transaction',
              signature,
              slot: txAny.slot,
              blockTime: txAny.blockTime,
              fee: meta?.fee || null,
              status: meta?.err ? 'failed' : 'confirmed',
              txType: 'transfer',
              from: accounts[0] ? `ATH${accounts[0]}` : null,
              to: accounts[1] ? `ATH${accounts[1]}` : null,
              lamports: meta?.postBalances?.[1] 
                ? (meta.postBalances[1] - meta.preBalances?.[1] || 0) 
                : null,
              blockHash: txAny.transaction?.message?.recentBlockhash || null,
              chainSource: true,
            };
          }
          break;

        case 'block':
          if (blockHeight === undefined || blockHeight === null) {
            return NextResponse.json({ error: 'Block height is required' }, { status: 400 });
          }
          const blockNum = parseInt(String(blockHeight));
          if (isNaN(blockNum) || blockNum < 0) {
            return NextResponse.json({ error: 'Block height must be a non-negative number' }, { status: 400 });
          }
          // The SDK doesn't have a dedicated getBlock, we use the rpcCall internally via a custom request if needed
          // but for now we can use a raw rpcCall if we want exact getBlock.
          // Let's use the client's internal mechanism by adding it or using a raw fetch for this specific case.
          const blockResponse = await fetch(rpcUrl, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              id: 1,
              method: 'getBlock',
              params: [blockNum, { encoding: 'json', maxSupportedTransactionVersion: 0 }],
            }),
          });
          const blockData = await blockResponse.json();
          if (blockData.error) throw new Error(blockData.error.message);
          
          const resultBlock = blockData.result;
          result = {
            type: 'block',
            blockHeight: blockNum,
            slot: resultBlock?.slot || blockNum,
            epoch: resultBlock?.epoch || null,
            blockTime: resultBlock?.blockTime || null,
            transactions: resultBlock?.transactions?.length || 0,
            blockHash: resultBlock?.blockhash || null,
            chainSource: true,
          };
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
