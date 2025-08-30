/**
 * Tests for Changelog Generator
 * 
 * Comprehensive test suite covering conventional commit parsing, version bumping,
 * changelog generation, and CI integration scenarios.
 */

import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { ChangelogGenerator } from '../gen.js';

// Test utilities
class TestGitRepo {
  private tempDir: string;
  private originalCwd: string;

  constructor() {
    this.tempDir = '';
    this.originalCwd = process.cwd();
  }

  async setup(): Promise<void> {
    this.tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'changelog-test-'));
    process.chdir(this.tempDir);

    // Initialize git repo
    execSync('git init', { stdio: 'pipe' });
    execSync('git config user.name "Test User"', { stdio: 'pipe' });
    execSync('git config user.email "test@example.com"', { stdio: 'pipe' });

    // Create initial commit
    fs.writeFileSync('README.md', '# Test Repository\n');
    execSync('git add README.md', { stdio: 'pipe' });
    execSync('git commit -m "Initial commit"', { stdio: 'pipe' });
    
    // Create package.json
    fs.writeFileSync('package.json', JSON.stringify({
      name: 'test-repo',
      version: '1.0.0',
    }, null, 2));
    execSync('git add package.json', { stdio: 'pipe' });
    execSync('git commit -m "chore: add package.json"', { stdio: 'pipe' });
  }

  addCommit(message: string, files: Record<string, string> = {}): void {
    // Add files if provided
    for (const [filename, content] of Object.entries(files)) {
      const dir = path.dirname(filename);
      if (dir !== '.' && !fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      fs.writeFileSync(filename, content);
      execSync(`git add "${filename}"`, { stdio: 'pipe' });
    }

    execSync(`git commit -m "${message}"`, { stdio: 'pipe' });
  }

  createTag(tag: string): void {
    execSync(`git tag ${tag}`, { stdio: 'pipe' });
  }

  cleanup(): void {
    process.chdir(this.originalCwd);
    if (this.tempDir && fs.existsSync(this.tempDir)) {
      fs.rmSync(this.tempDir, { recursive: true, force: true });
    }
  }
}

// Test data
const testCommits = {
  feature: 'feat(ui): add new dashboard component\n\nAdd responsive dashboard with charts and metrics display.',
  fix: 'fix(api): resolve authentication timeout issue\n\nFixes #123\n\nThe authentication service was timing out after 30 seconds.\nIncreased timeout to 60 seconds and added retry logic.',
  breaking: 'feat(core)!: redesign plugin architecture\n\nBREAKING CHANGE: Plugin interface has changed.\nOld plugins need to be updated to use the new API.',
  chore: 'chore(deps): update dependencies\n\nUpdate various npm packages to latest versions.',
  docs: 'docs(readme): update installation instructions\n\nAdd Docker installation steps and troubleshooting guide.',
  nonConventional: 'Update readme file with new information',
};

