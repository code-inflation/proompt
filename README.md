# proompt
`proompt` is a small CLI utility designed to concatenate the contents of files within specified directories into a single prompt. It supports various options for customizing file processing, including handling hidden files, ignoring specific patterns, and managing different file extensions.

## Installation
```sh
cargo install proompt
```

## Help
```sh
/proompt --help
Concatenate a directory full of files into a single prompt for use with LLMs

Usage: proompt [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Paths to files or directories to process

Options:
  -e, --extension <EXTENSION>  Only include files with the specified extensions
      --include-hidden         Include hidden files and directories
      --ignore-gitignore       Ignore .gitignore files
      --ignore <PATTERN>       Patterns to ignore
  -o, --output <FILE>          Write output to a file
      --max-file-size <SIZE>   Maximum file size to process (e.g., 1MB, 256KB)
      --max-lines <COUNT>      Maximum number of lines to include per file
      --add-metadata           Add file metadata (size, lines, modification date) to headers
  -h, --help                   Print help
  -V, --version                Print version
```

## Example Usage
Copy full repo:
```sh
proompt . | pbcopy
```

Exclude tests dir:
```sh
proompt --ignore tests/* . | pbcopy
```

## New Features

### File Size Limiting
Skip files larger than specified size:
```sh
proompt --max-file-size 1MB . | pbcopy
proompt --max-file-size 256KB src/ | pbcopy
```

### Line Limiting
Limit the number of lines per file:
```sh
proompt --max-lines 50 . | pbcopy
proompt --max-lines 100 src/ | pbcopy
```

### Add File Metadata
Include file information in headers:
```sh
proompt --add-metadata . | pbcopy
```

Output format with metadata:
```
src/main.rs (150 lines, 4.2KB, modified: 2025-01-15)
---
[File contents]
---
```

### Combined Usage
Combine multiple options:
```sh
proompt --add-metadata --max-lines 100 --max-file-size 500KB src/ | pbcopy
```

