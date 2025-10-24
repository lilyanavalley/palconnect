# Release Process for PalConnect

This repository uses GitHub Actions to automatically build and publish releases with `cargo-packager` integration.

## Automated Release Process

### 1. Automatic Release on Tag Push

When you push a tag matching the pattern `v*.*.*` (e.g., `v1.0.0`, `v1.2.3`), the following happens automatically:

1. **Build**: The `build.yaml` workflow runs tests and builds packages for all platforms (Windows `.msi`, macOS `.dmg`, Linux `.deb`)
2. **Package**: Uses `cargo-packager` to create platform-specific installers
3. **Release**: Creates a GitHub release with auto-generated release notes
4. **Upload**: Attaches all platform packages to the release
5. **Update Config**: Creates/updates the auto-updater configuration for `cargo-packager-updater`

### 2. Manual Release Trigger

You can also trigger a release manually:

1. Go to the **Actions** tab in your GitHub repository
2. Select **Manual Release** workflow
3. Click **Run workflow**
4. Enter the version number (e.g., `0.1.1`)
5. Choose whether it's a pre-release
6. Click **Run workflow**

## Version Management

### Updating Version Before Release

Before creating a release, update the version in `Cargo.toml`:

```toml
[package]
version = "0.1.1"  # Update this version
```

### Semantic Versioning

Follow [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

## Auto-Updates

The release system is configured to work with `cargo-packager-updater`:

- Each release creates an updater configuration file
- Clients can check for updates automatically
- Users get notified about new versions within the app

## Files Created by Release Process

```text
.updater/
├── latest.json          # Auto-updater configuration
└── signatures/          # Cryptographic signatures (if enabled)

target/release/dist/
├── palconnect.deb       # Linux package
├── palconnect.dmg       # macOS package
└── palconnect.msi       # Windows package
```

## Example: Creating a New Release

```bash
# 1. Update version in Cargo.toml
vim Cargo.toml

# 2. Commit your changes
git add .
git commit -m "Release v0.1.1"

# 3. Create and push tag
git tag -a v0.1.1 -m "Release v0.1.1"
git push origin v0.1.1

# 4. GitHub Actions automatically handles the rest!
```

## Troubleshooting

### Common Issues

1. **Build Fails**: Check that all dependencies are properly listed in `Cargo.toml`
2. **Package Fails**: Ensure `cargo-packager` configuration is valid
3. **Upload Fails**: Verify GitHub token permissions include `contents: write`

### Logs

Check the **Actions** tab for detailed build logs and error messages.
