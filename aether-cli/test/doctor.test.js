#!/usr/bin/env node
/**
 * Test suite for aether-cli doctor command
 */

const { checkCPU, checkMemory, checkDisk, checkNetwork, checkFirewall } = require('../commands/doctor');

console.log('Running aether-cli doctor tests...\n');

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    const result = fn();
    console.log(`✅ ${name}`);
    passed++;
    return result;
  } catch (error) {
    console.log(`❌ ${name}: ${error.message}`);
    failed++;
    return null;
  }
}

// Test CPU check
test('checkCPU returns valid structure', () => {
  const cpu = checkCPU();
  if (!cpu.section) throw new Error('Missing section');
  if (!('passed' in cpu)) throw new Error('Missing passed');
  if (!cpu.message) throw new Error('Missing message');
  console.log(`   CPU: ${cpu.physicalCores} cores, ${cpu.model}`);
  return cpu;
});

// Test Memory check
test('checkMemory returns valid structure', () => {
  const mem = checkMemory();
  if (!mem.section) throw new Error('Missing section');
  if (!('passed' in mem)) throw new Error('Missing passed');
  if (!mem.message) throw new Error('Missing message');
  console.log(`   Memory: ${mem.total} total, ${mem.available} available`);
  return mem;
});

// Test Disk check
test('checkDisk returns valid structure', () => {
  const disk = checkDisk();
  if (!disk.section) throw new Error('Missing section');
  if (!('passed' in disk)) throw new Error('Missing passed');
  if (!disk.message) throw new Error('Missing message');
  console.log(`   Disk: ${disk.total} total, ${disk.free} free`);
  return disk;
});

// Test Network check
test('checkNetwork returns valid structure', () => {
  const net = checkNetwork();
  if (!net.section) throw new Error('Missing section');
  if (!('passed' in net)) throw new Error('Missing passed');
  console.log(`   Network: ${net.publicIP}`);
  return net;
});

// Test Firewall check
test('checkFirewall returns valid structure', () => {
  const fw = checkFirewall();
  if (!fw.section) throw new Error('Missing section');
  if (!('passed' in fw)) throw new Error('Missing passed');
  console.log(`   Firewall: P2P=${fw.p2p}, RPC=${fw.rpc}, SSH=${fw.ssh}`);
  return fw;
});

// Summary
console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
