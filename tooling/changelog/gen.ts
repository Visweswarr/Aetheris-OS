#!/usr/bin/env node

/**
 * Polymera OS Changelog Generator
 * 
 * Automatically generates changelogs from conventional commits following the
 * Conventional Commits specification (https://conventionalcommits.org/).
 * 
 * Features:
 * - Parses conventional commits and categorizes changes
 * - Generates semantic version bumps based on commit types
 * - Creates markdown changelogs with proper formatting
 * - Supports release artifacts and GitHub release creation
 * - Integrates with CI/CD for automated releases
 */

import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

// Types for conventional commits and changelog generation
interface ConventionalCommit {
  hash: string;
  type: string;
  scope?: string;
  description: string;
  body?: string;
  footer?: string;
  breaking: boolean;
  timestamp: Date;
  author: string;
  pr?: number;
}

interface ChangelogSection {
  title: string;
  commits: ConventionalCommit[];
  order: number;
}

interface ReleaseInfo {
  version: string;
  date: string;
  previousVersion?: string;
  commits: ConventionalCommit[];
  sections: ChangelogSection[];
  breakingChanges: ConventionalCommit[];
  contributors: string[];
  stats: {
    totalCommits: number;
    featuresCount: number;
    fixesCount: number;
    breakingCount: number;
  };
}

interface VersionBump {
  current: string;
  next: string;
  type: 'major' | 'minor' | 'patch';
  reason: string;
}

// Configuration for changelog generation
interface ChangelogConfig {
  types: Record<string, { title: string; order: number; includeInChangelog: boolean }>;
  scopes: Record<string, string>;
  excludeScopes: string[];
  includeAuthors: boolean;
  includePRLinks: boolean;
  githubRepo: string;
  releaseNotesTemplate: string;
  artifactPaths: string[];
}

const defaultConfig: ChangelogConfig = {
  types: {
    feat: { title: '🚀 Features', order: 1, includeInChangelog: true },
    fix: { title: '🐛 Bug Fixes', order: 2, includeInChangelog: true },
    perf: { title: '⚡ Performance Improvements', order: 3, includeInChangelog: true },
    refactor: { title: '♻️ Code Refactoring', order: 4, includeInChangelog: true },
    docs: { title: '📚 Documentation', order: 5, includeInChangelog: true },
    test: { title: '✅ Tests', order: 6, includeInChangelog: false },
    build: { title: '🏗️ Build System', order: 7, includeInChangelog: true },
    ci: { title: '👷 CI/CD', order: 8, includeInChangelog: false },
    chore: { title: '🔧 Maintenance', order: 9, includeInChangelog: false },
    style: { title: '🎨 Styling', order: 10, includeInChangelog: false },
    revert: { title: '⏪ Reverts', order: 11, includeInChangelog: true },
  },
  scopes: {
    kernel: 'Kernel',
    services: 'Services',
    ui: 'User Interface',
    docs: 'Documentation',
    ci: 'CI/CD',
    deps: 'Dependencies',
    release: 'Release',
    security: 'Security',
  },
  excludeScopes: ['internal', 'temp'],
  includeAuthors: true,
  includePRLinks: true,
  githubRepo: 'polymera-os/polymera-os',
  releaseNotesTemplate: `## What's Changed

{CHANGELOG}

{CONTRIBUTORS}

{ARTIFACTS}

**Full Changelog**: {COMPARE_URL}`,
  artifactPaths: [
    'dist/*.tar.gz',
    'dist/*.zip', 
    'dist/*.dmg',
    'dist/*.exe',
    'dist/*.deb',
    'dist/*.rpm',
  ],
};

class ChangelogGenerator {
  private config: ChangelogConfig;
  private currentVersion: string;
  private gitDirectory: string;

  constructor(config: Partial<ChangelogConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
    this.currentVersion = this.getCurrentVersion();
    this.gitDirectory = this.findGitDirectory();
  }

