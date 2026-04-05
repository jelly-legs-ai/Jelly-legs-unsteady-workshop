// post-sprint-update.js - Post sprint update to GitHub issue #109
const { Octokit } = require("@octokit/rest");

const octokit = new Octokit({ auth: process.env.GITHUB_TOKEN });

const ISSUE_NUMBER = 109;
const OWNER = "jelly-legs-ai";
const REPO = "Jelly-legs-unsteady-workshop";

const sprintUpdate = `## 🦑 Sprint 6 Complete - Frontend Visual Effects

**Location:** \`aether-site/src/components/MobileResponsive.tsx\`

**Added Interactive Effects:**
- **CursorAttraction** - Elements subtly follow cursor within range (150px default)
- **InteractiveGlow** - Radial glow follows mouse position inside element
- **ElasticScrollSnap** - Carousels with elastic drag resistance (0.7 factor)
- **MorphingBlob** - Animated gradient blob with blur for ambient backgrounds

**Changes:**
- 164 lines added
- All components use smooth transitions (150-300ms)
- Fully responsive, mobile-optimized
- Zero dependencies - pure React + Tailwind

**Commit:** \`b7d34ac\` - Pushed to main

---
*Continuous development mode - next sprint incoming*`;

async function postUpdate() {
  try {
    await octokit.rest.issues.createComment({
      owner: OWNER,
      repo: REPO,
      issue_number: ISSUE_NUMBER,
      body: sprintUpdate,
    });
    console.log("✅ Posted sprint update to issue #" + ISSUE_NUMBER);
  } catch (err) {
    console.error("❌ Failed to post update:", err.message);
    process.exit(1);
  }
}

postUpdate();
