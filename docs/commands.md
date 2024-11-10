# Command Line Interface

## Commands

- `watch`: Watch directory and convert files
  - Arguments:
    - `--dir` / `-d`: Directory to watch for new DST files (optional)
    - `--output-format` / `-o`: Output format, e.g., 'jef', 'pes' (optional)
    - `--machine` / `-m`: Target machine, determines accepted formats (optional)
- `set`: Set default machine (alias for 'config set machine')
  - Arguments:
    - `what`: What to set ('machine' only for now)
    - `value`: Value to set (if not provided, will prompt for input)
- `machine`: Machine-related commands
  - Subcommands:
    - `list`: List all supported machines
      - Arguments:
        - `--format` / `-f`: Filter by file format (optional)
        - `--verbose` / `-v`: Verbose output (optional)
    - `info`: Show detailed information for a specific machine
      - Arguments:
        - `name`: Name of the machine
- `machines`: List all supported machines (alias for 'machine list')
  - Arguments:
    - `--format` / `-f`: Filter by file format (optional)
    - `--verbose` / `-v`: Verbose output (optional)
- `formats`: List supported file formats
- `config`: Configuration commands
  - Subcommands:
    - `show`: Show current configuration
    - `set`: Set a configuration value
      - Arguments:
        - `key`: Configuration key to set (watch-dir, machine)
        - `value`: Value to set (if not provided, will prompt for input)
    - `clear`: Clear a configuration value
      - Arguments:
        - `key`: Configuration key to clear (watch-dir, machine)
- `update`: Update stitch-sync to the latest version
  - Arguments:
    - `--dry-run`: Check for updates but don't install them (optional)
- `homepage`: Open the project homepage
- `report-bug`: Create a new bug report on GitHub
- `version`: Show version and build information

## Examples

Set your embroidery machine:
```bash
stitch-sync set machine
```

Specify a target machine for just the current session, and watch for new designs:
```bash
stitch-sync watch --machine "Brother PE800"
```

Watch a directory besides the default downloads directory:
```bash
stitch-sync watch --dir /path/to/directory
```

Select a different output format from the default (DST):
```bash
stitch-sync watch --output-format jef
```

List all supported machines:
```bash
stitch-sync machines
```

List machines that support a specific format:
```bash
stitch-sync machines --format dst
```

List all supported file formats:
```bash
stitch-sync formats
```

Show detailed information for a specific machine:
```bash
stitch-sync machine info "Brother PE800"
```

Set default watch directory:
```bash
stitch-sync config set watch-dir /path/to/directory
```

Set default machine:
```bash
stitch-sync config set machine "Brother PE800"
```

Clear a configuration value:
```bash
stitch-sync config clear watch-dir
```

View current configuration:
```bash
stitch-sync config show
```

Update to the latest version:
```bash
stitch-sync update
```

Open the project homepage:
```bash
stitch-sync homepage
```

Report a bug:
```bash
stitch-sync report-bug
```

View version and build information:
```bash
stitch-sync version
```

View help:
```bash
stitch-sync --help
```
