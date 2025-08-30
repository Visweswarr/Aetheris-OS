# Contributing to Polymera OS

Thank you for your interest in contributing to Polymera OS! This document provides comprehensive guidelines for contributing to our next-generation quantum-ready operating system.

## 🚀 Quick Start

1. **Fork** the repository
2. **Clone** your fork locally
3. **Create** a feature branch from `develop`
4. **Make** your changes following our standards
5. **Test** your changes thoroughly
6. **Commit** using conventional commit format
7. **Push** and create a pull request
8. **Wait** for review and address feedback

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Project Overview](#project-overview)
- [Development Setup](#development-setup)
- [Contribution Workflow](#contribution-workflow)
- [Code Standards](#code-standards)
- [Testing Requirements](#testing-requirements)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Review Process](#review-process)
- [Release Process](#release-process)
- [Community Guidelines](#community-guidelines)

## 📜 Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## 🎯 Project Overview

Polymera OS is a next-generation operating system designed for the quantum era, built with:
- **Security First**: Post-quantum cryptography and zero-knowledge proofs
- **Deterministic Performance**: Predictable, bounded latency with SLOs
- **Verifiable Computing**: Cryptographic attestation and reproducible builds
- **Privacy by Design**: Zero-knowledge privacy guarantees

### Technology Stack Requirements

- **Kernel/Systems/Crypto/Net**: Must be written in **Rust**
- **Graphics Paths**: Must be written in **C++**
- **Web/Desktop UI**: Must be written in **TypeScript**
- **Agents/Tooling**: Must be written in **Python**

### Mandatory Technologies

- **Cryptography**: Kyber/Dilithium hybrids
- **Zero-Knowledge**: Noir/Halo2 for ZK flows
- **Policy Engine**: OPA/Rego→WASM for policy
- **Networking**: libp2p/QUIC for mesh networking
- **XR Support**: OpenXR+Vulkan for extended reality

## 🛠️ Development Setup

### Prerequisites

- **Rust**: 1.75+ (stable and nightly)
- **Bazel**: 7.0+
- **Nix**: Package manager
- **QEMU**: For emulation testing
- **Git**: Latest version

### Environment Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/polymera-os/polymera-os.git
   cd polymera-os
   ```

2. **Enter development environment**:
   ```bash
   nix-shell
   ```

3. **Verify setup**:
   ```bash
   bazel build //kernel:all
   cargo test
   ```

### Development Container

We provide a VS Code dev container with all required tools:

1. **Install VS Code** and the Dev Containers extension
2. **Open the repository** in VS Code
3. **Reopen in Container** when prompted
4. **Wait** for the container to build and start

## 🔄 Contribution Workflow

### Branch Strategy

- **`main`**: Production-ready releases only
- **`develop`**: Integration branch for all features
- **`feature/*`**: Feature development branches
- **`hotfix/*`**: Critical bug fixes for production
- **`release/*`**: Release preparation branches

### Feature Development

1. **Start from develop**:
   ```bash
   git checkout develop
   git pull origin develop
   ```

2. **Create feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes** following our standards

4. **Test thoroughly**:
   ```bash
   cargo test
   bazel test //...
   ```

5. **Commit with conventional format**:
   ```bash
   git commit -m "feat(kernel): implement basic process management"
   ```

6. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   # Create PR on GitHub
   ```

## 📝 Code Standards

### Language-Specific Standards

#### Rust (Kernel, Systems, Crypto, Net)
- Use `rustfmt` with project-specific rules
- Follow Rust naming conventions
- Use `clippy` with strict warnings enabled
- Document all public APIs
- Write comprehensive tests

#### C++ (Graphics)
- Use `clang-format` with project rules
- Follow modern C++ standards (C++20)
- Use `clang-tidy` for static analysis
- Document all public interfaces
- Write unit tests for all components

#### TypeScript (Web/Desktop UI)
- Use `prettier` with project rules
- Follow TypeScript best practices
- Use strict type checking
- Write unit tests with Jest
- Document component APIs

#### Python (Agents/Tooling)
- Use `black` with project rules
- Follow PEP 8 guidelines
- Use type hints
- Write unit tests with pytest
- Document all functions and classes

### General Standards

- **Documentation**: All code must be documented
- **Testing**: >90% code coverage required
- **Error Handling**: Comprehensive error handling
- **Security**: Follow security best practices
- **Performance**: Meet specified SLOs

## 🧪 Testing Requirements

### Test Categories

- **Unit Tests**: Individual component testing
- **Integration Tests**: Component interaction testing
- **Performance Tests**: SLO compliance testing
- **Security Tests**: Security validation testing
- **Fuzz Tests**: Where applicable (crypto, parsing)

### Test Coverage

- **Minimum Coverage**: >90% for all components
- **Coverage Reporting**: Automated in CI pipeline
- **Coverage Enforcement**: PRs blocked if coverage drops

### Running Tests

```bash
# Run all tests
cargo test
bazel test //...

# Run specific test categories
cargo test --test integration
bazel test //kernel:unit_tests

# Run with coverage
cargo tarpaulin
```

## 📝 Commit Guidelines

### Conventional Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Commit Types

- **`feat`**: New feature for the user
- **`fix`**: Bug fix for the user
- **`docs`**: Documentation only changes
- **`style`**: Changes that do not affect code meaning
- **`refactor`**: Code change that neither fixes a bug nor adds a feature
- **`perf`**: Code change that improves performance
- **`test`**: Adding missing tests or correcting existing tests
- **`chore`**: Changes to the build process or auxiliary tools

### Commit Scopes

- **`kernel`**: Kernel-related changes
- **`crypto`**: Cryptographic implementations
- **`services`**: Service layer changes
- **`runtime`**: Runtime layer changes
- **`ui`**: User interface changes
- **`tooling`**: Build and development tools
- **`ci`**: CI/CD pipeline changes
- **`docs`**: Documentation changes

### Commit Examples

```bash
feat(kernel): implement basic process management

feat(crypto): add CRYSTALS-Kyber implementation

fix(services): resolve memory leak in PolyNet

docs(api): add comprehensive API documentation

chore(ci): update GitHub Actions workflow
```

## 🔀 Pull Request Process

### PR Requirements

- **Description**: Clear description of changes
- **Related Issues**: Link to related issues
- **Testing**: Describe testing performed
- **Documentation**: Document any new features
- **Breaking Changes**: Note any breaking changes
- **Checklist**: Complete all required checks

### PR Template

Use our [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md) which includes:

- Change description
- Related issues
- Testing performed
- Documentation updates
- Breaking changes
- Checklist for requirements

### PR Validation

All PRs must pass:

- [ ] **Code Review**: At least one maintainer approval
- [ ] **CI Checks**: All automated tests passing
- [ ] **Coverage**: Test coverage maintained or improved
- [ ] **Documentation**: Documentation updated
- [ ] **Conventional Commits**: Commit messages follow format
- [ ] **License Headers**: All source files have headers

## 👀 Review Process

### Review Requirements

- **All Changes**: Every change requires code review
- **Maintainer Approval**: At least one maintainer must approve
- **CI Passing**: All CI checks must pass
- **Documentation**: Code changes must include documentation updates

### Review Guidelines

- **Be Respectful**: Provide constructive feedback
- **Be Specific**: Point to specific issues
- **Be Helpful**: Suggest improvements
- **Be Timely**: Respond within 48 hours

### Review Checklist

- [ ] **Code Quality**: Follows project standards
- [ ] **Functionality**: Implements requirements correctly
- [ ] **Testing**: Adequate test coverage
- [ ] **Documentation**: Clear and complete
- [ ] **Performance**: Meets SLO requirements
- [ ] **Security**: No security issues introduced

## 🚀 Release Process

### Release Schedule

- **Monthly Releases**: First Monday of each month
- **Hotfix Releases**: Critical security or bug fixes as needed
- **Versioning**: Semantic versioning (MAJOR.MINOR.PATCH)

### Release Process

1. **Feature Freeze**: 1 week before release
2. **Testing**: Comprehensive testing of release candidate
3. **Release Notes**: Generate changelog from conventional commits
4. **Release**: Tag and publish release
5. **Announcement**: Community announcement and documentation update

### Release Checklist

- [ ] **All Tests Passing**: CI pipeline green
- [ ] **Documentation Updated**: All changes documented
- [ ] **Release Notes**: Changelog generated
- [ ] **Version Tagged**: Git tag created
- [ ] **Artifacts Published**: Release artifacts available
- [ ] **Community Notified**: Release announcement posted

## 🤝 Community Guidelines

### Communication

- **Be Respectful**: Treat all contributors with respect
- **Be Inclusive**: Welcome contributors from all backgrounds
- **Be Helpful**: Help newcomers and answer questions
- **Be Patient**: Development takes time

### Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Discord**: For real-time community chat
- **Email**: For security issues (security@polymera-os.org)

### Recognition

- **Contributors**: All contributors recognized in CONTRIBUTORS.md
- **Maintainers**: Active contributors may become maintainers
- **Hall of Fame**: Special recognition for significant contributions

## 📚 Additional Resources

- **Project Documentation**: [docs.polymera-os.org](https://docs.polymera-os.org)
- **Architecture Guide**: [DESIGN.md](DESIGN.md)
- **Development Tasks**: [TASKS.md](TASKS.md)
- **CI Policies**: [CI_POLICIES.md](CI_POLICIES.md)
- **Security Policy**: [SECURITY.md](SECURITY.md)

## 🆘 Need Help?

If you need help with contributing:

1. **Check Documentation**: Review this guide and project docs
2. **Search Issues**: Look for similar questions in GitHub issues
3. **Ask Community**: Post in GitHub discussions or Discord
4. **Contact Maintainers**: Reach out to project maintainers

---

*Thank you for contributing to Polymera OS! Your contributions help build a more secure and performant computing future.*
