# Demo Fixtures

This directory contains sample files for the file browser demo.

## Contents

- `config/` - Configuration files (TOML, YAML)
- `logs/` - Sample log files
- `nested/` - Nested directory structure demo

## Usage

These files demonstrate the FilePicker component's ability to:

1. Navigate directory trees
2. Preview file contents
3. Handle different file types
4. Show hidden files (toggle with `h`)

## File Types

| Extension | Handler |
|-----------|---------|
| `.md` | Markdown preview (glamour) |
| `.toml` | Syntax highlighted |
| `.yaml` | Syntax highlighted |
| `.log` | Plain text with scroll |
