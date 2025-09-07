# Contributing to Aetheris OS

Thank you for your interest in contributing to Aetheris OS! This document provides guidelines and information for contributors.

## Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a feature branch** from `main`
4. **Make your changes** following our coding standards
5. **Test your changes** thoroughly
6. **Submit a pull request** with a clear description

## Development Setup

### Prerequisites

- **Rust 1.79.0+** (see `rust-toolchain.toml`)
- **Go 1.22+** (see `go.mod` files)
- **Node.js 20+** (see `.nvmrc`)
- **Python 3.11+** (see `.python-version`)
- **Git LFS** for large assets

### Bootstrap

```bash
# Clone and setup
git clone https://github.com/Visweswarr/Aetheris-OS.git
cd Aetheris-OS
git lfs install

# Bootstrap the environment
make bootstrap

# Verify Phase 4 readiness
bash scripts/phase4-verify.sh
```

## Coding Standards

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

feat(xr): add multi-user synchronization
fix(hal): resolve GPIO timing issue
docs(ai): update model documentation
```

### Code Style

- **Rust**: Use `cargo fmt` and `cargo clippy`
- **Go**: Use `gofmt` and `golangci-lint`
- **TypeScript**: Use `prettier` and `eslint`
- **Python**: Use `black` and `ruff`

### Testing

- **Unit tests**: Required for all new features
- **Integration tests**: For cross-component functionality
- **Performance tests**: For critical paths
- **Deterministic tests**: For AI/ML components

## Pull Request Process

### Before Submitting

1. **Run tests**: `make test`
2. **Check formatting**: `make fmt`
3. **Run linting**: `make lint`
4. **Verify Phase 4**: `bash scripts/phase4-verify.sh`

### PR Requirements

- **Clear title**: Use conventional commit format
- **Detailed description**: What, why, and how
- **Tests included**: Unit and integration tests
- **Documentation updated**: If applicable
- **Performance impact**: Documented if significant

### Review Process

1. **Automated checks** must pass
2. **Code review** by maintainers
3. **Security review** for sensitive changes
4. **Performance review** for critical paths

## Architecture Guidelines

### Phase 4 Components

- **XR**: Extended Reality rendering and synchronization
- **HAL**: Hardware Abstraction Layer
- **AI**: Machine Learning inference and training
- **Contracts**: Web3 smart contracts
- **Relay**: Network communication layer

### Design Principles

- **Deterministic**: Reproducible results across runs
- **Performant**: Meet latency and throughput targets
- **Secure**: Defense in depth
- **Modular**: Clear separation of concerns
- **Testable**: Comprehensive test coverage

## Security

### Reporting Vulnerabilities

Please report security vulnerabilities privately:

1. **Email**: security@aetheris-os.org
2. **Include**: Detailed description and reproduction steps
3. **Do not**: Open public issues for security vulnerabilities

### Security Guidelines

- **Input validation**: Sanitize all inputs
- **Authentication**: Use strong authentication
- **Authorization**: Principle of least privilege
- **Encryption**: Use industry-standard algorithms
- **Secrets**: Never commit secrets to version control

## Performance

### Benchmarks

We maintain performance baselines in `artifacts/bench/`:

- **XR**: 90 FPS target, <11ms latency
- **HAL**: <1μs GPIO operations
- **AI**: <100ms inference latency

### Performance Guidelines

- **Profile first**: Measure before optimizing
- **Cache appropriately**: Balance memory and speed
- **Avoid allocations**: In hot paths
- **Use async**: For I/O operations
- **Monitor regressions**: CI gates prevent performance degradation

## Documentation

### Required Documentation

- **API docs**: For all public interfaces
- **Architecture docs**: For system design
- **User guides**: For end-user features
- **Developer guides**: For contributor workflows

### Documentation Standards

- **Clear and concise**: Easy to understand
- **Examples included**: Show usage patterns
- **Up to date**: Keep current with code
- **Searchable**: Use consistent terminology

## Community

### Getting Help

- **GitHub Discussions**: For questions and ideas
- **Discord**: For real-time chat
- **Matrix**: For decentralized communication
- **GitHub Issues**: For bug reports and feature requests

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- **Be respectful**: Treat everyone with dignity
- **Be constructive**: Focus on improving the project
- **Be patient**: Remember we're all volunteers
- **Be inclusive**: Welcome newcomers and diverse perspectives

## License

By contributing to Aetheris OS, you agree that your contributions will be licensed under the same license as the project.

## Questions?

If you have questions about contributing, please:

1. Check existing documentation
2. Search GitHub Issues and Discussions
3. Ask in Discord or Matrix
4. Open a new issue if needed

Thank you for contributing to Aetheris OS! 🚀