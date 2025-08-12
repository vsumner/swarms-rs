# GitHub Workflows Documentation

This directory contains the CI/CD workflows for swarms-rs. Each workflow serves a specific purpose in maintaining code quality, security, and reliability.

## Workflows Overview

### 🔧 Core CI/CD Workflows

#### `ci.yml` - Continuous Integration
**Triggers:** Push to `main`/`develop`, PRs, daily schedule
- **Multi-platform testing** (Ubuntu, Windows, macOS)
- **Multi-Rust version testing** (stable, beta, nightly)
- **Comprehensive test suite** (unit, integration, doc tests)
- **Example compilation checks**
- **Memory safety testing with Miri**
- **Code coverage reporting**
- **Minimal versions compatibility**

#### `release.yml` - Release Automation
**Triggers:** Version tags (`v*.*.*`), manual dispatch
- **Version validation** and format checking
- **Multi-platform binary builds**
- **Automated crates.io publishing**
- **GitHub release creation** with changelogs
- **Docker image building and publishing**
- **Release artifact management**

#### `security.yml` - Security Auditing
**Triggers:** Push, PRs, daily schedule
- **Dependency vulnerability scanning** (`cargo-audit`)
- **License compliance checking** (`cargo-deny`)
- **Supply chain security** (`cargo-vet`)
- **Security advisory monitoring**

### 📊 Quality Assurance Workflows

#### `format.yml` - Code Quality
**Triggers:** Push to `main`/`develop`, PRs
- **Code formatting** (`rustfmt`)
- **Linting** (`clippy` with pedantic rules)
- **Documentation checks** (warnings as errors)
- **Unused dependency detection** (`cargo-machete`)
- **SemVer compatibility** (`cargo-semver-checks`)
- **MSRV validation**
- **Typo detection**

#### `performance.yml` - Performance Monitoring
**Triggers:** Push to `main`, PRs, weekly schedule
- **Benchmark comparison** for PRs
- **Memory profiling** with Valgrind
- **CPU profiling** with perf
- **Binary size tracking** (`cargo-bloat`)

#### `docs.yml` - Documentation
**Triggers:** Push to `main`, PRs, daily schedule
- **Documentation building** with custom styling
- **GitHub Pages deployment**
- **Link validation**
- **Documentation coverage analysis**
- **Examples documentation generation**

### 🛠️ Maintenance Workflows

#### `dependency-update.yml` - Dependency Management
**Triggers:** Weekly schedule, manual dispatch
- **Automated dependency updates**
- **Security advisory checking**
- **Rust toolchain updates**
- **GitHub Actions version updates**
- **Automated PR creation**

## Workflow Configuration

### Required Secrets

Add these secrets to your repository settings:

```bash
# Required for releases
CARGO_REGISTRY_TOKEN    # crates.io API token
DOCKER_USERNAME         # Docker Hub username
DOCKER_PASSWORD         # Docker Hub password

# Optional for enhanced features
CODECOV_TOKEN          # Codecov integration
```

### Required Permissions

The workflows require the following permissions:

- **Repository permissions:** Read/Write
- **Actions permissions:** Write
- **Pages permissions:** Write (for documentation deployment)
- **Pull requests permissions:** Write (for automated PRs)

## Branch Protection Rules

Recommended branch protection rules for `main`:

```yaml
Required status checks:
  - Test Suite (ubuntu-latest, stable)
  - Test Suite (windows-latest, stable)
  - Test Suite (macos-latest, stable)
  - Code Formatting
  - Clippy Lints
  - Security Audit

Require branches to be up to date: true
Restrict pushes to matching branches: true
```

## Badge Integration

Add these badges to your README.md:

```markdown
[![CI](https://github.com/The-Swarm-Corporation/swarms-rs/workflows/CI/badge.svg)](https://github.com/The-Swarm-Corporation/swarms-rs/actions/workflows/ci.yml)
[![Security](https://github.com/The-Swarm-Corporation/swarms-rs/workflows/Security%20Audit/badge.svg)](https://github.com/The-Swarm-Corporation/swarms-rs/actions/workflows/security.yml)
[![Docs](https://github.com/The-Swarm-Corporation/swarms-rs/workflows/Documentation/badge.svg)](https://github.com/The-Swarm-Corporation/swarms-rs/actions/workflows/docs.yml)
[![codecov](https://codecov.io/gh/The-Swarm-Corporation/swarms-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/The-Swarm-Corporation/swarms-rs)
```

## Workflow Scheduling

| Workflow | Frequency | Time (UTC) | Purpose |
|----------|-----------|------------|---------|
| CI | On push/PR | - | Immediate feedback |
| Security | Daily | 2:00 AM | Security monitoring |
| Documentation | Daily | 4:00 AM | Keep docs fresh |
| Dependency Updates | Weekly | Monday 9:00 AM | Maintenance |
| Performance | Weekly | Sunday 3:00 AM | Performance tracking |

## Customization

### Adding New Workflows

1. Create a new `.yml` file in `.github/workflows/`
2. Follow the existing naming convention
3. Include appropriate triggers and permissions
4. Add documentation here

### Modifying Existing Workflows

1. Test changes in a feature branch first
2. Consider backward compatibility
3. Update this documentation
4. Test with a draft release if changing release workflows

### Performance Considerations

- Workflows use caching extensively to reduce build times
- Parallel job execution where possible
- Conditional execution based on file changes
- Artifact cleanup with appropriate retention periods

## Troubleshooting

### Common Issues

1. **Failed tests on specific platforms:** Check platform-specific dependencies
2. **Security audit failures:** Update vulnerable dependencies
3. **Documentation build failures:** Check for broken links or missing files
4. **Release failures:** Verify version consistency and secrets

### Debugging Tips

1. Check workflow logs in the Actions tab
2. Verify all required secrets are set
3. Ensure branch protection rules aren't blocking automated PRs
4. Check file permissions for executable scripts

## Future Enhancements

Planned improvements:

- [ ] Integration with external security scanning tools
- [ ] Automated performance regression detection
- [ ] Integration with code quality services
- [ ] Enhanced notification systems
- [ ] Multi-architecture Docker builds
- [ ] Automated changelog generation
