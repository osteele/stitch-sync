# Format Selection and Machine Compatibility

This document explains how dst2jef handles file format selection and machine compatibility.

## Overview

dst2jef can operate in two modes:
- Simple format conversion (using --output-format)
- Machine-aware conversion (using --machine)

## Command Line Options

- `--machine` / `-m`: Specify target embroidery machine
- `--output-format` / `-o`: Specify desired output format
- Both options are optional and can be used together

## Format Selection Logic

### Default Behavior (No Options)
- Accepts DST files
- Converts to JEF format
- Copies only converted files to EMB directory

### With --machine Specified
- Accepts all formats supported by the specified machine
- Uses machine's primary format as preferred output format
- Copies both converted files and already-compatible files to EMB directory

### With --output-format Specified
- Accepts only input files that can be converted to specified format
- Converts all files to specified format
- Copies only converted files to EMB directory

### With Both Options
- Accepts all formats supported by the specified machine
- Uses specified output format instead of machine's default
- Copies both converted files and already-compatible files to EMB directory

## Examples

```bash
# Convert only DST files to JEF
dst2jef watch

# Accept any Brother PE800-compatible format, convert others to PES
dst2jef watch --machine "Brother PE800"

# Convert everything to JEF+, regardless of machine compatibility
dst2jef watch --output-format jef+

# Accept Brother PE800 formats but convert incompatible files to JEF+
dst2jef watch --machine "Brother PE800" --output-format jef+
```

## Technical Details

### Acceptable Formats
- With --machine: All formats listed in machine's specifications
- Without --machine: Only the target format (DST by default)

### Preferred Format
Priority order:
1. Explicitly specified --output-format
2. First format in machine's format list (if --machine specified)
3. Default format (DST)

### File Handling
1. If file extension matches an acceptable format:
   - File is copied directly to EMB directory
   - No conversion performed
2. If file requires conversion:
   - Converted to preferred format
   - Converted file copied to EMB directory
3. If file is neither acceptable nor convertible:
   - File is ignored
