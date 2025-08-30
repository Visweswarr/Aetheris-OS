# Polymera OS Documentation Portal

This directory contains the complete documentation portal for Polymera OS, built with Docusaurus and featuring auto-generated API references, interactive diagrams, and comprehensive guides.

## 🏗️ Architecture

```
docs/
├── BUILD                      # Bazel build configuration
├── scripts/                   # Build and maintenance scripts
│   ├── dev-server.sh         # Development server launcher
│   ├── lint-docs.sh          # Documentation linting
│   └── test-links.sh         # Link validation
├── runbooks/                  # Troubleshooting and operational guides
│   ├── README.md             # Runbooks index and quick reference
│   └── kernel-debug.md       # Kernel debugging comprehensive guide
└── site/                     # Docusaurus site
    ├── package.json          # Node.js dependencies
    ├── docusaurus.config.js  # Site configuration
    ├── sidebars.js           # Navigation structure
    ├── docs/                 # Documentation content
    │   ├── intro.md          # Introduction page
    │   ├── getting-started/  # Installation and setup guides
    │   ├── concepts/         # Core concepts and principles
    │   ├── api/              # API reference documentation
    │   ├── architecture/     # System architecture docs
    │   ├── development/      # Development guides
    │   └── security/         # Security documentation
    ├── blog/                 # Blog posts and announcements
    ├── src/                  # React components and pages
    └── static/               # Static assets (images, files)
```

## 🚀 Quick Start

### Prerequisites

- Node.js 18+
- npm or yarn
- Bazel (for build integration)

### Development Server

Start the local development server:

```bash
# Using Bazel (recommended)
bazel run //docs:dev_server

# Using npm directly
cd docs/site
npm install
npm start
```

The site will be available at http://localhost:3000

### Building the Site

Build the complete documentation site:

```bash
# Build with Bazel
bazel build //docs:site

# Build with npm
cd docs/site
npm run build
```

## 📋 Features

### 🔄 Auto-Generated API Documentation

- **Protocol Buffer APIs**: Generated from `.proto` files
- **gRPC Services**: Service definitions and examples
- **REST APIs**: OpenAPI specification integration
- **Code Examples**: Multi-language client examples

### 📊 Interactive Diagrams

- **Mermaid Integration**: Architecture and sequence diagrams
- **System Topology**: Visual representation of components
- **Data Flow**: Process and data movement diagrams
- **Network Architecture**: Mesh networking visualization

### 🧭 Navigation & Discovery

- **Structured Navigation**: Logical organization of content
- **Full-Text Search**: Powered by Algolia (when configured)
- **Cross-References**: Automatic linking between docs
- **Version Support**: Multiple documentation versions

### 🎨 Enhanced User Experience

- **Dark/Light Themes**: Automatic theme switching
- **Mobile Responsive**: Optimized for all devices
- **Code Syntax Highlighting**: Multi-language support
- **Copy-to-Clipboard**: Easy code snippet copying

## 📝 Content Organization

### Getting Started
- **Installation Guide**: Step-by-step setup instructions
- **Quick Start**: 5-minute introduction to key features
- **Configuration**: System configuration options

### Core Concepts
- **Post-Quantum Cryptography**: NIST algorithms and hybrid schemes
- **Privacy Budgets**: Differential privacy and budget management
- **Session Keys**: Purpose-bound, time-scoped access control
- **Deterministic Computing**: Reproducible execution guarantees

### API Reference
- **Protocol Buffers**: Message definitions and service interfaces
- **gRPC Services**: Remote procedure call documentation
- **REST APIs**: HTTP endpoints and authentication
- **SDKs**: Language-specific client libraries

### Architecture
- **System Overview**: High-level component architecture
- **Security Design**: Zero-trust and post-quantum security
- **Network Architecture**: Mesh networking and DTN protocols
- **Storage Architecture**: Distributed and encrypted storage

### Development
- **Building from Source**: Compilation and build instructions
- **Testing Guide**: Unit, integration, and security testing
- **Contributing**: Development workflow and guidelines
- **Code Style**: Coding standards and best practices

### Runbooks & Troubleshooting
- **Kernel Debugging**: Comprehensive debugging guide for common issues
- **System Recovery**: Troubleshooting and recovery procedures
- **Performance Tuning**: Optimization and performance analysis

## 🔧 Build System Integration

### Bazel Targets

```bash
# Build documentation site
bazel build //docs:site

# Generate API documentation
bazel build //docs:generate_proto_docs

# Run development server
bazel run //docs:dev_server

# Test documentation
bazel test //docs:docs_tests

# Lint documentation
bazel test //docs:lint_docs

# Test links
bazel test //docs:test_links
```