  /**
   * Generate changelog for a specific version range
   */
  async generateChangelog(
    fromTag?: string,
    toTag: string = 'HEAD',
    outputFile?: string
  ): Promise<ReleaseInfo> {
    console.log('🔄 Generating changelog...');
    
    const commits = this.getCommitsSinceTag(fromTag, toTag);
    const conventionalCommits = this.parseConventionalCommits(commits);
    const sections = this.groupCommitsByType(conventionalCommits);
    const breakingChanges = conventionalCommits.filter(c => c.breaking);
    const contributors = this.getUniqueContributors(conventionalCommits);
    
    const versionBump = this.calculateVersionBump(conventionalCommits);
    const version = toTag === 'HEAD' ? versionBump.next : toTag.replace(/^v/, '');
    
    const releaseInfo: ReleaseInfo = {
      version,
      date: new Date().toISOString().split('T')[0],
      previousVersion: fromTag?.replace(/^v/, ''),
      commits: conventionalCommits,
      sections,
      breakingChanges,
      contributors,
      stats: {
        totalCommits: conventionalCommits.length,
        featuresCount: conventionalCommits.filter(c => c.type === 'feat').length,
        fixesCount: conventionalCommits.filter(c => c.type === 'fix').length,
        breakingCount: breakingChanges.length,
      },
    };

    const changelogContent = this.formatChangelog(releaseInfo);
    
    if (outputFile) {
      await this.writeChangelog(changelogContent, outputFile);
      console.log(`✅ Changelog written to ${outputFile}`);
    }

    return releaseInfo;
  }

  /**
   * Generate full changelog from all tags
   */
  async generateFullChangelog(outputFile: string = 'CHANGELOG.md'): Promise<void> {
    console.log('🔄 Generating full changelog from all releases...');
    
    const tags = this.getAllTags();
    let fullChangelog = this.getChangelogHeader();
    
    for (let i = 0; i < tags.length; i++) {
      const fromTag = tags[i + 1];
      const toTag = tags[i];
      
      console.log(`Processing ${toTag}${fromTag ? ` (from ${fromTag})` : ' (initial)'}`);
      
      const releaseInfo = await this.generateChangelog(fromTag, toTag);
      fullChangelog += this.formatChangelog(releaseInfo) + '\n';
    }
    
    // Add unreleased changes if any
    if (tags.length > 0) {
      const unreleasedCommits = this.getCommitsSinceTag(tags[0], 'HEAD');
      if (unreleasedCommits.length > 0) {
        console.log('Processing unreleased changes...');
        const unreleasedInfo = await this.generateChangelog(tags[0], 'HEAD');
        unreleasedInfo.version = 'Unreleased';
        fullChangelog = this.getChangelogHeader() + 
          this.formatChangelog(unreleasedInfo) + '\n' + 
          fullChangelog.replace(this.getChangelogHeader(), '');
      }
    }
    
    await this.writeChangelog(fullChangelog, outputFile);
    console.log(`✅ Full changelog written to ${outputFile}`);
  }

  /**
   * Calculate semantic version bump based on conventional commits
   */
  calculateVersionBump(commits: ConventionalCommit[]): VersionBump {
    const hasBreaking = commits.some(c => c.breaking);
    const hasFeatures = commits.some(c => c.type === 'feat');
    const hasFixes = commits.some(c => c.type === 'fix');
    
    let bumpType: 'major' | 'minor' | 'patch';
    let reason: string;
    
    if (hasBreaking) {
      bumpType = 'major';
      reason = 'Breaking changes detected';
    } else if (hasFeatures) {
      bumpType = 'minor';
      reason = 'New features added';
    } else if (hasFixes || commits.length > 0) {
      bumpType = 'patch';
      reason = 'Bug fixes and improvements';
    } else {
      bumpType = 'patch';
      reason = 'No significant changes';
    }
    
    const nextVersion = this.bumpVersion(this.currentVersion, bumpType);
    
    return {
      current: this.currentVersion,
      next: nextVersion,
      type: bumpType,
      reason,
    };
  }

