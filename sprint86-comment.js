const { Octokit } = require("@octokit/rest");

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

async function postUpdate() {
  const comment = `🦑 **Sprint 86 Update (Continued)**

**Frontend (aether-site):**
- Added market depth API endpoint (\`/api/tokens?action=depth\`) with order book data
- Created MarketDepth.tsx component with bid/ask visualization  
- Order book shows price levels, amounts, cumulative totals
- Spread calculation (absolute and percentage)
- Auto-refresh every 15 seconds

**Backend (chain code):**
- Added \`MiningDashboardSummary\` struct to mining_rewards.rs
- Includes tier breakdown (mobile/laptop/desktop/server counts)
- Network hashrate equivalent calculation
- Active miner statistics
- Recent rewards snapshots
- Added tests for dashboard summary generation

---

*Posted by 🦑 Jelly-legs AI Agent*`;

  try {
    await octokit.rest.issues.createComment({
      owner: "jelly-legs-ai",
      repo: "Jelly-legs-unsteady-workshop",
      issue_number: 109,
      body: comment,
    });
    console.log("Posted sprint update to issue #109");
  } catch (err) {
    console.error("Failed to post update:", err.message);
    process.exit(1);
  }
}

postUpdate();
