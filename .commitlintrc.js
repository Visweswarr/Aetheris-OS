module.exports = {
  extends: ['@commitlint/config-conventional'],

  // Custom rules for Polymera OS
  rules: {
    // Enforce conventional commit format
    'type-enum': [
      2,
      'always',
      [
        'feat',     // New feature for the user
        'fix',      // Bug fix for the user
        'docs',     // Documentation only changes
        'style',    // Changes that do not affect the meaning of the code
        'refactor', // Code change that neither fixes a bug nor adds a feature
        'perf',     // Code change that improves performance
        'test',     // Adding missing tests or correcting existing tests
        'chore',    // Changes to the build process or auxiliary tools
        'ci',       // Changes to CI configuration files and scripts
        'build',    // Changes that affect the build system or external dependencies
        'revert'    // Reverts a previous commit
      ]
    ],

    // Enforce scope for Polymera OS components
    'scope-enum': [
      2,
      'always',
      [
        // Core components
        'kernel',      // Kernel-related changes
        'crypto',      // Cryptographic implementations
        'services',    // Service layer changes
        'runtime',     // Runtime layer changes
        'ui',          // User interface changes
        'tooling',     // Build and development tools

        // Infrastructure
        'ci',          // CI/CD pipeline changes
        'docs',        // Documentation changes
        'build',       // Build system changes
        'test',        // Testing infrastructure changes
        'security',    // Security-related changes
        'performance', // Performance-related changes

        // Specific services
        'devicekit',   // Device management
        'polyaudio',   // Audio processing
        'polynet',     // Networking
        'keyvault',    // Key management
        'ngfs',        // File system
        'polyimage',   // Image processing
        'attestation', // Attestation service
        'wallet',      // Wallet service

        // Development tools
        'bazel',       // Bazel build system
        'nix',         // Nix package manager
        'rust',        // Rust toolchain
        'cpp',         // C++ toolchain
        'typescript',  // TypeScript toolchain
        'python'       // Python toolchain
      ]
    ],

    // Enforce scope format
    'scope-case': [2, 'always', 'lowercase'],

    // Enforce subject format
    'subject-case': [2, 'always', 'lowercase'],
    'subject-empty': [2, 'never'],
    'subject-full-stop': [2, 'never', '.'],
    'subject-max-length': [2, 'always', 72],

    // Enforce body format
    'body-leading-blank': [2, 'always'],
    'body-max-line-length': [2, 'always', 100],

    // Enforce footer format
    'footer-leading-blank': [2, 'always'],
    'footer-max-line-length': [2, 'always', 100],

    // Enforce header format
    'header-max-length': [2, 'always', 100],

    // Enforce type format
    'type-case': [2, 'always', 'lowercase'],
    'type-empty': [2, 'never'],

    // Custom rules for Polymera OS
    'scope-empty': [2, 'never'], // Require scope for all commits

    // Enforce conventional commit format
    'type-enum': [
      2,
      'always',
      [
        'feat',     // New feature for the user
        'fix',      // Bug fix for the user
        'docs',     // Documentation only changes
        'style',    // Changes that do not affect the meaning of the code
        'refactor', // Code change that neither fixes a bug nor adds a feature
        'perf',     // Code change that improves performance
        'test',     // Adding missing tests or correcting existing tests
        'chore',    // Changes to the build process or auxiliary tools
        'ci',       // Changes to CI configuration files and scripts
        'build',    // Changes that affect the build system or external dependencies
        'revert'    // Reverts a previous commit
      ]
    ]
  },

  // Custom parser for Polymera OS specific patterns
  parserPreset: {
    parserOpts: {
      issuePrefixes: ['#'],
      referenceActions: ['closes', 'fixes', 'resolves', 'relates'],
      noteKeywords: ['BREAKING CHANGE', 'BREAKING CHANGES']
    }
  },

  // Custom plugins for Polymera OS
  plugins: [
    // Add any custom plugins here
  ],

  // Ignore patterns for certain commits
  ignores: [
    // Ignore merge commits
    '^Merge branch',
    '^Merge pull request',
    '^Merge remote-tracking branch',

    // Ignore revert commits
    '^Revert',

    // Ignore WIP commits
    '^WIP:',
    '^wip:',

    // Ignore draft commits
    '^Draft:',
    '^draft:'
  ],

  // Custom help text for Polymera OS
  helpUrl: 'https://github.com/polymera-os/polymera-os/blob/main/CONTRIBUTING.md#commit-guidelines',

  // Default values for Polymera OS
  defaultIgnores: true,

  // Custom prompt for Polymera OS
  prompt: {
    questions: {
      type: {
        description: 'Select the type of change that you\'re committing:',
        enum: {
          feat: {
            description: '✨ A new feature for the user',
            title: 'Features',
            emoji: '✨'
          },
          fix: {
            description: '🐛 A bug fix for the user',
            title: 'Bug Fixes',
            emoji: '🐛'
          },
          docs: {
            description: '📚 Documentation only changes',
            title: 'Documentation',
            emoji: '📚'
          },
          style: {
            description: '💄 Changes that do not affect the meaning of the code',
            title: 'Styles',
            emoji: '💄'
          },
          refactor: {
            description: '♻️ Code change that neither fixes a bug nor adds a feature',
            title: 'Code Refactoring',
            emoji: '♻️'
          },
          perf: {
            description: '⚡ Code change that improves performance',
            title: 'Performance Improvements',
            emoji: '⚡'
          },
          test: {
            description: '🚨 Adding missing tests or correcting existing tests',
            title: 'Tests',
            emoji: '🚨'
          },
          chore: {
            description: '🔧 Changes to the build process or auxiliary tools',
            title: 'Chores',
            emoji: '🔧'
          }
        }
      },
      scope: {
        description: 'What is the scope of this change (e.g., kernel, crypto, services)?',
        enum: [
          'kernel',
          'crypto',
          'services',
          'runtime',
          'ui',
          'tooling',
          'ci',
          'docs',
          'build',
          'test',
          'security',
          'performance'
        ]
      },
      subject: {
        description: 'Write a short, imperative description of the change'
      },
      body: {
        description: 'Provide a longer description of the change'
      },
      isBreaking: {
        description: 'Are there any breaking changes?'
      },
      breakingBody: {
        description: 'A BREAKING CHANGE commit requires a body. Please enter a longer description of the commit itself'
      },
      breaking: {
        description: 'Describe the breaking changes'
      },
      isIssueAffected: {
        description: 'Does this change affect any open issues?'
      },
      issuesBody: {
        description: 'If issues are closed, the commit requires a body. Please enter a longer description of the commit itself'
      },
      issues: {
        description: 'Add issue references (e.g., "fix #123", "re #123")'
      }
    }
  }
};
