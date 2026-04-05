const { execSync } = require('child_process');

const comment = `**Sprint 53 Update** 🦑

**Frontend - WhyChooseSection Enhancements:**
- Added visual highlights for AeTHer Chain's key competitive advantages
- Enhanced row styling with gradient backgrounds for highlighted features
- Added subtle glow animation on AeTHer advantage columns
- Better visual differentiation between AeTHer and competitors

**Files Modified:**
- \`src/components/WhyChooseSection.tsx\` - Added aetherHighlight interface and conditional glow styling

Pushed to main: e00f276`;

console.log(comment);

// Post to GitHub
try {
  const issueNumber = 109;
  
  const bodyEncoded = encodeURIComponent(comment);
  const cmd = `gh issue comment ${issueNumber} --body "${bodyEncoded}"`;
  
  execSync(cmd, { cwd: 'C:\\Users\\RM_Ga\\.openclaw\\workspace', encoding: 'utf8' });
  console.log('\n✅ Posted to GitHub issue #109');
} catch (e) {
  console.log('\n⚠️ Could not post to GitHub:', e.message);
}
