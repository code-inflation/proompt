use clap::Parser;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};

/// Concatenate a directory full of files into a single prompt for use with LLMs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to files or directories to process
    paths: Vec<PathBuf>,

    /// Only include files with the specified extensions
    #[arg(short, long)]
    extension: Vec<String>,

    /// Include hidden files and directories
    #[arg(long)]
    include_hidden: bool,

    /// Ignore .gitignore files
    #[arg(long)]
    ignore_gitignore: bool,

    /// Patterns to ignore
    #[arg(long, value_name = "PATTERN")]
    ignore: Vec<String>,

    /// Write output to a file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn process_file(path: &Path, writer: &mut impl std::io::Write) -> Result<(), std::io::Error> {
    match fs::read_to_string(path) {
        Ok(content) => {
            writeln!(writer, "{}", path.display())?;
            writeln!(writer, "---")?;
            writeln!(writer, "{}", content)?;
            writeln!(writer, "---")?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            eprintln!(
                "Warning: Skipping file {} due to UnicodeDecodeError",
                path.display()
            );
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

fn process_path(
    path: &Path,
    args: &Args,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    if path.is_file() {
        process_file(path, writer)?;
    } else if path.is_dir() {
        let walker = WalkBuilder::new(path)
            .hidden(!args.include_hidden)
            .ignore(!args.ignore_gitignore)
            .git_ignore(!args.ignore_gitignore)
            .git_global(!args.ignore_gitignore)
            .git_exclude(!args.ignore_gitignore)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().unwrap().is_file() {
                        let relative_path = entry
                            .path()
                            .strip_prefix(path)
                            .expect("Failed to get relative path");
                        let relative_path_str = relative_path.to_string_lossy();

                        let should_process = if !args.extension.is_empty() {
                            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                                args.extension.contains(&ext.to_string())
                            } else {
                                false
                            }
                        } else {
                            true
                        };

                        let should_process = should_process
                            && if !args.ignore.is_empty() {
                                !args.ignore.iter().any(|pattern| {
                                    glob::Pattern::new(pattern)
                                        .unwrap()
                                        .matches(&relative_path_str)
                                })
                            } else {
                                true
                            };

                        if should_process {
                            process_file(entry.path(), writer)?;
                        }
                    }
                }
                Err(err) => eprintln!("ERROR: {}", err),
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let mut output_writer: Box<dyn std::io::Write> = match &args.output {
        Some(output_path) => Box::new(fs::File::create(output_path)?),
        None => Box::new(std::io::stdout()),
    };

    for path in &args.paths {
        if !path.exists() {
            eprintln!("Error: Path does not exist: {}", path.display());
            std::process::exit(1);
        }
        process_path(path, &args, &mut output_writer)?;
    }

    Ok(())
}