  /**
   * Create GitHub release with artifacts
   */
  async createGitHubRelease(
    releaseInfo: ReleaseInfo,
    isDraft: boolean = false,
    isPrerelease: boolean = false
  ): Promise<void> {
    console.log(`🚀 Creating GitHub release for v${releaseInfo.version}...`);
    
    const releaseNotes = this.formatReleaseNotes(releaseInfo);
    const tagName = `v${releaseInfo.version}`;
    
    // Create the tag if it doesn't exist
    try {
      execSync(`git tag -a ${tagName} -m "Release ${tagName}"`, { stdio: 'pipe' });
      console.log(`✅ Created git tag ${tagName}`);
    } catch (error) {
      console.log(`ℹ️ Tag ${tagName} already exists`);
    }
    
    // Push the tag
    try {
      execSync(`git push origin ${tagName}`, { stdio: 'pipe' });
      console.log(`✅ Pushed tag ${tagName} to origin`);
    } catch (error) {
      console.warn(`⚠️ Failed to push tag: ${error}`);
    }
    
    // Generate release command for GitHub CLI
    const artifacts = this.findArtifacts();
    const artifactArgs = artifacts.map(file => `--attach "${file}"`).join(' ');
    
    const releaseCommand = [
      'gh release create',
      `"${tagName}"`,
      `--title "Release ${tagName}"`,
      `--notes "${releaseNotes.replace(/"/g, '\\"')}"`,
      isDraft ? '--draft' : '',
      isPrerelease ? '--prerelease' : '',
      artifactArgs,
    ].filter(Boolean).join(' ');
    
    console.log('GitHub CLI command:');
    console.log(releaseCommand);
    
    // Save release command to file for CI execution
    fs.writeFileSync('.github-release-command', releaseCommand);
    console.log('✅ Release command saved to .github-release-command');
  }

  /**
   * Validate conventional commits format
   */
  validateCommits(fromTag?: string, toTag: string = 'HEAD'): { valid: boolean; errors: string[] } {
    console.log('🔍 Validating conventional commits format...');
    
    const commits = this.getCommitsSinceTag(fromTag, toTag);
    const errors: string[] = [];
    
    for (const commit of commits) {
      if (!this.isValidConventionalCommit(commit)) {
        errors.push(`Invalid commit format: ${commit.hash.substring(0, 8)} - "${commit.description}"`);
      }
    }
    
    const valid = errors.length === 0;
    
    if (valid) {
      console.log(`✅ All ${commits.length} commits follow conventional format`);
    } else {
      console.log(`❌ Found ${errors.length} commits with invalid format`);
      errors.forEach(error => console.log(`  - ${error}`));
    }
    
    return { valid, errors };
  }

  // Private methods for implementation

  private getCurrentVersion(): string {
    try {
      const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
      return packageJson.version || '0.0.0';
    } catch {
      try {
        const latestTag = execSync('git describe --tags --abbrev=0', { encoding: 'utf8' }).trim();
        return latestTag.replace(/^v/, '');
      } catch {
        return '0.0.0';
      }
    }
  }

  private findGitDirectory(): string {
    let currentDir = process.cwd();
    while (currentDir !== '/') {
      if (fs.existsSync(path.join(currentDir, '.git'))) {
        return currentDir;
      }
      currentDir = path.dirname(currentDir);
    }
    throw new Error('Not in a git repository');
  }

  private getCommitsSinceTag(fromTag?: string, toTag: string = 'HEAD'): any[] {
    let command: string;
    
    if (fromTag) {
      command = `git log ${fromTag}..${toTag} --pretty=format:"%H|%s|%b|%an|%ad" --date=iso`;
    } else {
      command = `git log ${toTag} --pretty=format:"%H|%s|%b|%an|%ad" --date=iso`;
    }
    
    try {
      const output = execSync(command, { encoding: 'utf8', cwd: this.gitDirectory });
      
      return output.split('\n')
        .filter(line => line.trim())
        .map(line => {
          const [hash, subject, body, author, date] = line.split('|');
          return {
            hash,
            description: subject,
            body: body || '',
            author,
            timestamp: new Date(date),
          };
        });
    } catch (error) {
      console.warn(`Warning: Could not get commits: ${error}`);
      return [];
    }
  }

  private parseConventionalCommits(commits: any[]): ConventionalCommit[] {
    return commits
      .map(commit => this.parseConventionalCommit(commit))
      .filter((commit): commit is ConventionalCommit => commit !== null);
  }

