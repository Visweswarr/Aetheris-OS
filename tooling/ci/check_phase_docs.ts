#!/usr/bin/env node

/**
 * Phase 1 Documentation Guard Script
 * 
 * Enforces the rule: SPEC → DESIGN → TASKS → CODE/TESTS → DOCS
 * 
 * This script checks if PRs that modify kernel/ files also update
 * Phase 1 documentation (SPEC.md or DESIGN.md).
 * 
 * Exit codes:
 * - 0: Documentation requirements satisfied
 * - 1: Documentation requirements violated
 * - 2: Script error
 */

import { execSync } from 'child_process';
import * as process from 'process';

interface FileChange {
  path: string;
  status: 'added' | 'modified' | 'deleted' | 'renamed';
}

interface CheckResult {
  hasKernelChanges: boolean;
  hasPhaseDocsChanges: boolean;
  kernelFiles: string[];
  phaseDocFiles: string[];
  allChangedFiles: string[];
}

/**
 * Get list of changed files in the current PR
 */
function getChangedFiles(): string[] {
  try {
    // Try to get changed files from Git
    const baseSha = process.env.BASE_SHA || process.env.GITHUB_BASE_REF || 'origin/main';
    const headSha = process.env.HEAD_SHA || 'HEAD';
    
    console.log(`Comparing changes: ${baseSha}...${headSha}`);
    
    // Get list of changed files
    const gitCommand = `git diff --name-only ${baseSha}...${headSha}`;
    console.log(`Running: ${gitCommand}`);
    
    const output = execSync(gitCommand, { 
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe']
    });
    
    const files = output
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0);
    
    console.log(`Found ${files.length} changed files`);
    return files;
    
  } catch (error) {
    console.error('Error getting changed files:', error);
    
    // Fallback: try alternative methods
    try {
      console.log('Trying fallback method with git log...');
      const fallbackCommand = 'git log --name-only --pretty=format: -n 10';
      const fallbackOutput = execSync(fallbackCommand, { encoding: 'utf8' });
      
      const fallbackFiles = fallbackOutput
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.length > 0 && !line.startsWith('commit'));
      
      console.log(`Fallback found ${fallbackFiles.length} files from recent commits`);
      return [...new Set(fallbackFiles)]; // Remove duplicates
      
    } catch (fallbackError) {
      console.error('Fallback method also failed:', fallbackError);
      throw new Error('Unable to determine changed files');
    }
  }
}

/**
 * Analyze changed files for Phase 1 documentation requirements
 */
function analyzeChanges(changedFiles: string[]): CheckResult {
  const kernelFiles: string[] = [];
  const phaseDocFiles: string[] = [];
  
  console.log('\nAnalyzing changed files:');
  
  for (const file of changedFiles) {
    console.log(`  ${file}`);
    
    // Check if file is in kernel/ directory
    if (file.startsWith('kernel/')) {
      kernelFiles.push(file);
    }
    
    // Check if file is Phase 1 documentation
    if (file === 'docs/phase-1/SPEC.md' || file === 'docs/phase-1/DESIGN.md') {
      phaseDocFiles.push(file);
    }
  }
  
  const hasKernelChanges = kernelFiles.length > 0;
  const hasPhaseDocsChanges = phaseDocFiles.length > 0;
  
  console.log(`\nKernel files changed: ${kernelFiles.length}`);
  if (kernelFiles.length > 0) {
    kernelFiles.forEach(file => console.log(`  - ${file}`));
  }
  
  console.log(`\nPhase 1 docs changed: ${phaseDocFiles.length}`);
  if (phaseDocFiles.length > 0) {
    phaseDocFiles.forEach(file => console.log(`  - ${file}`));
  }
  
  return {
    hasKernelChanges,
    hasPhaseDocsChanges,
    kernelFiles,
    phaseDocFiles,
    allChangedFiles: changedFiles
  };
}

/**
 * Validate Phase 1 documentation requirements
 */