describe('Changelog Generator', () => {
  let testRepo: TestGitRepo;
  let generator: ChangelogGenerator;

  beforeEach(async () => {
    testRepo = new TestGitRepo();
    await testRepo.setup();
    generator = new ChangelogGenerator();
  });

  afterEach(() => {
    testRepo.cleanup();
  });

  describe('Conventional Commit Parsing', () => {
    test('should parse valid conventional commits', () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.addCommit(testCommits.breaking);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);

      expect(conventionalCommits).toHaveLength(4); // Including package.json commit

      const featureCommit = conventionalCommits.find(c => c.type === 'feat' && c.scope === 'ui');
      expect(featureCommit).toBeDefined();
      expect(featureCommit?.description).toBe('add new dashboard component');
      expect(featureCommit?.scope).toBe('ui');
      expect(featureCommit?.breaking).toBe(false);

      const fixCommit = conventionalCommits.find(c => c.type === 'fix' && c.scope === 'api');
      expect(fixCommit).toBeDefined();
      expect(fixCommit?.description).toBe('resolve authentication timeout issue');
      expect(fixCommit?.body).toContain('Fixes #123');

      const breakingCommit = conventionalCommits.find(c => c.breaking === true);
      expect(breakingCommit).toBeDefined();
      expect(breakingCommit?.type).toBe('feat');
      expect(breakingCommit?.description).toBe('redesign plugin architecture');
    });

    test('should handle non-conventional commits gracefully', () => {
      testRepo.addCommit(testCommits.nonConventional);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);

      const nonConventionalCommit = conventionalCommits.find(
        c => c.description === testCommits.nonConventional
      );
      expect(nonConventionalCommit).toBeDefined();
      expect(nonConventionalCommit?.type).toBe('chore'); // Default categorization
    });

    test('should extract PR numbers from commit messages', () => {
      testRepo.addCommit('feat: add new feature (#42)\n\nImplements feature XYZ as requested in #42.');

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);

      const prCommit = conventionalCommits.find(c => c.pr === 42);
      expect(prCommit).toBeDefined();
      expect(prCommit?.description).toBe('add new feature (#42)');
    });
  });

  describe('Version Bump Calculation', () => {
    test('should calculate major version bump for breaking changes', () => {
      testRepo.addCommit(testCommits.breaking);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);
      const versionBump = generator.calculateVersionBump(conventionalCommits);

      expect(versionBump.type).toBe('major');
      expect(versionBump.current).toBe('1.0.0');
      expect(versionBump.next).toBe('2.0.0');
      expect(versionBump.reason).toContain('Breaking changes');
    });

    test('should calculate minor version bump for features', () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.docs);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);
      const versionBump = generator.calculateVersionBump(conventionalCommits);

      expect(versionBump.type).toBe('minor');
      expect(versionBump.current).toBe('1.0.0');
      expect(versionBump.next).toBe('1.1.0');
      expect(versionBump.reason).toContain('New features');
    });

    test('should calculate patch version bump for fixes', () => {
      testRepo.addCommit(testCommits.fix);
      testRepo.addCommit(testCommits.chore);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);
      const versionBump = generator.calculateVersionBump(conventionalCommits);

      expect(versionBump.type).toBe('patch');
      expect(versionBump.current).toBe('1.0.0');
      expect(versionBump.next).toBe('1.0.1');
      expect(versionBump.reason).toContain('Bug fixes');
    });

    test('should prioritize breaking changes over features', () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.addCommit(testCommits.breaking);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);
      const versionBump = generator.calculateVersionBump(conventionalCommits);

      expect(versionBump.type).toBe('major');
      expect(versionBump.next).toBe('2.0.0');
    });
  });

  describe('Changelog Generation', () => {
    test('should generate changelog for single release', async () => {
      // Add various types of commits
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.addCommit(testCommits.docs);
      testRepo.addCommit(testCommits.chore);

      const releaseInfo = await generator.generateChangelog(undefined, 'HEAD', 'CHANGELOG.md');

      expect(releaseInfo.version).toBe('1.1.0'); // Minor bump for feature
      expect(releaseInfo.commits.length).toBeGreaterThan(0);
      expect(releaseInfo.sections.length).toBeGreaterThan(0);

      // Check if changelog file was created
      expect(fs.existsSync('CHANGELOG.md')).toBe(true);
      
      const changelogContent = fs.readFileSync('CHANGELOG.md', 'utf8');
      expect(changelogContent).toContain('## [1.1.0]');
      expect(changelogContent).toContain('🚀 Features');
      expect(changelogContent).toContain('🐛 Bug Fixes');
      expect(changelogContent).toContain('📚 Documentation');
      expect(changelogContent).toContain('add new dashboard component');
      expect(changelogContent).toContain('resolve authentication timeout issue');
    });

    test('should handle breaking changes section', async () => {
      testRepo.addCommit(testCommits.breaking);
      testRepo.addCommit(testCommits.feature);

      const releaseInfo = await generator.generateChangelog(undefined, 'HEAD');

      expect(releaseInfo.breakingChanges.length).toBe(1);
      expect(releaseInfo.stats.breakingCount).toBe(1);

      const changelogContent = generator['formatChangelog'](releaseInfo);
      expect(changelogContent).toContain('⚠️ BREAKING CHANGES');
      expect(changelogContent).toContain('Plugin interface has changed');
    });

    test('should generate full changelog with multiple tags', async () => {
      // Create first release
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.createTag('v1.0.0');

      // Create second release
      testRepo.addCommit(testCommits.breaking);
      testRepo.addCommit('feat(api): add GraphQL support');
      testRepo.createTag('v2.0.0');

      // Add unreleased changes
      testRepo.addCommit('fix(ui): resolve button styling issue');

      await generator.generateFullChangelog('FULL_CHANGELOG.md');

      const changelogContent = fs.readFileSync('FULL_CHANGELOG.md', 'utf8');
      
      expect(changelogContent).toContain('## [Unreleased]');
      expect(changelogContent).toContain('## [v2.0.0]');
      expect(changelogContent).toContain('## [v1.0.0]');
      expect(changelogContent).toContain('resolve button styling issue');
      expect(changelogContent).toContain('add GraphQL support');
    });

    test('should include contributor information', async () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);

      const releaseInfo = await generator.generateChangelog();

      expect(releaseInfo.contributors).toContain('Test User');
      
      const changelogContent = generator['formatChangelog'](releaseInfo);
      expect(changelogContent).toContain('👥 Contributors');
      expect(changelogContent).toContain('Test User');
    });

    test('should filter out excluded scopes', async () => {
      testRepo.addCommit('feat(internal): add debug logging');
      testRepo.addCommit('feat(ui): add dashboard');

      const customGenerator = new ChangelogGenerator({
        excludeScopes: ['internal'],
      });

      const releaseInfo = await customGenerator.generateChangelog();
      
      // Should only include the UI feature, not internal
      const featureCommits = releaseInfo.commits.filter(c => c.type === 'feat');
      const includedFeatures = featureCommits.filter(c => c.scope !== 'internal');
      
      expect(includedFeatures.length).toBeGreaterThan(0);
      
      const changelogContent = customGenerator['formatChangelog'](releaseInfo);
      expect(changelogContent).toContain('add dashboard');
      expect(changelogContent).not.toContain('debug logging');
    });
  });

  describe('Release Creation', () => {
    test('should generate release notes with artifacts', async () => {
      // Create mock artifacts
      fs.mkdirSync('dist', { recursive: true });
      fs.writeFileSync('dist/app.tar.gz', 'mock artifact');
      fs.writeFileSync('dist/app.zip', 'mock artifact');

      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);

      const customGenerator = new ChangelogGenerator({
        artifactPaths: ['dist/*.tar.gz', 'dist/*.zip'],
      });

      const releaseInfo = await customGenerator.generateChangelog();
      
      const releaseNotes = customGenerator['formatReleaseNotes'](releaseInfo);
      
      expect(releaseNotes).toContain('What\'s Changed');
      expect(releaseNotes).toContain('Contributors');
      expect(releaseNotes).toContain('Release Artifacts');
      expect(releaseNotes).toContain('app.tar.gz');
      expect(releaseNotes).toContain('app.zip');
    });

    test('should create GitHub release command', async () => {
      testRepo.addCommit(testCommits.feature);

      const releaseInfo = await generator.generateChangelog();
      await generator.createGitHubRelease(releaseInfo, false, false);

      // Check if release command file was created
      expect(fs.existsSync('.github-release-command')).toBe(true);
      
      const releaseCommand = fs.readFileSync('.github-release-command', 'utf8');
      expect(releaseCommand).toContain('gh release create');
      expect(releaseCommand).toContain(`"v${releaseInfo.version}"`);
      expect(releaseCommand).toContain('--title');
      expect(releaseCommand).toContain('--notes');
    });

    test('should handle draft and prerelease flags', async () => {
      testRepo.addCommit(testCommits.feature);

      const releaseInfo = await generator.generateChangelog();
      await generator.createGitHubRelease(releaseInfo, true, true);

      const releaseCommand = fs.readFileSync('.github-release-command', 'utf8');
      expect(releaseCommand).toContain('--draft');
      expect(releaseCommand).toContain('--prerelease');
    });
  });

  describe('Validation', () => {
    test('should validate conventional commit format', () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.addCommit(testCommits.nonConventional);

      const validation = generator.validateCommits(undefined, 'HEAD');

      expect(validation.valid).toBe(false); // Non-conventional commit present
      expect(validation.errors.length).toBeGreaterThan(0);
      expect(validation.errors[0]).toContain('Invalid commit format');
      expect(validation.errors[0]).toContain(testCommits.nonConventional);
    });

    test('should pass validation for all conventional commits', () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.addCommit(testCommits.breaking);

      const validation = generator.validateCommits(undefined, 'HEAD');

      expect(validation.valid).toBe(true);
      expect(validation.errors.length).toBe(0);
    });
  });

  describe('CI Integration Scenarios', () => {
    test('should handle tag-triggered release workflow', async () => {
      // Simulate existing release
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);
      testRepo.createTag('v1.0.0');

      // Add new commits for next release
      testRepo.addCommit('feat(api): add authentication');
      testRepo.addCommit('fix(ui): resolve layout issue');
      testRepo.addCommit('docs: update API documentation');

      // Simulate CI tag trigger
      testRepo.createTag('v1.1.0');

      // Generate changelog between tags
      const releaseInfo = await generator.generateChangelog('v1.0.0', 'v1.1.0');

      expect(releaseInfo.version).toBe('1.1.0');
      expect(releaseInfo.previousVersion).toBe('1.0.0');
      
      // Should only include commits between tags
      const newCommits = releaseInfo.commits.filter(c => 
        c.description.includes('authentication') || 
        c.description.includes('layout issue') ||
        c.description.includes('API documentation')
      );
      
      expect(newCommits.length).toBe(3);
    });

    test('should generate artifacts list for CI', () => {
      // Create mock build artifacts
      fs.mkdirSync('dist', { recursive: true });
      const artifacts = [
        'dist/polymera-os-linux-x64.tar.gz',
        'dist/polymera-os-windows-x64.zip',
        'dist/polymera-os-macos-x64.tar.gz',
      ];

      artifacts.forEach(artifact => {
        fs.writeFileSync(artifact, 'mock content');
      });

      const foundArtifacts = generator['findArtifacts']();
      
      expect(foundArtifacts.length).toBe(artifacts.length);
      artifacts.forEach(artifact => {
        expect(foundArtifacts).toContain(artifact);
      });
    });

    test('should output version for GitHub Actions', async () => {
      // Mock GitHub Actions environment
      const outputFile = path.join(os.tmpdir(), 'github-output');
      process.env.GITHUB_OUTPUT = outputFile;

      testRepo.addCommit(testCommits.feature);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);
      const versionBump = generator.calculateVersionBump(conventionalCommits);

      // Simulate the bump command that writes to GITHUB_OUTPUT
      fs.writeFileSync(outputFile, `version=${versionBump.next}\nbump-type=${versionBump.type}\n`);

      const output = fs.readFileSync(outputFile, 'utf8');
      expect(output).toContain('version=1.1.0');
      expect(output).toContain('bump-type=minor');

      // Cleanup
      delete process.env.GITHUB_OUTPUT;
      fs.unlinkSync(outputFile);
    });

    test('should handle workflow dispatch with custom version', async () => {
      testRepo.addCommit(testCommits.feature);
      testRepo.addCommit(testCommits.fix);

      // Simulate manual release with custom version
      const customVersion = '2.0.0-beta.1';
      const releaseInfo = await generator.generateChangelog(undefined, `v${customVersion}`);

      expect(releaseInfo.version).toBe(customVersion);

      const changelogContent = generator['formatChangelog'](releaseInfo);
      expect(changelogContent).toContain(`## [${customVersion}]`);
    });
  });

  describe('Edge Cases and Error Handling', () => {
    test('should handle empty commit range', async () => {
      testRepo.createTag('v1.0.0');
      
      // Try to generate changelog with no new commits
      const releaseInfo = await generator.generateChangelog('v1.0.0', 'HEAD');

      expect(releaseInfo.commits.length).toBe(0);
      expect(releaseInfo.sections.length).toBe(0);
      expect(releaseInfo.stats.totalCommits).toBe(0);
    });

    test('should handle repository without any tags', async () => {
      testRepo.addCommit(testCommits.feature);

      const releaseInfo = await generator.generateChangelog();

      expect(releaseInfo.previousVersion).toBeUndefined();
      expect(releaseInfo.version).toBe('1.1.0'); // Bump from package.json version
    });

    test('should handle malformed commit messages gracefully', () => {
      // Add commits with various malformed structures
      testRepo.addCommit('feat('); // Malformed scope
      testRepo.addCommit('feat: '); // Empty description
      testRepo.addCommit('feat()!: test'); // Empty scope with breaking
      testRepo.addCommit('FEAT: uppercase type'); // Wrong case

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);

      // Should not throw errors and should categorize commits
      expect(conventionalCommits.length).toBeGreaterThan(0);
      
      // All malformed commits should be categorized as 'chore' by default
      const malformedCommits = conventionalCommits.filter(c => 
        c.description.includes('feat(') || 
        c.description.includes('FEAT:') ||
        c.description === ''
      );
      
      expect(malformedCommits.every(c => c.type === 'chore')).toBe(true);
    });

    test('should handle very large commit messages', () => {
      const longDescription = 'a'.repeat(1000);
      const longBody = 'b'.repeat(5000);
      testRepo.addCommit(`feat: ${longDescription}\n\n${longBody}`);

      const commits = generator['getCommitsSinceTag'](undefined, 'HEAD');
      const conventionalCommits = generator['parseConventionalCommits'](commits);

      const longCommit = conventionalCommits.find(c => c.description.includes('aaa'));
      expect(longCommit).toBeDefined();
      expect(longCommit?.body).toContain('bbb');
    });
  });

  describe('Configuration Options', () => {
    test('should respect custom type configuration', async () => {
      const customConfig = {
        types: {
          feat: { title: '✨ New Features', order: 1, includeInChangelog: true },
          fix: { title: '🔧 Bug Fixes', order: 2, includeInChangelog: true },
          custom: { title: '🎨 Custom Changes', order: 3, includeInChangelog: true },
        },
      };

      testRepo.addCommit('custom: add special functionality');
      testRepo.addCommit(testCommits.feature);

      const customGenerator = new ChangelogGenerator(customConfig);
      const releaseInfo = await customGenerator.generateChangelog();

      const changelogContent = customGenerator['formatChangelog'](releaseInfo);
      expect(changelogContent).toContain('✨ New Features');
      expect(changelogContent).toContain('🎨 Custom Changes');
    });

    test('should exclude configured commit types', async () => {
      const customConfig = {
        types: {
          test: { title: 'Tests', order: 1, includeInChangelog: false },
          ci: { title: 'CI', order: 2, includeInChangelog: false },
          feat: { title: 'Features', order: 3, includeInChangelog: true },
        },
      };

      testRepo.addCommit('test: add unit tests');
      testRepo.addCommit('ci: update workflow');
      testRepo.addCommit(testCommits.feature);

      const customGenerator = new ChangelogGenerator(customConfig);
      const releaseInfo = await customGenerator.generateChangelog();

      const changelogContent = customGenerator['formatChangelog'](releaseInfo);
      expect(changelogContent).toContain('Features');
      expect(changelogContent).not.toContain('Tests');
      expect(changelogContent).not.toContain('CI');
    });
  });
});

