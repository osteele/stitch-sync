# Configuration

The program supports a configuration file located at:

- Linux/macOS: `~/.config/stitch-sync/config.toml`
- Windows: `%APPDATA%\stitch-sync\config.toml`

Example configuration:
```toml
# Default directory to watch
watch_dir = "/Users/username/Downloads"

# Default machine
machine = "Brother PE800"
```

You can set configuration values using the following commands:

```bash
# Set default watch directory
stitch-sync set watch-dir /path/to/directory

# Set default machine
stitch-sync set machine "Brother PE800"

# Clear a configuration value
stitch-sync config clear watch-dir

# View current configuration
stitch-sync config show
```
