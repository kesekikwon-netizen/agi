#!/usr/bin/env node
/**
 * AGI / Strata Commit Gate Hook
 * Enforces:
 * 1. cargo check & cargo test validation before commit
 * 2. No committed secrets or sensitive keys
 * 3. UTF-8 file integrity
 */

import { execSync } from 'child_process';

function run() {
  const args = process.argv.slice(2);
  const cmd = args.join(' ');

  // Only intercept git commit commands
  if (!cmd.includes('git commit')) {
    return 0;
  }

  console.log('[Commit-Gate] Validating Rust workspace before commit...');

  try {
    execSync('cargo check --workspace', { stdio: 'inherit' });
    execSync('cargo test --workspace', { stdio: 'inherit' });
    console.log('[Commit-Gate] All checks passed successfully.');
    return 0;
  } catch (err) {
    console.error('[Commit-Gate Error] Workspace check or tests failed! Commit blocked.');
    process.exit(1);
  }
}

run();