  private parseConventionalCommit(commit: any): ConventionalCommit | null {
    const conventionalPattern = /^(\w+)(\(([^)]+)\))?(!)?: (.+)$/;
    const match = commit.description.match(conventionalPattern);
    
    if (!match) {
      // Try to categorize non-conventional commits
      const description = commit.description.toLowerCase();
      let type = 'chore';
      
      if (description.includes('fix') || description.includes('bug')) {
        type = 'fix';
      } else if (description.includes('add') || description.includes('feat')) {
        type = 'feat';
      } else if (description.includes('doc')) {
        type = 'docs';
      } else if (description.includes('test')) {
        type = 'test';
      }
      
      return {
        hash: commit.hash,
        type,
        description: commit.description,
        body: commit.body,
        breaking: false,
        timestamp: commit.timestamp,
        author: commit.author,
      };
    }
    
    const [, type, , scope, breakingMarker, description] = match;
    const breaking = breakingMarker === '!' || 
      commit.body.includes('BREAKING CHANGE') || 
      commit.body.includes('BREAKING-CHANGE');
    
    // Extract PR number from description or body
    const prMatch = (commit.description + ' ' + commit.body).match(/#(\d+)/);
    const pr = prMatch ? parseInt(prMatch[1]) : undefined;
    
    return {
      hash: commit.hash,
      type,
      scope,
      description,
      body: commit.body,
      breaking,
      timestamp: commit.timestamp,
      author: commit.author,
      pr,
    };
  }

