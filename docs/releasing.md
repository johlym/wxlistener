# Release Process

This project uses automated releases via GitHub Actions.

## Table of Contents

- [How It Works](#how-it-works)
- [Release Notes](#release-notes)
- [Creating a New Release](#creating-a-new-release)
- [Release Artifacts](#release-artifacts)
- [Versioning](#versioning)
- [Manual Release (if needed)](#manual-release-if-needed)
- [Troubleshooting](#troubleshooting)
  - [Release workflow didn't trigger](#release-workflow-didnt-trigger)
  - [Build failed for a specific platform](#build-failed-for-a-specific-platform)
  - [Release was created but binaries are missing](#release-was-created-but-binaries-are-missing)

## How It Works

The release workflow runs on every push to the `main` branch. When the `version` field in `Cargo.toml` has changed, it automatically:

1. Detects the version change
2. Creates a new Git tag (e.g., `v0.1.2`)
3. Generates release notes from all commits since the last tag
4. Creates a GitHub release with that tag and the generated notes
5. Builds binaries for multiple platforms:
   - Linux (x86_64, ARM64)
   - macOS (Intel x86_64, Apple Silicon ARM64)
   - Windows (x86_64)
6. Uploads the compiled binaries as release assets

## Release Notes

The release notes are automatically generated and include:

- **What's Changed**: All commit messages since the last release tag
- **Installation**: Instructions for downloading and using the binaries
- **Available Platforms**: List of all platform-specific binaries
- **Full Changelog**: Link to the detailed CHANGELOG.md file

Each commit message will appear as a bullet point in the release notes with its short hash.

## Creating a New Release

1. Update the version in `Cargo.toml`:

   ```toml
   [package]
   version = "0.1.3"  # Increment as appropriate
   ```

2. Update `CHANGELOG.md` with the changes in this release:

   ```markdown
   ## [0.1.3] - 2025-12-10

   ### Added

   - New feature description

   ### Changed

   - Changed feature description

   ### Fixed

   - Bug fix description
   ```

3. Commit and push to `main`:

   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Bump version to 0.1.3"
   git push origin main
   ```

4. The GitHub Action will automatically:
   - Detect the version change
   - Build binaries for all platforms
   - Create a release with all binaries attached

## Release Artifacts

Each release includes the following downloadable files:

- `wxlistener-linux-x86_64.tar.gz` - Linux 64-bit (Intel/AMD)
- `wxlistener-linux-aarch64.tar.gz` - Linux 64-bit (ARM)
- `wxlistener-macos-x86_64.tar.gz` - macOS Intel
- `wxlistener-macos-aarch64.tar.gz` - macOS Apple Silicon
- `wxlistener-windows-x86_64.exe.zip` - Windows 64-bit

## Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version (x.0.0) - Incompatible API changes
- **MINOR** version (0.x.0) - New functionality in a backwards compatible manner
- **PATCH** version (0.0.x) - Backwards compatible bug fixes

## Manual Release (if needed)

If you need to create a release manually:

1. Create and push a tag:

   ```bash
   git tag v0.1.3
   git push origin v0.1.3
   ```

2. Go to GitHub → Releases → Draft a new release
3. Select the tag you just created
4. Add release notes
5. Manually build and upload binaries (not recommended)

## Troubleshooting

### Release workflow didn't trigger

- Ensure the commit changed `Cargo.toml` specifically the `version` field
- Check that you pushed to the `main` branch
- Verify the workflow file exists at `.github/workflows/release.yml`

### Build failed for a specific platform

- Check the Actions tab on GitHub for error logs
- Common issues:
  - Missing dependencies for cross-compilation
  - Platform-specific code issues
  - Cargo.lock conflicts

### Release was created but binaries are missing

- Check the build job logs in GitHub Actions
- Ensure all build jobs completed successfully
- Verify the upload step didn't fail due to network issues
