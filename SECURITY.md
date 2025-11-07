# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in Stitch-sync, please report it by:

1. **DO NOT** open a public issue
2. Email the maintainer at: steele@osteele.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

You should receive a response within 48 hours. If the vulnerability is accepted, we will:
- Work on a fix and release a patch as soon as possible
- Credit you in the release notes (unless you prefer to remain anonymous)
- Notify you when the fix is released

## Known Security Considerations

### Dependency Vulnerabilities

GitHub Dependabot has identified vulnerabilities in some dependencies. To check and update:

```bash
# Install cargo-audit for vulnerability scanning
cargo install cargo-audit

# Scan for vulnerabilities
cargo audit

# Update dependencies to latest compatible versions
cargo update

# Check for outdated dependencies
cargo outdated
```

### Security Best Practices

When using Stitch-sync:

1. **File Path Validation**: The application sanitizes filenames but always review files before processing
2. **USB Drive Access**: The tool requires access to USB drives - ensure you trust the embroidery files being processed
3. **Inkscape Integration**: Files are processed through Inkscape/Ink/Stitch - keep these tools updated
4. **Network Access**: The self-update feature downloads binaries from GitHub - ensure you're on a trusted network

### Command Execution

Stitch-sync executes external commands (Inkscape, diskutil, etc.). Key security measures:

1. **No User Input in Commands**: Paths are validated and sanitized before use
2. **Whitelist Approach**: Only predefined commands are executed
3. **Error Handling**: Failed commands don't expose sensitive information

### File Operations

1. **Filename Sanitization**: Non-alphanumeric characters are converted to hyphens
2. **Path Validation**: Directory traversal attempts are prevented
3. **Extension Validation**: Only known embroidery formats are processed

## Security Hardening Recommendations

For production use, consider:

1. **Run with Minimal Permissions**: Don't run as root/administrator unless necessary
2. **Isolate Watch Directories**: Use dedicated directories for embroidery files
3. **Verify File Sources**: Only process files from trusted sources
4. **Keep Dependencies Updated**: Regularly run `cargo update` and `cargo audit`
5. **Monitor USB Devices**: Be aware of which USB drives are mounted

## Development Security

For contributors:

1. **Review Dependencies**: Check new dependencies with `cargo audit`
2. **Avoid Unsafe Code**: Use unsafe blocks only when absolutely necessary and document thoroughly
3. **Input Validation**: Always validate external input (files, paths, user input)
4. **Error Messages**: Don't expose sensitive information in error messages
5. **Testing**: Include security test cases for path handling and command execution

## Automated Security

This project uses:

- **GitHub Dependabot**: Automated dependency vulnerability scanning
- **Cargo Clippy**: Linting for common security issues
- **GitHub Actions**: CI/CD with security checks

## Regular Maintenance Tasks

Maintainers should:

1. Review and merge Dependabot PRs promptly
2. Run `cargo audit` before each release
3. Update dependencies monthly with `cargo update`
4. Monitor GitHub Security Advisories
5. Review code for unsafe patterns during PR reviews

## Disclosure Policy

When a vulnerability is fixed:

1. Release a patch version immediately
2. Update CHANGELOG.md with security fix details
3. Credit the reporter (with permission)
4. Notify users through GitHub releases
5. Consider creating a GitHub Security Advisory for severe vulnerabilities

## Contact

For security concerns, contact: steele@osteele.com

For general issues, use: https://github.com/osteele/stitch-sync/issues

Thank you for helping keep Stitch-sync secure!
