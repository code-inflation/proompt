use clap::Parser;
use ignore::WalkBuilder;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

    /// Maximum file size to process (e.g., 1MB, 256KB)
    #[arg(long, value_name = "SIZE")]
    max_file_size: Option<String>,

    /// Maximum number of lines to include per file
    #[arg(long, value_name = "COUNT")]
    max_lines: Option<usize>,

    /// Add file metadata (size, lines, modification date) to headers
    #[arg(long)]
    add_metadata: bool,
}

fn parse_file_size(size_str: &str) -> Result<u64, String> {
    let size_str = size_str.trim().to_uppercase();
    let (num_str, unit) = if size_str.ends_with("KB") {
        (&size_str[..size_str.len() - 2], 1024u64)
    } else if size_str.ends_with("MB") {
        (&size_str[..size_str.len() - 2], 1024u64 * 1024)
    } else if size_str.ends_with("GB") {
        (&size_str[..size_str.len() - 2], 1024u64 * 1024 * 1024)
    } else if size_str.ends_with("B") {
        (&size_str[..size_str.len() - 1], 1u64)
    } else {
        (size_str.as_str(), 1u64)
    };

    num_str
        .parse::<u64>()
        .map(|n| n * unit)
        .map_err(|_| format!("Invalid file size format: {}", size_str))
}

fn get_file_metadata(path: &Path) -> Result<(u64, usize, String), std::io::Error> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();

    let content = fs::read_to_string(path)?;
    let line_count = content.lines().count();

    let modified_time = metadata
        .modified()
        .unwrap_or(SystemTime::now())
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let modified_date = chrono::DateTime::from_timestamp(modified_time as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok((size, line_count, modified_date))
}

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.1}GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}KB", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}

fn process_file(
    path: &Path,
    args: &Args,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    // Check file size limit
    if let Some(max_size_str) = &args.max_file_size {
        let max_size = parse_file_size(max_size_str)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        let file_size = fs::metadata(path)?.len();
        if file_size > max_size {
            eprintln!(
                "Warning: Skipping file {} due to size limit ({} > {})",
                path.display(),
                format_file_size(file_size),
                max_size_str
            );
            return Ok(());
        }
    }

    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            eprintln!(
                "Warning: Skipping file {} due to UnicodeDecodeError",
                path.display()
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    // Apply line limit if specified
    let processed_content = if let Some(max_lines) = args.max_lines {
        let lines: Vec<&str> = content.lines().take(max_lines).collect();
        let truncated = content.lines().count() > max_lines;
        let mut result = lines.join("\n");
        if truncated {
            result.push_str("\n... (truncated)");
        }
        result
    } else {
        content
    };

    // Write file header with optional metadata
    if args.add_metadata {
        match get_file_metadata(path) {
            Ok((size, line_count, modified_date)) => {
                writeln!(
                    writer,
                    "{} ({} lines, {}, modified: {})",
                    path.display(),
                    line_count,
                    format_file_size(size),
                    modified_date
                )?;
            }
            Err(_) => {
                writeln!(writer, "{}", path.display())?;
            }
        }
    } else {
        writeln!(writer, "{}", path.display())?;
    }

    writeln!(writer, "---")?;
    writeln!(writer, "{}", processed_content)?;
    writeln!(writer, "---")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_size() {
        assert_eq!(parse_file_size("1024").unwrap(), 1024);
        assert_eq!(parse_file_size("1KB").unwrap(), 1024);
        assert_eq!(parse_file_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_file_size("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_file_size("2MB").unwrap(), 2 * 1024 * 1024);
        assert!(parse_file_size("invalid").is_err());
        assert!(parse_file_size("1XB").is_err());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(512), "512B");
        assert_eq!(format_file_size(1024), "1.0KB");
        assert_eq!(format_file_size(1536), "1.5KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0GB");
    }

    #[test]
    fn test_get_file_metadata() {
        let test_file = "test_temp_file.txt";
        std::fs::write(test_file, "line1\nline2\nline3").unwrap();

        let (size, line_count, _date) = get_file_metadata(std::path::Path::new(test_file)).unwrap();
        assert_eq!(line_count, 3);
        assert_eq!(size, 17); // "line1\nline2\nline3" = 17 bytes

        std::fs::remove_file(test_file).unwrap();
    }
}

fn process_path(
    path: &Path,
    args: &Args,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    if path.is_file() {
        process_file(path, args, writer)?;
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
                            process_file(entry.path(), args, writer)?;
                        }
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
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
