# dst2jef

A command-line utility that automatically converts DST embroidery files to JEF
format using Inkscape with the ink/stitch extension. It watches a directory for
new DST files and automatically converts them when they appear.

## Features

- Watches a directory (default: Downloads) for new .dst files
- Automatically converts DST files to JEF format using Inkscape
- Optionally copies converted files to an EMB/Embf directory (e.g., on a USB
  drive)
- Database of embroidery machines and their supported formats
- Cross-platform support (macOS, Windows, Linux)
- Sanitizes output filenames for better compatibility
- Real-time conversion status updates

## Prerequisites

1. [Inkscape](https://inkscape.org/) must be installed on your system
2. The [ink/stitch extension](https://inkstitch.org/) must be installed in
   Inkscape
3. Rust and Cargo must be installed on your system

## Installation

```bash
# Clone the repository
git clone https://github.com/osteele/dst2jef
cd dst2jef

# Build and install
cargo install --path .
```

## Usage

Basic usage (watches Downloads directory):

```bash
dst2jef watch
```

Watch a specific directory:

```bash
dst2jef watch --dir /path/to/directory
```

List all supported machines:

```bash
dst2jef machines
```

List machines that support a specific format:

```bash
dst2jef machines --format dst
```

List all supported file formats:

```bash
dst2jef formats
```

Show detailed information for a specific machine:

```bash
dst2jef machine info "Brother PE800"
```

View help:

```bash
dst2jef --help
```

Example output:

```bash
# List file formats
$ dst2jef formats
dst: Tajima
  Industry standard format, widely supported
exp: Melco Expanded
jef: Janome Embroidery Format
jef+: Janome Embroidery Format Plus
  Enhanced version of JEF with additional features
pes: Brother Embroidery Format
vip: Viking/Pfaff
  Legacy format
vp3: Viking/Pfaff Phase 3
  Current format for Viking and Pfaff machines
xxx: Singer

# List all machines
$ dst2jef machines
Brother PE800 (pes)
Janome MC9900 (jef, dst)
Pfaff Creative 4 (vp3)
```

## How It Works

1. The program starts watching the specified directory for new .dst files
2. When a new .dst file is detected:
   - Converts it to .jef format using Inkscape with ink/stitch
   - Sanitizes the output filename (removes spaces/underscores)
   - If a USB drive with an EMB/Embf directory is found, copies the .jef file there
3. Press 'q' to quit the program

## Supported Platforms

- macOS:
  - Looks for Inkscape in PATH and `/Applications/Inkscape.app`
  - Checks `/Volumes` for USB drives
- Windows:
  - Looks for Inkscape in PATH and Program Files
  - Checks all drive letters for USB drives
- Linux:
  - Looks for Inkscape in PATH and common installation directories
  - Checks `/media/<username>` for USB drives

## Troubleshooting

### Inkscape Not Found

Make sure Inkscape is installed and accessible. Download from:
- macOS: [Inkscape for macOS][inkscape-mac]
- Windows: [Inkscape for Windows][inkscape-win]
- Linux: Use your package manager or [Inkscape for Linux][inkscape-linux]

### ink/stitch Extension Not Found

1. Download the [ink/stitch extension][inkstitch-install]
2. Follow the installation instructions for your platform
3. Restart Inkscape after installation

### Conversion Errors

1. Ensure your DST file is valid
2. Check that ink/stitch is properly installed
3. Try converting the file manually in Inkscape to verify it works

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [Inkscape](https://inkscape.org/) - Vector graphics software
- [ink/stitch](https://inkstitch.org/) - Embroidery extension for Inkscape

[inkscape-mac]: https://inkscape.org/release/1.4/mac-os-x/
[inkscape-win]: https://inkscape.org/release/1.4/windows/
[inkscape-linux]: https://inkscape.org/release/1.4/linux/
[inkstitch-install]: https://inkstitch.org/docs/install/
