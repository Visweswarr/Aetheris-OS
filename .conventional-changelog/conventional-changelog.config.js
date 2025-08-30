module.exports = {
  // Conventional Changelog configuration for Polymera OS

  // Preset configuration
  preset: 'angular',

  // Release rules for Polymera OS
  releaseRules: [
    // Breaking changes
    { type: 'feat', release: 'minor' },
    { type: 'fix', release: 'patch' },
    { type: 'docs', release: 'patch' },
    { type: 'style', release: 'patch' },
    { type: 'refactor', release: 'patch' },
    { type: 'perf', release: 'patch' },
    { type: 'test', release: 'patch' },
    { type: 'chore', release: 'patch' },
    { type: 'ci', release: 'patch' },
    { type: 'build', release: 'patch' },

    // Breaking changes always trigger major release
    { breaking: true, release: 'major' },

    // Security fixes trigger patch release
    { type: 'fix', scope: 'security', release: 'patch' },

    // Performance improvements trigger patch release
    { type: 'perf', release: 'patch' },

    // Documentation updates trigger patch release
    { type: 'docs', release: 'patch' }
  ],

  // Parser options
  parserOpts: {
    noteKeywords: ['BREAKING CHANGE', 'BREAKING CHANGES', 'SECURITY', 'SECURITY FIX'],
    issuePrefixes: ['#'],
    referenceActions: ['closes', 'fixes', 'resolves', 'relates', 'addresses']
  },

  // Writer options
  writerOpts: {
    // Group commits by type
    groupBy: 'type',

    // Sort commits within groups
    commitGroupsSort: 'title',

    // Sort commits within groups
    commitsSort: ['scope', 'subject'],

    // Note groups
    noteGroupsSort: 'title',

    // Main template
    mainTemplate: `# Changelog

All notable changes to Polymera OS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

{{#if compareUrl}}
[Full Changelog]({{compareUrl}})
{{/if}}

{{#if releases.length}}
{{#each releases}}
{{#unless @first}}
## [{{name}}] - {{isoDate}}

{{#if summary}}
{{summary}}
{{/if}}

{{#if merges.length}}
### Merged
{{#each merges}}
- {{#if this.owner}}{{this.owner}}/{{/if}}{{this.repository}}#{{this.number}} {{this.message}}
{{/each}}
{{/if}}

{{#if fixes.length}}
### Fixed
{{#each fixes}}
- {{#if this.owner}}{{this.owner}}/{{/if}}{{this.repository}}#{{this.number}} {{this.message}}
{{/each}}
{{/if}}

{{#if commits.length}}
{{#each commitGroups}}
{{#if title}}
### {{title}}
{{/if}}
{{#each commits}}
{{#if this.scope}}- **{{this.scope}}**: {{/if}}{{this.subject}}
{{/each}}

{{/each}}
{{/if}}

{{#if breaking.length}}
### BREAKING CHANGES
{{#each breaking}}
- {{this.message}}
{{/each}}
{{/if}}

{{#if notes.length}}
{{#each noteGroups}}
{{#if title}}
### {{title}}
{{/if}}
{{#each notes}}
{{this.message}}
{{/each}}

{{/each}}
{{/if}}

{{/unless}}
{{/each}}
{{/if}}

[Unreleased]: {{compareUrl}}
{{#each releases}}
{{#unless @first}}
[{{name}}]: {{../compareUrl}}...{{name}}
{{/unless}}
{{/each}}`,

    // Commit template
    commitTemplate: '{{#if scope}}**{{scope}}**: {{/if}}{{subject}}',

    // Issue template
    issueTemplate: '{{#if this.owner}}{{this.owner}}/{{/if}}{{this.repository}}#{{this.number}}',

    // Merge template
    mergeTemplate: '{{#if this.owner}}{{this.owner}}/{{/if}}{{this.repository}}#{{this.number}} {{this.message}}',

    // Note template
    noteTemplate: '{{text}}',

    // Header template
    headerPartial: `# Changelog

All notable changes to Polymera OS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

{{#if compareUrl}}
[Full Changelog]({{compareUrl}})
{{/if}}

`,

    // Commit partial
    commitPartial: `{{#if scope}}**{{scope}}**: {{/if}}{{subject}}`,

    // Footer partial
    footerPartial: `
[Unreleased]: {{compareUrl}}
{{#each releases}}
{{#unless @first}}
[{{name}}]: {{../compareUrl}}...{{name}}
{{/unless}}
{{/each}}`
  },

  // Plugin options
  plugins: [
    // Conventional Changelog Angular preset
    '@semantic-release/commit-analyzer',
    '@semantic-release/release-notes-generator',
    '@semantic-release/changelog',
    '@semantic-release/npm',
    '@semantic-release/git',
    '@semantic-release/github'
  ],

  // Semantic release configuration
  branches: [
    'main',
    {
      name: 'develop',
      prerelease: true,
      channel: 'beta'
    }
  ],

  // Release configuration
  tagFormat: 'v${version}',

  // Changelog configuration
  changelogFile: 'CHANGELOG.md',
  changelogTitle: '# Polymera OS Changelog',

  // GitHub configuration
  githubUrl: 'https://github.com/polymera-os/polymera-os',
  githubApiPathPrefix: '/api/v3',

  // NPM configuration
  npmPublish: false,

  // Git configuration
  git: {
    assets: [
      'CHANGELOG.md',
      'package.json',
      'package-lock.json'
    ],
    message: 'chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}'
  },

  // GitHub configuration
  github: {
    assets: [
      {
        path: 'dist/*.tar.gz',
        label: 'Source Code (tar.gz)'
      },
      {
        path: 'dist/*.zip',
        label: 'Source Code (zip)'
      }
    ],
    successComment: 'This PR is included in version ${nextRelease.version} :tada:',
    failTitle: 'The automated release is failing :rotating_light:',
    labels: ['release']
  }
};
