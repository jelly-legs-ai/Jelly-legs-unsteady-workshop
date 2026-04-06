import { NextResponse } from 'next/server';
import { AetherClient, DEFAULT_RPC_URL } from '@/lib/aether-sdk';

/**
 * Network Stats API Route
 * Returns real-time network statistics from the Aether blockchain
 */

export async function GET() {
  const rpcUrl = process.env.AETHER_RPC || DEFAULT_RPC_URL;
  const client = new AetherClient({ rpcUrl, timeoutMs: 5000 });

  try {
    const [slot, tps, epochInfo, version, blockHeight] = await Promise.all([
      client.getSlot().catch(() => 0),
      client.getTPS().catch(() => 0),
      client.getEpochInfo().catch(() => null),
      client.getVersion().catch(() => ({ 'solana-core': 'unknown' })),
      client.getBlockHeight().catch(() => 0),
    ]);

    let epoch = 0;
    let slotIndex = 0;
    let slotsInEpoch = 432000;
    let epochProgress = '0.0';

    if (epochInfo && typeof epochInfo === 'object' && 'epoch' in epochInfo) {
      epoch = (epochInfo as any).epoch || 0;
      slotIndex = (epochInfo as any).slotIndex || 0;
      slotsInEpoch = (epochInfo as any).slotsInEpoch || 432000;
      if (slotsInEpoch > 0) {
        epochProgress = ((slotIndex / slotsInEpoch) * 100).toFixed(1);
      }
    }

    const connected = slot > 0;

    return NextResponse.json({
      slot: slot || 0,
      tps: tps || 0,
      epoch,
      epochProgress,
      blockHeight: blockHeight || 0,
      chainVersion: (version as any)['solana-core'] || '1.0.0',
      rpcUrl,
      connected,
    });
  } catch (error: any) {
    console.error('[Network Stats API] Error:', error);
    return NextResponse.json(
      { 
        error: 'Failed to fetch network stats',
        slot: 0,
        tps: 0,
        epoch: 0,
        epochProgress: '0.0',
        blockHeight: 0,
        chainVersion: 'unknown',
        rpcUrl,
        connected: false,
      },
      { status: 200 } // Return 200 with fallback data
    );
  }
}