function validatePhase1Requirements(result: CheckResult): boolean {
  console.log('\n=== Phase 1 Documentation Guard ===');
  
  if (!result.hasKernelChanges) {
    console.log('✅ No kernel changes detected - documentation rule not applicable');
    return true;
  }
  
  console.log('🔍 Kernel changes detected - checking documentation requirements...');
  
  if (result.hasPhaseDocsChanges) {
    console.log('✅ Phase 1 documentation updated alongside kernel changes');
    console.log('\nUpdated documentation files:');
    result.phaseDocFiles.forEach(file => console.log(`  ✓ ${file}`));
    return true;
  }
  
  // Violation detected
  console.log('\n❌ PHASE 1 RULE VIOLATION');
  console.log('\nRule: SPEC → DESIGN → TASKS → CODE/TESTS → DOCS');
  console.log('\nThis PR modifies kernel code but does not update Phase 1 documentation.');
  
  console.log('\nKernel files modified:');
  result.kernelFiles.forEach(file => console.log(`  🔧 ${file}`));
  
  console.log('\nRequired actions:');
  console.log('  📝 Update docs/phase-1/SPEC.md if requirements changed');
  console.log('  🏗️  Update docs/phase-1/DESIGN.md if architecture changed');
  console.log('  📋 Ensure documentation reflects your code changes');
  
  console.log('\nWhy this rule exists:');
  console.log('  • Maintains architectural coherence');
  console.log('  • Ensures design decisions are documented');
  console.log('  • Prevents implementation drift from specification');
  console.log('  • Helps reviewers understand the complete change');
  
  console.log('\nFor more information:');
  console.log('  📖 See: docs/phase-1/README.md');
  console.log('  🔗 Phase 1 Specification: docs/phase-1/SPEC.md');
  console.log('  🎯 Phase 1 Design: docs/phase-1/DESIGN.md');
  
  return false;
}

/**
 * Print summary information
 */
function printSummary(result: CheckResult, passed: boolean): void {
  console.log('\n=== Summary ===');
  console.log(`Total files changed: ${result.allChangedFiles.length}`);
  console.log(`Kernel files changed: ${result.kernelFiles.length}`);
  console.log(`Phase 1 docs changed: ${result.phaseDocFiles.length}`);
  console.log(`Rule compliance: ${passed ? '✅ PASS' : '❌ FAIL'}`);
  
  if (!passed) {
    console.log('\n💡 Quick fix:');
    console.log('   1. Review your kernel changes');
    console.log('   2. Update docs/phase-1/SPEC.md if requirements changed');
    console.log('   3. Update docs/phase-1/DESIGN.md if architecture changed');
    console.log('   4. Commit and push the documentation updates');
  }
}

/**
 * Main execution function
 */
function main(): void {
  try {
    console.log('🚀 Phase 1 Documentation Guard - Starting validation...');
    console.log(`Working directory: ${process.cwd()}`);
    console.log(`Node version: ${process.version}`);
    
    // Check if we're in a Git repository
    try {
      execSync('git rev-parse --git-dir', { stdio: 'pipe' });
    } catch {
      console.error('❌ Not in a Git repository');
      process.exit(2);
    }
    
    // Get and analyze changed files
    const changedFiles = getChangedFiles();
    
    if (changedFiles.length === 0) {
      console.log('ℹ️  No files changed - nothing to validate');
      process.exit(0);
    }
    
    const result = analyzeChanges(changedFiles);
    const passed = validatePhase1Requirements(result);
    
    printSummary(result, passed);
    
    // Exit with appropriate code
    process.exit(passed ? 0 : 1);
    
  } catch (error) {
    console.error('\n💥 Script error:', error);
    
    if (error instanceof Error) {
      console.error('Error message:', error.message);
      if (error.stack) {
        console.error('Stack trace:', error.stack);
      }
    }
    
    console.error('\n🔧 Debugging information:');
    console.error(`  Environment: ${process.env.NODE_ENV || 'unknown'}`);
    console.error(`  GitHub Actions: ${process.env.GITHUB_ACTIONS || 'false'}`);
    console.error(`  PR Number: ${process.env.PR_NUMBER || 'unknown'}`);
    console.error(`  Base SHA: ${process.env.BASE_SHA || 'unknown'}`);
    console.error(`  Head SHA: ${process.env.HEAD_SHA || 'unknown'}`);
    
    process.exit(2);
  }
}

// Execute if run directly
if (require.main === module) {
  main();
}

export { main, getChangedFiles, analyzeChanges, validatePhase1Requirements };
