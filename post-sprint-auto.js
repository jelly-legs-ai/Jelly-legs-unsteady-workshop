const https = require('https');
const fs = require('fs');
const path = require('path');

// GitHub API helper to post comment on issue
async function postComment(owner, repo, issueNumber, body) {
  const token = process.env.GITHUB_TOKEN;
  
  if (!token) {
    console.log('⚠️ GITHUB_TOKEN not set. Skipping GitHub comment.');
    console.log('Set it with: set GITHUB_TOKEN=your_token_here');
    return null;
  }

  const data = JSON.stringify({ body });

  const options = {
    hostname: 'api.github.com',
    path: `/repos/${owner}/${repo}/issues/${issueNumber}/comments`,
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Accept': 'application/vnd.github+json',
      'User-Agent': 'jelly-legs-ai',
      'Content-Type': 'application/json',
      'Content-Length': data.length,
    },
  };

  return new Promise((resolve, reject) => {
    const req = https.request(options, (res) => {
      let responseBody = '';
      res.on('data', chunk => responseBody += chunk);
      res.on('end', () => {
        if (res.statusCode === 201) {
          console.log('✅ Comment posted successfully!');
          resolve(JSON.parse(responseBody));
        } else {
          console.error(`❌ Failed to post comment. Status: ${res.statusCode}`);
          console.error(responseBody);
          reject(new Error(`HTTP ${res.statusCode}`));
        }
      });
    });

    req.on('error', (e) => {
      console.error(`❌ Request error: ${e.message}`);
      reject(e);
    });

    req.write(data);
    req.end();
  });
}

// Main flow - reads from sprint-comment.txt
async function main() {
  console.log('🦑 Jelly-legs Auto Sprint Comment Poster\n');

  const commentPath = path.join(__dirname, 'sprint-comment.txt');
  
  if (!fs.existsSync(commentPath)) {
    console.log('⚠️ sprint-comment.txt not found. Exiting.');
    return;
  }

  const commentBody = fs.readFileSync(commentPath, 'utf-8');

  if (!commentBody.trim()) {
    console.log('⚠️ Comment file is empty. Exiting.');
    return;
  }

  console.log('📝 Sprint Update loaded from sprint-comment.txt\n');
  console.log('📤 Posting to GitHub issue #109...\n');

  try {
    await postComment('jelly-legs-ai', 'Jelly-legs-unsteady-workshop', 109, commentBody);
    console.log('\n✅ Done! Sprint update posted.');
  } catch (err) {
    console.log('\n⚠️ Comment posting failed (GitHub account suspension?).');
    console.log('   Code changes are committed locally and ready to push when access is restored.');
  }
}

main().catch(console.error);
