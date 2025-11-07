# Contributing to Stitch-sync

Thank you for your interest in contributing to Stitch-sync! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please be respectful and constructive in all interactions.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue on GitHub with:
- A clear, descriptive title
- Detailed steps to reproduce the issue
- Expected behavior vs. actual behavior
- Your environment (OS, Rust version, Inkscape version)
- Any relevant error messages or logs

### Suggesting Features

Feature suggestions are welcome! Please create an issue with:
- A clear description of the feature
- The use case and why it would be valuable
- Any relevant examples from similar tools

### Contributing Code

1. **Fork the repository** and create a new branch from `main`
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Set up your development environment**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Clone your fork
   git clone https://github.com/YOUR_USERNAME/stitch-sync
   cd stitch-sync

   # Install dependencies (Linux only)
   sudo apt-get install libudev-dev  # Ubuntu/Debian

   # Build the project
   cargo build

   # Run tests
   cargo test
   ```

3. **Make your changes**
   - Write clean, readable code that follows Rust conventions
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass: `cargo test`
   - Run the formatter: `cargo fmt`
   - Run the linter: `cargo clippy`

4. **Commit your changes**
   - Use clear, descriptive commit messages
   - Follow conventional commits format when possible:
     - `feat:` for new features
     - `fix:` for bug fixes
     - `docs:` for documentation changes
     - `test:` for test additions/changes
     - `refactor:` for code refactoring
     - `chore:` for maintenance tasks

5. **Push to your fork and submit a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Wait for review**
   - Address any feedback from reviewers
   - Keep your branch up to date with main

## Development Guidelines

### Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and reasonably sized

### Error Handling

- Use `anyhow::Result` for error propagation
- Add context to errors using `.context()`
- Avoid `unwrap()` in production code
- Use `expect()` with descriptive messages only for truly unreachable cases

### Testing

- Write unit tests for new functions
- Add integration tests for end-to-end functionality
- Test on multiple platforms when possible
- Mock external dependencies when appropriate

### Documentation

- Add doc comments (`///`) for public functions and types
- Update README.md for user-facing changes
- Update docs/ for significant features
- Include examples in documentation

## Contributing to the Machine Database

The machine database (`src/types/machines.csv`) can always be expanded. To add a machine:

1. Add a row to `machines.csv` with:
   - Machine Name
   - Supported File Formats (comma-separated)
   - USB Path (if specific path required)
   - Notes (optional)
   - Design Size (optional)
   - Synonyms (optional, comma-separated)

2. Verify the information against manufacturer specifications

3. Test with the actual machine if possible

4. Submit a pull request with the addition

## Project Structure

```
stitch-sync/
├── src/
│   ├── cli/              # Command-line interface
│   ├── config/           # Configuration management
│   ├── services/         # Core business logic
│   ├── types/            # Domain models
│   └── utils/            # Helper functions
├── docs/                 # Documentation
├── scripts/              # Installation and utility scripts
└── .github/workflows/    # CI/CD configuration
```

## Building for Different Platforms

### Linux
```bash
cargo build --release
```

### macOS
```bash
cargo build --release
```

### Windows
```bash
cargo build --release
```

Cross-compilation is also supported. See the release workflow for examples.

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for specific module
cargo test services::
```

## Getting Help

- Check existing issues and pull requests
- Read the [developer documentation](docs/developer-notes.md)
- Create a discussion on GitHub for questions

## License

By contributing to Stitch-sync, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing!
