# Developer Notes

This document contains technical information and guidelines for developers working on the Stitch-sync project.

## High-Level Design

```mermaid
graph TD
A[Embroidery Design Files] --> B[Watch Directory]
B --> C[Stitch-sync Application]
C --> D{Configuration}
D --> E[Machine Database]
D --> F[User Preferences]
C --> G[Ink/Stitch Extension]
C --> H[USB Drive Detection]
H --> I[USB Drive]
```

## Project Structure

```text
stitch-sync/
├── src/
│   ├── main.rs           # Application entry point
│   ├── cli/              # Command-line interface modules
│   ├── config/           # Configuration handling
│   ├── conversion/       # File conversion logic
│   ├── machine/          # Embroidery machine definitions
│   └── watcher/          # File system monitoring
├── tests/                # Integration tests
├── docs/                 # Documentation
└── resources/            # Static resources (machine data, etc.)
```

## Development Setup

1. Install development dependencies:
   - Rust and Cargo (latest stable)
   - Inkscape with ink/stitch extension
   - (Linux only) libudev-dev: `sudo apt-get install libudev-dev`

2. Clone and build:
   ```bash
   git clone https://github.com/osteele/stitch-sync
   cd stitch-sync
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Cross-Platform Development

The project uses conditional compilation for platform-specific code:

```rust
#[cfg(target_os = "macos")]
fn get_usb_drives() -> Vec<Path> {
    // macOS-specific implementation
}

#[cfg(target_os = "windows")]
fn get_usb_drives() -> Vec<Path> {
    // Windows-specific implementation
}

#[cfg(target_os = "linux")]
fn get_usb_drives() -> Vec<Path> {
    // Linux-specific implementation
}
```

### Platform-Specific Dependencies

- **Linux**: Uses `libudev` for USB device detection
- **macOS**: Uses `core-foundation` and `io-kit-sys` for device management
- **Windows**: Uses the `windows` crate for Win32 API access

## Testing

### Unit Tests

Write unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory. These tests verify the interaction between multiple components.

### Cross-Platform Testing

The project uses GitHub Actions to test on multiple platforms:
- Ubuntu Latest
- Windows Latest
- macOS Latest

See `.github/workflows/crossplatform-test.yml` for the CI configuration.

## Code Style

- Follow the Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Prefer functional programming patterns where appropriate

## Error Handling

- Use `anyhow::Result` for error propagation
- Create custom errors when needed using `thiserror`
- Provide meaningful error messages for user-facing errors

## Configuration Management

Configuration is handled through:
1. Command-line arguments (highest priority)
2. Environment variables
3. Configuration file (lowest priority)

The configuration file uses TOML format and is located at:
- Linux/macOS: `~/.config/stitch-sync/config.toml`
- Windows: `%APPDATA%\stitch-sync\config.toml`