  private isValidConventionalCommit(commit: any): boolean {
    const conventionalPattern = /^(\w+)(\(([^)]+)\))?(!)?: (.+)$/;
    return conventionalPattern.test(commit.description);
  }

  private groupCommitsByType(commits: ConventionalCommit[]): ChangelogSection[] {
    const groups: Record<string, ConventionalCommit[]> = {};
    
    for (const commit of commits) {
      if (this.config.excludeScopes.includes(commit.scope || '')) {
        continue;
      }
      
      const typeConfig = this.config.types[commit.type];
      if (!typeConfig?.includeInChangelog) {
        continue;
      }
      
      if (!groups[commit.type]) {
        groups[commit.type] = [];
      }
      groups[commit.type].push(commit);
    }
    
    return Object.entries(groups)
      .map(([type, commits]) => ({
        title: this.config.types[type]?.title || type,
        commits: commits.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime()),
        order: this.config.types[type]?.order || 999,
      }))
      .sort((a, b) => a.order - b.order);
  }

  private getUniqueContributors(commits: ConventionalCommit[]): string[] {
    const contributors = new Set(commits.map(c => c.author));
    return Array.from(contributors).sort();
  }

  private formatChangelog(releaseInfo: ReleaseInfo): string {
    let changelog = '';
    
    // Version header
    const versionHeader = releaseInfo.version === 'Unreleased' 
      ? '## [Unreleased]'
      : `## [${releaseInfo.version}] - ${releaseInfo.date}`;
    
    changelog += versionHeader + '\n\n';
    
    // Release summary
    if (releaseInfo.stats.totalCommits > 0) {
      const summary = [
        `${releaseInfo.stats.totalCommits} commit${releaseInfo.stats.totalCommits !== 1 ? 's' : ''}`,
        releaseInfo.stats.featuresCount > 0 ? `${releaseInfo.stats.featuresCount} feature${releaseInfo.stats.featuresCount !== 1 ? 's' : ''}` : null,
        releaseInfo.stats.fixesCount > 0 ? `${releaseInfo.stats.fixesCount} fix${releaseInfo.stats.fixesCount !== 1 ? 'es' : ''}` : null,
        releaseInfo.stats.breakingCount > 0 ? `${releaseInfo.stats.breakingCount} breaking change${releaseInfo.stats.breakingCount !== 1 ? 's' : ''}` : null,
      ].filter(Boolean).join(', ');
      
      changelog += `*${summary}*\n\n`;
    }
    
    // Breaking changes section
    if (releaseInfo.breakingChanges.length > 0) {
      changelog += '### ⚠️ BREAKING CHANGES\n\n';
      for (const commit of releaseInfo.breakingChanges) {
        changelog += this.formatCommitLine(commit) + '\n';
        if (commit.body && commit.body.includes('BREAKING CHANGE')) {
          const breakingNote = commit.body.split('BREAKING CHANGE:')[1]?.trim();
          if (breakingNote) {
            changelog += `  ${breakingNote}\n`;
          }
        }
      }
      changelog += '\n';
    }
    
    // Sections by type
    for (const section of releaseInfo.sections) {
      if (section.commits.length > 0) {
        changelog += `### ${section.title}\n\n`;
        for (const commit of section.commits) {
          changelog += this.formatCommitLine(commit) + '\n';
        }
        changelog += '\n';
      }
    }
    
    // Contributors
    if (this.config.includeAuthors && releaseInfo.contributors.length > 0) {
      changelog += '### 👥 Contributors\n\n';
      for (const contributor of releaseInfo.contributors) {
        changelog += `- ${contributor}\n`;
      }
      changelog += '\n';
    }
    
    return changelog;
  }

  private formatCommitLine(commit: ConventionalCommit): string {
    let line = `- `;
    
    // Add scope if present
    if (commit.scope) {
      const scopeTitle = this.config.scopes[commit.scope] || commit.scope;
      line += `**${scopeTitle}**: `;
    }
    
    // Add description
    line += commit.description;
    
    // Add PR link if available
    if (this.config.includePRLinks && commit.pr) {
      line += ` ([#${commit.pr}](https://github.com/${this.config.githubRepo}/pull/${commit.pr}))`;
    }
    
    // Add commit hash link
    line += ` ([${commit.hash.substring(0, 8)}](https://github.com/${this.config.githubRepo}/commit/${commit.hash}))`;
    
    return line;
  }

  private formatReleaseNotes(releaseInfo: ReleaseInfo): string {
    const changelog = this.formatChangelog(releaseInfo);
    const contributors = releaseInfo.contributors.length > 0 
      ? `## 👥 Contributors\n\nThanks to all contributors who made this release possible:\n${releaseInfo.contributors.map(c => `- ${c}`).join('\n')}\n\n`
      : '';
    
    const artifacts = this.findArtifacts();
    const artifactsList = artifacts.length > 0 
      ? `## 📦 Release Artifacts\n\n${artifacts.map(a => `- ${path.basename(a)}`).join('\n')}\n\n`
      : '';
    
    const compareUrl = releaseInfo.previousVersion 
      ? `https://github.com/${this.config.githubRepo}/compare/v${releaseInfo.previousVersion}...v${releaseInfo.version}`
      : `https://github.com/${this.config.githubRepo}/releases/tag/v${releaseInfo.version}`;
    
    return this.config.releaseNotesTemplate
      .replace('{CHANGELOG}', changelog)
      .replace('{CONTRIBUTORS}', contributors)
      .replace('{ARTIFACTS}', artifactsList)
      .replace('{COMPARE_URL}', compareUrl);
  }

  private getAllTags(): string[] {
    try {
      const output = execSync('git tag -l --sort=-version:refname', { encoding: 'utf8' });
      return output.split('\n').filter(tag => tag.trim()).filter(tag => /^v?\d+\.\d+\.\d+/.test(tag));
    } catch {
      return [];
    }
  }

  private bumpVersion(version: string, bumpType: 'major' | 'minor' | 'patch'): string {
    const [major, minor, patch] = version.split('.').map(Number);
    
    switch (bumpType) {
      case 'major':
        return `${major + 1}.0.0`;
      case 'minor':
        return `${major}.${minor + 1}.0`;
      case 'patch':
        return `${major}.${minor}.${patch + 1}`;
      default:
        return version;
    }
  }

  private getChangelogHeader(): string {
    return `# Changelog

All notable changes to Polymera OS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`;
  }

  private findArtifacts(): string[] {
    const artifacts: string[] = [];
    
    for (const pattern of this.config.artifactPaths) {
      try {
        const files = execSync(`find . -path "${pattern}" 2>/dev/null || echo ""`, { encoding: 'utf8' })
          .split('\n')
          .filter(file => file.trim())
          .map(file => file.trim());
        
        artifacts.push(...files);
      } catch {
        // Ignore errors for missing patterns
      }
    }
    
    return artifacts.filter(file => fs.existsSync(file));
  }

  private async writeChangelog(content: string, filepath: string): Promise<void> {
    const dir = path.dirname(filepath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
    
    fs.writeFileSync(filepath, content, 'utf8');
  }
}

