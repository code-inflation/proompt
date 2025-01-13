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