// Integration tests that simulate the full CI workflow
describe('CI Workflow Integration', () => {
  let testRepo: TestGitRepo;

  beforeEach(async () => {
    testRepo = new TestGitRepo();
    await testRepo.setup();
  });

  afterEach(() => {
    testRepo.cleanup();
  });

  test('should simulate complete tag-triggered release workflow', async () => {
    // Phase 1: Development commits
    testRepo.addCommit('feat(ui): add user dashboard');
    testRepo.addCommit('feat(api): implement user authentication');
    testRepo.addCommit('fix(db): resolve connection timeout issue');
    testRepo.addCommit('docs: update API documentation');
    testRepo.addCommit('test: add integration tests for auth');

    // Phase 2: Create release tag (simulates GitHub tag push)
    const releaseTag = 'v1.1.0';
    testRepo.createTag(releaseTag);

    // Phase 3: CI validates commits
    const generator = new ChangelogGenerator();
    const validation = generator.validateCommits(undefined, releaseTag);
    expect(validation.valid).toBe(true);

    // Phase 4: Generate changelog
    const releaseInfo = await generator.generateChangelog(undefined, releaseTag, 'RELEASE_CHANGELOG.md');
    
    expect(releaseInfo.version).toBe('1.1.0');
    expect(releaseInfo.stats.featuresCount).toBe(2);
    expect(releaseInfo.stats.fixesCount).toBe(1);

    // Phase 5: Verify changelog file created
    expect(fs.existsSync('RELEASE_CHANGELOG.md')).toBe(true);
    const changelogContent = fs.readFileSync('RELEASE_CHANGELOG.md', 'utf8');
    
    expect(changelogContent).toContain('## [1.1.0]');
    expect(changelogContent).toContain('user dashboard');
    expect(changelogContent).toContain('user authentication');
    expect(changelogContent).toContain('connection timeout');
    
    // Phase 6: Create GitHub release
    await generator.createGitHubRelease(releaseInfo, false, false);
    expect(fs.existsSync('.github-release-command')).toBe(true);

    console.log('✅ Complete CI workflow simulation passed');
  });

  test('should handle pre-release workflow', async () => {
    testRepo.addCommit('feat: add experimental feature');
    testRepo.addCommit('fix: minor bug fixes');

    const prereleaseTag = 'v2.0.0-alpha.1';
    testRepo.createTag(prereleaseTag);

    const generator = new ChangelogGenerator();
    const releaseInfo = await generator.generateChangelog(undefined, prereleaseTag);
    
    await generator.createGitHubRelease(releaseInfo, false, true);

    const releaseCommand = fs.readFileSync('.github-release-command', 'utf8');
    expect(releaseCommand).toContain('--prerelease');
    expect(releaseCommand).toContain('v2.0.0-alpha.1');

    console.log('✅ Pre-release workflow simulation passed');
  });

  test('should handle multi-release scenario', async () => {
    const generator = new ChangelogGenerator();

    // Release 1.0.0
    testRepo.addCommit('feat: initial release features');
    testRepo.addCommit('fix: initial bug fixes');
    testRepo.createTag('v1.0.0');

    // Release 1.1.0
    testRepo.addCommit('feat: add new dashboard');
    testRepo.addCommit('feat: implement search functionality');
    testRepo.createTag('v1.1.0');

    // Release 2.0.0 (with breaking changes)
    testRepo.addCommit('feat!: redesign API structure\n\nBREAKING CHANGE: API endpoints have changed');
    testRepo.addCommit('feat: add GraphQL support');
    testRepo.createTag('v2.0.0');

    // Generate full changelog
    await generator.generateFullChangelog('FULL_CHANGELOG.md');
    
    const fullChangelog = fs.readFileSync('FULL_CHANGELOG.md', 'utf8');
    
    expect(fullChangelog).toContain('## [v2.0.0]');
    expect(fullChangelog).toContain('## [v1.1.0]');
    expect(fullChangelog).toContain('## [v1.0.0]');
    expect(fullChangelog).toContain('⚠️ BREAKING CHANGES');
    expect(fullChangelog).toContain('API endpoints have changed');

    console.log('✅ Multi-release scenario simulation passed');
  });
});

console.log('🧪 Changelog Generator Test Suite Initialized');
console.log('Run with: npm test or node --test gen.test.ts');