// CLI Interface
async function main() {
  const args = process.argv.slice(2);
  const command = args[0];
  
  const generator = new ChangelogGenerator();
  
  try {
    switch (command) {
      case 'generate': {
        const fromTag = args[1];
        const toTag = args[2] || 'HEAD';
        const outputFile = args[3];
        
        const releaseInfo = await generator.generateChangelog(fromTag, toTag, outputFile);
        console.log(`\n📋 Release Summary for v${releaseInfo.version}:`);
        console.log(`   ${releaseInfo.stats.totalCommits} commits`);
        console.log(`   ${releaseInfo.stats.featuresCount} features`);
        console.log(`   ${releaseInfo.stats.fixesCount} fixes`);
        console.log(`   ${releaseInfo.stats.breakingCount} breaking changes`);
        break;
      }
      
      case 'full': {
        const outputFile = args[1] || 'CHANGELOG.md';
        await generator.generateFullChangelog(outputFile);
        break;
      }
      
      case 'bump': {
        const fromTag = args[1];
        const toTag = args[2] || 'HEAD';
        
        const commits = generator['getCommitsSinceTag'](fromTag, toTag);
        const conventionalCommits = generator['parseConventionalCommits'](commits);
        const versionBump = generator.calculateVersionBump(conventionalCommits);
        
        console.log(`\n📈 Version Bump Analysis:`);
        console.log(`   Current: ${versionBump.current}`);
        console.log(`   Next: ${versionBump.next}`);
        console.log(`   Type: ${versionBump.type}`);
        console.log(`   Reason: ${versionBump.reason}`);
        
        // Output for CI consumption
        if (process.env.GITHUB_OUTPUT) {
          fs.appendFileSync(process.env.GITHUB_OUTPUT, 
            `version=${versionBump.next}\nbump-type=${versionBump.type}\n`);
        }
        break;
      }
      
      case 'release': {
        const fromTag = args[1];
        const version = args[2];
        const isDraft = args.includes('--draft');
        const isPrerelease = args.includes('--prerelease');
        
        const releaseInfo = await generator.generateChangelog(fromTag, version ? `v${version}` : 'HEAD');
        await generator.createGitHubRelease(releaseInfo, isDraft, isPrerelease);
        break;
      }
      
      case 'validate': {
        const fromTag = args[1];
        const toTag = args[2] || 'HEAD';
        
        const validation = generator.validateCommits(fromTag, toTag);
        
        if (!validation.valid) {
          console.error('\n❌ Validation failed:');
          validation.errors.forEach(error => console.error(`  ${error}`));
          process.exit(1);
        }
        break;
      }
      
      default: {
        console.log(`
🔄 Polymera OS Changelog Generator

Usage:
  node gen.ts generate [from-tag] [to-tag] [output-file]
    Generate changelog for specific version range
    
  node gen.ts full [output-file]
    Generate full changelog from all tags
    
  node gen.ts bump [from-tag] [to-tag]
    Calculate version bump based on conventional commits
    
  node gen.ts release [from-tag] [version] [--draft] [--prerelease]
    Create GitHub release with changelog and artifacts
    
  node gen.ts validate [from-tag] [to-tag]
    Validate conventional commits format

Examples:
  node gen.ts generate v1.0.0 HEAD CHANGELOG.md
  node gen.ts full
  node gen.ts bump v1.0.0
  node gen.ts release v1.0.0 1.1.0
  node gen.ts validate v1.0.0
        `);
        break;
      }
    }
  } catch (error) {
    console.error(`❌ Error: ${error}`);
    process.exit(1);
  }
}

// Export for programmatic usage
export { ChangelogGenerator, ConventionalCommit, ReleaseInfo, VersionBump };

// Run CLI if this is the main module
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
