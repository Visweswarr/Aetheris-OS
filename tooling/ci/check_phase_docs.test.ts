#!/usr/bin/env node

/**
 * Test suite for Phase 1 Documentation Guard
 */

import { analyzeChanges, validatePhase1Requirements } from './check_phase_docs';

interface TestCase {
  name: string;
  changedFiles: string[];
  expectPass: boolean;
  description: string;
}

const testCases: TestCase[] = [
  {
    name: 'No changes',
    changedFiles: [],
    expectPass: true,
    description: 'Empty changeset should pass'
  },
  {
    name: 'Non-kernel changes only',
    changedFiles: [
      'README.md',
      'services/wallet/src/main.rs',
      'ui/components/Button.tsx'
    ],
    expectPass: true,
    description: 'Changes outside kernel/ should pass without doc updates'
  },
  {
    name: 'Kernel changes with SPEC update',
    changedFiles: [
      'kernel/src/lib.rs',
      'kernel/src/boot/mod.rs',
      'docs/phase-1/SPEC.md'
    ],
    expectPass: true,
    description: 'Kernel changes with SPEC.md update should pass'
  },
  {
    name: 'Kernel changes with DESIGN update',
    changedFiles: [
      'kernel/src/scheduler/mod.rs',
      'kernel/src/memory/allocator.rs',
      'docs/phase-1/DESIGN.md'
    ],
    expectPass: true,
    description: 'Kernel changes with DESIGN.md update should pass'
  },
  {
    name: 'Kernel changes with both SPEC and DESIGN updates',
    changedFiles: [
      'kernel/src/arch/x86_64.rs',
      'kernel/src/arch/aarch64.rs',
      'docs/phase-1/SPEC.md',
      'docs/phase-1/DESIGN.md'
    ],
    expectPass: true,
    description: 'Kernel changes with both docs updated should pass'
  },
  {
    name: 'Kernel changes without doc updates',
    changedFiles: [
      'kernel/src/panic.rs',
      'kernel/src/log.rs'
    ],
    expectPass: false,
    description: 'Kernel changes without doc updates should fail'
  },
  {
    name: 'Mixed changes - kernel without docs',
    changedFiles: [
      'kernel/src/main.rs',
      'services/hello/src/main.rs',
      'docs/README.md'
    ],
    expectPass: false,
    description: 'Mixed changes including kernel without Phase 1 docs should fail'
  },
  {
    name: 'Documentation updates only',
    changedFiles: [
      'docs/phase-1/SPEC.md',
      'docs/phase-1/DESIGN.md',
      'docs/phase-1/TASKS.md'
    ],
    expectPass: true,
    description: 'Documentation-only changes should pass'
  },
  {
    name: 'Large kernel refactor with docs',
    changedFiles: [
      'kernel/src/lib.rs',
      'kernel/src/boot/mod.rs',
      'kernel/src/boot/uefi_main.rs',
      'kernel/src/arch/x86_64.rs',
      'kernel/src/arch/aarch64.rs',
      'kernel/src/memory/mod.rs',
      'kernel/src/scheduler/mod.rs',
      'kernel/src/ipc/mod.rs',
      'docs/phase-1/SPEC.md',
      'docs/phase-1/DESIGN.md'
    ],
    expectPass: true,
    description: 'Large kernel refactor with comprehensive doc updates should pass'
  },
  {
    name: 'Build system changes affecting kernel',
    changedFiles: [
      'kernel/Cargo.toml',
      'kernel/build.rs',
      'kernel/BUILD'
    ],
    expectPass: false,
    description: 'Build system changes in kernel/ should require doc updates'
  }
];

function runTest(testCase: TestCase): boolean {
  console.log(`\n🧪 Running test: ${testCase.name}`);
  console.log(`   Description: ${testCase.description}`);
  console.log(`   Changed files: ${testCase.changedFiles.length}`);
  
  try {
    const result = analyzeChanges(testCase.changedFiles);
    const actualPass = validatePhase1Requirements(result);
    
    if (actualPass === testCase.expectPass) {
      console.log(`   ✅ PASS - Expected ${testCase.expectPass ? 'pass' : 'fail'}, got ${actualPass ? 'pass' : 'fail'}`);
      return true;
    } else {
      console.log(`   ❌ FAIL - Expected ${testCase.expectPass ? 'pass' : 'fail'}, got ${actualPass ? 'pass' : 'fail'}`);
      return false;
    }
  } catch (error) {
    console.log(`   💥 ERROR - ${error}`);
    return false;
  }
}

function runAllTests(): void {
  console.log('🚀 Starting Phase 1 Documentation Guard Tests');
  console.log(`Running ${testCases.length} test cases...\n`);
  
  let passed = 0;
  let failed = 0;
  
  for (const testCase of testCases) {
    if (runTest(testCase)) {
      passed++;
    } else {
      failed++;
    }
  }
  
  console.log('\n=== Test Results ===');
  console.log(`Total tests: ${testCases.length}`);
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`Success rate: ${((passed / testCases.length) * 100).toFixed(1)}%`);
  
  if (failed === 0) {
    console.log('\n🎉 All tests passed!');
    process.exit(0);
  } else {
    console.log('\n❌ Some tests failed!');
    process.exit(1);
  }
}

// Run tests if executed directly
if (require.main === module) {
  runAllTests();
}

export { runAllTests, testCases };
