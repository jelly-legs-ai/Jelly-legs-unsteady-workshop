const fs = require('fs');
const https = require('https');

const commentFile = process.argv[2] || 'sprint-comment-body.txt';
let commentBody;

try {
  commentBody = fs.readFileSync(commentFile, 'utf8').trim();
  if (!commentBody) {
    console.log('⚠️ Comment file is empty. Exiting.');
    process.exit(0);
  }
} catch (e) {
  console.log('⚠️ Could not read comment file:', e.message);
  process.exit(0);
}

const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
if (!token) {
  console.log('⚠️ No GitHub token found. Set $env:GITHUB_TOKEN or $env:GH_TOKEN');
  process.exit(0);
}

async function postComment(owner, repo, issueNumber, body) {
  const data = JSON.stringify({ body });

  const options = {
    hostname: 'api.github.com',
    path: `/repos/${owner}/${repo}/issues/${issueNumber}/comments`,
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Accept': 'application/vnd.github+json',
      'User-Agent': 'jelly-legs-ai-sprint-bot',
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(data),
    },
  };

  return new Promise((resolve, reject) => {
    const req = https.request(options, (res) => {
      let responseBody = '';
      res.on('data', chunk => responseBody += chunk);
      res.on('end', () => {
        console.log(`GitHub API Response: ${res.statusCode}`);
        if (res.statusCode === 201) {
          console.log('✅ Comment posted successfully!');
          resolve(JSON.parse(responseBody));
        } else {
          console.error(`❌ Failed - Status: ${res.statusCode}`);
          console.error(responseBody.substring(0, 500));
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

async function main() {
  console.log('🦑 Jelly-legs Sprint Comment Poster\n');
  console.log(`📤 Posting sprint update from: ${commentFile}\n`);
  console.log(`Token: ${token.substring(0, 8)}...\n`);

  try {
    await postComment('jelly-legs-ai', 'Jelly-legs-unsteady-workshop', 109, commentBody);
    console.log('\n✅ Done! Sprint update posted.');
  } catch (err) {
    console.log('\n⚠️ Comment posting failed (code was pushed successfully).');
  }
}

main().catch(console.error);