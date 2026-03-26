const https = require('https');
const http = require('http');

// GitHub API helper to post comment on issue
async function postComment(owner, repo, issueNumber, body) {
  const token = process.env.GITHUB_TOKEN;
  
  if (!token) {
    console.log('⚠️ GITHUB_TOKEN not set. Skipping GitHub comment.');
    console.log('Set it with: set GITHUB_TOKEN=your_token_here');
    return;
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

// Main flow
async function main() {
  console.log('🦑 Jelly-legs Sprint Comment Poster\n');
  console.log('Enter your sprint update (blank line to finish):\n');

  const lines = [];
  const readline = require('readline').createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  const waitForInput = () => {
    return new Promise((resolve) => {
      readline.prompt();
    });
  };

  console.log('📝 Sprint Update:');
  
  for await (const line of readline) {
    if (line.trim() === '' && lines.length > 0) {
      break;
    }
    lines.push(line);
  }

  readline.close();

  const commentBody = lines.join('\n');

  if (!commentBody.trim()) {
    console.log('⚠️ No comment entered. Exiting.');
    return;
  }

  console.log('\n📤 Posting to GitHub issue #109...\n');

  try {
    await postComment('jelly-legs-ai', 'Jelly-legs-unsteady-workshop', 109, commentBody);
    console.log('\n✅ Done! Sprint update posted.');
  } catch (err) {
    console.log('\n⚠️ Comment posting failed (this is OK - code was pushed).');
  }
}

main().catch(console.error);
