# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Improved error handling in watch service - replaced unwraps with proper Result propagation
- watch() and watch_directory() functions now return Result<()> for better error handling
- Enhanced CI/CD: Added macOS to cross-platform test matrix
- Updated GitHub Actions: Migrated from deprecated actions-rs/toolchain to dtolnay/rust-toolchain

### Added
- FILE_SETTLE_DURATION constant for file modification debounce delay
- CONTRIBUTING.md with comprehensive contribution guidelines
- CHANGELOG.md to track version history
- Better error context messages throughout the codebase

### Fixed
- Removed panic-prone unwrap() calls in terminal raw mode operations
- Improved graceful error handling for keyboard and file system events
- Fixed potential crashes in watch loop when terminal mode fails

## [0.1.3] - 2024-XX-XX

### Changed
- Switched documentation from Jekyll to MkDocs
- Updated README with improved structure

### Fixed
- Various bug fixes and improvements

## [0.1.2] - 2024-XX-XX

### Added
- Cross-platform support improvements

## [0.1.1] - 2024-XX-XX

### Fixed
- Installation script improvements

## [0.1.0] - 2024-XX-XX

### Added
- Initial release
- File watching and conversion functionality
- USB drive detection for Linux, macOS, and Windows
- Machine database with format support
- Inkscape integration for file conversion
- Configuration management with TOML
- Interactive machine selection with fuzzy matching
- Command-line interface with multiple commands
- Self-update functionality
- Cross-platform installation scripts

[Unreleased]: https://github.com/osteele/stitch-sync/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/osteele/stitch-sync/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/osteele/stitch-sync/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/osteele/stitch-sync/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/osteele/stitch-sync/releases/tag/v0.1.0
