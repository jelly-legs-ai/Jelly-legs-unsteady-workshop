#!/usr/bin/env node
/**
 * Memory Summarizer & Audit Tool
 * 
 * Commands:
 * - audit: Scan memory files and report context concerns
 * - summarize: Create daily summary from memory files
 * - cleanup: Remove outdated or redundant entries
 */

const fs = require('fs');
const path = require('path');

const MEMORY_DIR = path.join(__dirname);
const MEMORY_FILE = path.join(__dirname, '..', 'MEMORY.md');

function getTodayDate() {
  return new Date().toISOString().split('T')[0];
}

function readMemoryFile(filePath) {
  try {
    return fs.readFileSync(filePath, 'utf-8');
  } catch (err) {
    return null;
  }
}

function listMemoryFiles() {
  try {
    const files = fs.readdirSync(MEMORY_DIR)
      .filter(f => f.endsWith('.md') && f !== 'summarizer.js')
      .map(f => ({
        name: f,
        path: path.join(MEMORY_DIR, f),
        date: f.match(/^(\d{4}-\d{2}-\d{2})\.md$/)?.[1]
      }))
      .filter(f => f.date);
    
    return files.sort((a, b) => b.date.localeCompare(a.date));
  } catch (err) {
    console.error('Error listing memory files:', err.message);
    return [];
  }
}

function auditMemory() {
  console.log('🔍 Memory Audit Report');
  console.log('='.repeat(50));
  console.log(`Date: ${new Date().toISOString()}`);
  console.log('');
  
  const concerns = [];
  const memoryFiles = listMemoryFiles();
  
  // Check 1: Memory directory exists
  console.log('✅ Memory directory exists');
  
  // Check 2: Today's memory file
  const today = getTodayDate();
  const todayFile = memoryFiles.find(f => f.date === today);
  if (todayFile) {
    console.log(`✅ Today's memory file exists: ${todayFile.name}`);
  } else {
    console.log(`⚠️  Today's memory file missing: ${today}.md`);
    concerns.push('Missing today\'s memory file');
  }
  
  // Check 3: MEMORY.md exists
  const memoryMd = readMemoryFile(MEMORY_FILE);
  if (memoryMd) {
    console.log('✅ MEMORY.md exists (long-term memory)');
    console.log(`   Size: ${memoryMd.length} characters`);
  } else {
    console.log('⚠️  MEMORY.md missing (create for long-term memories)');
    concerns.push('MEMORY.md not created yet');
  }
  
  // Check 4: Recent memory files (last 7 days)
  const recentFiles = memoryFiles.slice(0, 7);
  console.log(`\n📁 Recent memory files (${recentFiles.length}):`);
  recentFiles.forEach(f => {
    const content = readMemoryFile(f.path);
    const lines = content ? content.split('\n').length : 0;
    console.log(`   ${f.date}: ${lines} lines`);
  });
  
  // Check 5: Memory file age gaps
  if (memoryFiles.length > 1) {
    const dates = memoryFiles.map(f => f.date).sort();
    const gaps = [];
    for (let i = 1; i < dates.length; i++) {
      const prev = new Date(dates[i-1]);
      const curr = new Date(dates[i]);
      const diffDays = Math.floor((prev - curr) / (1000 * 60 * 60 * 24));
      if (diffDays > 3) {
        gaps.push(`${dates[i]} → ${dates[i-1]} (${diffDays} days)`);
      }
    }
    if (gaps.length > 0) {
      console.log('\n⚠️  Gaps in memory logging (>3 days):');
      gaps.forEach(g => console.log(`   ${g}`));
      concerns.push('Gaps in daily memory logging');
    } else {
      console.log('\n✅ Consistent daily memory logging');
    }
  }
  
  // Check 6: File size anomalies
  const largeFiles = memoryFiles.filter(f => {
    const content = readMemoryFile(f.path);
    return content && content.length > 50000;
  });
  if (largeFiles.length > 0) {
    console.log('\n⚠️  Large memory files (>50KB):');
    largeFiles.forEach(f => console.log(`   ${f.name}`));
    concerns.push('Some memory files may need summarization');
  }
  
  // Summary
  console.log('\n' + '='.repeat(50));
  if (concerns.length === 0) {
    console.log('✅ No context concerns found');
  } else {
    console.log(`⚠️  Found ${concerns.length} concern(s):`);
    concerns.forEach((c, i) => console.log(`   ${i+1}. ${c}`));
  }
  
  return concerns.length === 0;
}

function summarize() {
  console.log('📝 Memory Summarizer');
  console.log('='.repeat(50));
  // Placeholder for summarize logic
  console.log('Summarize command not yet implemented');
}

function cleanup() {
  console.log('🧹 Memory Cleanup');
  console.log('='.repeat(50));
  // Placeholder for cleanup logic
  console.log('Cleanup command not yet implemented');
}

// Main
const command = process.argv[2] || 'audit';

switch (command) {
  case 'audit':
    auditMemory();
    break;
  case 'summarize':
    summarize();
    break;
  case 'cleanup':
    cleanup();
    break;
  default:
    console.error(`Unknown command: ${command}`);
    console.log('Usage: node summarizer.js [audit|summarize|cleanup]');
    process.exit(1);
}
