# Polymera OS Changelog Generator

Automated changelog generation from conventional commits with CI/CD integration.

## Features

- 📝 **Conventional Commits**: Parse and categorize commits following the [Conventional Commits](https://conventionalcommits.org/) specification
- 🔢 **Semantic Versioning**: Automatic version bump calculation (major/minor/patch)
- 📋 **Rich Changelogs**: Generate beautiful markdown changelogs with sections, contributors, and artifacts
- 🚀 **GitHub Integration**: Create GitHub releases with changelogs and artifact uploads
- 🔍 **Validation**: Validate commit message format and enforce conventional commit standards
- 🎯 **CI/CD Ready**: Seamless integration with GitHub Actions workflows
- ⚙️ **Configurable**: Customizable commit types, scopes, and formatting options

## Installation

```bash
cd tooling/changelog
npm install
```

## Usage

### CLI Commands

```bash
# Generate changelog for version range
node gen.ts generate [from-tag] [to-tag] [output-file]

# Generate full changelog from all tags
node gen.ts full [output-file]

# Calculate version bump based on commits
node gen.ts bump [from-tag] [to-tag]

# Create GitHub release with artifacts
node gen.ts release [from-tag] [version] [--draft] [--prerelease]

# Validate conventional commit format
node gen.ts validate [from-tag] [to-tag]
```

### Examples

```bash
# Generate changelog between two tags
node gen.ts generate v1.0.0 v1.1.0 CHANGELOG.md

# Generate full changelog
node gen.ts full

# Calculate next version
node gen.ts bump v1.0.0

# Create GitHub release
node gen.ts release v1.0.0 1.1.0

# Validate commits since last tag
node gen.ts validate v1.0.0
```

## CI/CD Integration

### GitHub Actions

The changelog generator integrates seamlessly with GitHub Actions. See `.github/workflows/release.yml` for the complete workflow.

**Trigger on Tag Push:**
```yaml
on:
  push:
    tags:
      - 'v*.*.*'
```

**Generate Changelog:**
```yaml
- name: Generate changelog
  run: |
    cd tooling/changelog
    npm install
    node gen.ts generate ${{ needs.validate-commits.outputs.previous-tag }} ${{ github.ref_name }} "../../RELEASE_CHANGELOG.md"
```

**Version Bump Calculation:**
```yaml
- name: Calculate version bump
  id: bump
  run: |
    cd tooling/changelog
    npm install
    node gen.ts bump ${{ steps.get-previous-tag.outputs.tag }} HEAD
  env:
    GITHUB_OUTPUT: ${{ github.output }}
```

## Configuration

The generator can be customized with various options:

```typescript
const config = {
  types: {
    feat: { title: '🚀 Features', order: 1, includeInChangelog: true },
    fix: { title: '🐛 Bug Fixes', order: 2, includeInChangelog: true },
    perf: { title: '⚡ Performance', order: 3, includeInChangelog: true },
    // ... more types
  },
  scopes: {
    kernel: 'Kernel',
    services: 'Services',
    ui: 'User Interface',
    // ... more scopes
  },
  excludeScopes: ['internal', 'temp'],
  githubRepo: 'polymera-os/polymera-os',
  artifactPaths: ['dist/*.tar.gz', 'dist/*.zip'],
};
```

## Conventional Commit Types

The generator recognizes the following commit types:

| Type | Description | Changelog | Version Bump |
|------|-------------|-----------|--------------|
| `feat` | New feature | ✅ | Minor |
| `fix` | Bug fix | ✅ | Patch |
| `perf` | Performance improvement | ✅ | Patch |
| `refactor` | Code refactoring | ✅ | Patch |
| `docs` | Documentation | ✅ | Patch |
| `test` | Tests | ❌ | - |
| `build` | Build system | ✅ | Patch |
| `ci` | CI/CD | ❌ | - |
| `chore` | Maintenance | ❌ | - |
| `style` | Code style | ❌ | - |
| `revert` | Revert changes | ✅ | Patch |

**Breaking Changes:** Any commit with `!` after the type or `BREAKING CHANGE:` in the body triggers a major version bump.

## Example Output

### Generated Changelog

```markdown
## [1.2.0] - 2024-01-15

*4 commits, 2 features, 1 fix*

### ⚠️ BREAKING CHANGES

- **core**: redesign plugin architecture ([a1b2c3d](https://github.com/polymera-os/polymera-os/commit/a1b2c3d))
  Plugin interface has changed. Old plugins need to be updated to use the new API.

### 🚀 Features

- **ui**: add new dashboard component ([#42](https://github.com/polymera-os/polymera-os/pull/42)) ([d4e5f6g](https://github.com/polymera-os/polymera-os/commit/d4e5f6g))
- **api**: implement user authentication ([g7h8i9j](https://github.com/polymera-os/polymera-os/commit/g7h8i9j))

### 🐛 Bug Fixes

- **api**: resolve authentication timeout issue ([#123](https://github.com/polymera-os/polymera-os/pull/123)) ([j9k0l1m](https://github.com/polymera-os/polymera-os/commit/j9k0l1m))

### 👥 Contributors

- Alice Developer
- Bob Maintainer
- Charlie Reviewer
```

### Version Bump Analysis

```bash
📈 Version Bump Analysis:
   Current: 1.1.0
   Next: 2.0.0
   Type: major
   Reason: Breaking changes detected
```

## Testing

Run the comprehensive test suite:

```bash
npm test
```

The tests cover:
- ✅ Conventional commit parsing
- ✅ Version bump calculation
- ✅ Changelog generation
- ✅ GitHub release creation
- ✅ CI workflow simulation
- ✅ Edge cases and error handling

### Test Coverage

- **Conventional Commit Parsing**: Valid/invalid formats, breaking changes, PR extraction
- **Version Bumping**: Major/minor/patch logic, priority handling
- **Changelog Generation**: Multi-release, contributors, artifacts, filtering
- **Release Creation**: GitHub integration, draft/prerelease flags
- **Validation**: Format checking, error reporting
- **CI Integration**: Tag triggers, artifact collection, workflow dispatch

## Release Workflow

### Automatic Release (Tag Push)

1. **Push tag**: `git tag v1.2.0 && git push origin v1.2.0`
2. **CI triggers**: GitHub Actions workflow runs
3. **Validation**: Commits validated for conventional format
4. **Build**: Artifacts built for multiple platforms
5. **Changelog**: Generated from commits since last tag
6. **Release**: GitHub release created with changelog and artifacts
7. **Deploy**: Documentation deployed to GitHub Pages

### Manual Release (Workflow Dispatch)

1. **Trigger**: Use GitHub Actions "Run workflow" button
2. **Input**: Version number, pre-release flag, draft flag
3. **Analysis**: Version bump calculated and displayed
4. **Confirmation**: Manual approval required
5. **Execution**: Same as automatic release

## Artifacts

The release workflow automatically collects and attaches artifacts:

- **Platform Binaries**: Linux (x64, ARM64), macOS (x64, ARM64), Windows (x64)
- **Container Images**: Multi-platform Docker images (linux/amd64, linux/arm64)
- **Documentation**: Static site build
- **Checksums**: SHA256 checksums for all binaries
- **Source Code**: Automatic GitHub source archives

## Best Practices

### Commit Messages

```bash
# ✅ Good
feat(ui): add user dashboard with charts
fix(api): resolve authentication timeout issue
docs: update installation instructions
feat!: redesign plugin architecture

# ❌ Bad
Add dashboard
Fix bug
Update docs
Change API
```

### Scopes

Use consistent scopes to categorize changes:

- `kernel`: Core kernel changes
- `services`: Service layer modifications  
- `ui`: User interface updates
- `docs`: Documentation changes
- `ci`: CI/CD pipeline updates
- `deps`: Dependency updates

### Breaking Changes

Mark breaking changes clearly:

```bash
feat(api)!: redesign authentication system

BREAKING CHANGE: The authentication API has been redesigned.
Previous token format is no longer supported. Users must
re-authenticate using the new OAuth2 flow.
```

## Troubleshooting

### Common Issues

**1. "Not in a git repository"**
```bash
# Ensure you're in a git repository
git init
```

**2. "No tags found"**
```bash
# Create an initial tag
git tag v0.1.0
```

**3. "Invalid conventional commit format"**
```bash
# Follow conventional commit format
git commit -m "feat: add new feature"
```

**4. "Artifacts not found"**
```bash
# Ensure artifacts exist in expected paths
ls -la dist/
```

### Debug Mode

Enable verbose logging:

```bash
DEBUG=1 node gen.ts generate
```

## Development

### Building

```bash
npm run build
```

### Testing

```bash
npm test
npm run test:watch
```

### Linting

```bash
npm run lint
npm run format
```

## License

Apache-2.0 - see [LICENSE](../../LICENSE) for details.

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines.
