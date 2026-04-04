const https = require('https');

const commentBody = `**Sprint 91 Update** — 2026-04-04 12:00 UTC

**Picked up:** Run the 2-node testnet end-to-end via testnet-local.ps1

**What was broken:** The script removed and recreated the testnet directory but never created the \`testnet/genesis/\` subdirectory before calling \`create-genesis\`. So when the binary ran \`std::fs::write(&out_path, json)\` where out_path was \`testnet/genesis/genesis.json\`, the parent directory didn't exist yet — "The system cannot find the path specified" (os error 3).

**Fix:** Added \`New-Item\` to create \`$genesisDir\` before invoking the validator binary, in testnet-local.ps1.

**Result — full 2-node testnet passed all checks:**
- Node 1 (bootstrap) and Node 2 both running
- Node 1 slot advancing: 53, Node 2 slot advancing: 46
- Genesis hash verified on both nodes: \`832DFLVeeE5k4XtXChKvHooAs8FemTu4AUuF7LbSgpmE\`
- Chain ID: aether-testnet-local
- Genesis handshake, peer connection, and slot sync all working

**Commit:** \`7a7decf\` — testnet-local.ps1: create genesis output dir before running create-genesis (fixes path error on fresh run)

**Next:** TCP P2P handshake is working between two separate processes. Next sprint I'll focus on RPC completeness — adding getBlock, getTransaction, and getVoteAccounts endpoints if any are missing, and then validating slot synchronisation when a new node joins.`;

const token = process.env.GITHUB_TOKEN;

if (!token) {
  console.log('GITHUB_TOKEN not set. Skipping.');
  process.exit(0);
}

const postData = JSON.stringify({ body: commentBody });

const options = {
  hostname: 'api.github.com',
  path: '/repos/jelly-legs-ai/Jelly-legs-unsteady-workshop/issues/111/comments',
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Accept': 'application/vnd.github+json',
    'User-Agent': 'jelly-legs-ai',
    'Content-Type': 'application/json'
  }
};

const req = https.request(options, (res) => {
  let body = '';
  res.on('data', chunk => body += chunk);
  res.on('end', () => {
    console.log('Status:', res.statusCode);
    if (res.statusCode === 201) {
      console.log('Comment posted successfully!');
    } else {
      console.error('Response:', body);
    }
  });
});

req.on('error', (e) => {
  console.error('Request error:', e.message);
});

req.write(postData);
req.end();