### Generated Content

The build system automatically generates:

1. **API Documentation**: From Protocol Buffer definitions
2. **Service Documentation**: From gRPC service definitions
3. **Code Examples**: From source code annotations
4. **Navigation Structure**: From directory organization

### Versioning Strategy

- **Current (Next)**: Development version with latest features
- **Stable**: Latest released version
- **Legacy**: Previous major versions (maintained for compatibility)

Documentation versions are managed through:
- Git tags for releases
- Branch-based development
- Automated deployment on version tags

## 🧪 Quality Assurance

### Linting

Comprehensive documentation linting includes:

```bash
# Run all documentation checks
bazel test //docs:lint_docs
```

Checks include:
- Markdown syntax validation
- Heading structure verification
- Image alt-text requirements
- Link validity
- Line length guidelines
- Spell checking

### Link Testing

Automated link validation:

```bash
# Test internal links
bazel test //docs:test_links

# Test external links (slower)
bazel test //docs:test_links -- --external
```

### Accessibility

- **WCAG 2.1 AA Compliance**: Accessible design standards
- **Keyboard Navigation**: Full keyboard accessibility
- **Screen Reader Support**: Semantic HTML and ARIA labels
- **Color Contrast**: Sufficient contrast ratios

## 🚀 Deployment

### GitHub Pages

Automated deployment to GitHub Pages:

```bash
# Deploy to GitHub Pages
bazel run //docs:deploy
```

### Custom Hosting

The built site is a static website that can be hosted anywhere:

```bash
# Build for production
bazel build //docs:site

# Extract built site
tar -xzf bazel-bin/docs/polymera-docs.tar.gz -C /var/www/html/
```

### Docker Deployment

```dockerfile
FROM nginx:alpine
COPY bazel-bin/docs/polymera-docs.tar.gz /tmp/
RUN cd /usr/share/nginx/html && \
    tar -xzf /tmp/polymera-docs.tar.gz && \
    rm /tmp/polymera-docs.tar.gz
```

## 🤝 Contributing

### Documentation Guidelines

1. **Clarity**: Write for your audience - be clear and concise
2. **Structure**: Use consistent heading hierarchy and organization
3. **Examples**: Include practical, working code examples
4. **Links**: Cross-reference related content appropriately
5. **Images**: Use descriptive alt-text and optimize file sizes

### Content Types

- **Guides**: Step-by-step instructions for specific tasks
- **References**: Comprehensive API and configuration documentation
- **Tutorials**: Learning-oriented, hands-on content
- **Explanations**: Understanding-oriented, conceptual content

### Review Process

1. **Local Testing**: Run linting and link tests locally
2. **Preview**: Use development server to preview changes
3. **Pull Request**: Submit changes for review
4. **Automated Checks**: CI/CD validates documentation quality
5. **Deployment**: Automatic deployment on merge to main

## 📊 Analytics & Monitoring

### Usage Analytics

- **Page Views**: Most popular documentation sections
- **Search Queries**: Common search terms and failures
- **User Journeys**: How users navigate through content
- **Geographic Distribution**: Global usage patterns

### Performance Monitoring

- **Page Load Times**: Core Web Vitals tracking
- **Search Performance**: Search response times
- **Build Times**: Documentation generation performance
- **Link Health**: Automated broken link detection

## 🔒 Security Considerations

### Content Security

- **Input Sanitization**: All user-generated content is sanitized
- **XSS Protection**: Content Security Policy headers
- **HTTPS Only**: All external links use HTTPS
- **Dependency Scanning**: Regular security audits of dependencies

### Access Control

- **Read Access**: Public documentation with no authentication
- **Write Access**: GitHub-based authentication for contributors
- **Admin Access**: Repository maintainers only

## 📞 Support

### Getting Help

- **[GitHub Issues](https://github.com/polymera-os/polymera-os/issues)**: Bug reports and feature requests
- **[GitHub Discussions](https://github.com/polymera-os/polymera-os/discussions)**: Community Q&A
- **[Documentation Feedback](https://github.com/polymera-os/polymera-os/issues/new?template=documentation.md)**: Specific documentation issues

### Maintenance

The documentation portal is actively maintained with:

- **Weekly**: Automated link checking and dependency updates
- **Monthly**: Content review and optimization
- **Quarterly**: Major feature updates and redesigns
- **Annually**: Complete content audit and reorganization

---

This documentation portal serves as the primary source of truth for all Polymera OS information, from getting started guides to detailed API references. It's designed to grow with the project and provide an excellent developer experience.
