use atty::Stream;
use clap::Parser;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Do not overwrite existing variables
    #[arg(short, long)]
    no_overwrite: bool,

    /// File path for the .env file
    #[arg(short, long, default_value = ".env")]
    file: String,

    /// KEY=value pairs to set
    #[arg(required = false)]
    vars: Vec<String>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Get the value of a single environment variable
    Get { key: String },
    /// Print all environment variables
    Print {
        /// File path for the .env file
        #[arg(short, long, default_value = ".env")]
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Get { key }) => {
            if let Err(e) = dotenv::from_filename(&cli.file) {
                eprintln!("Error loading .env file: {}", e);
                process::exit(1);
            }

            match env::var(key) {
                Ok(value) => println!("{}", value),
                Err(_) => {
                    eprintln!("Environment variable '{}' not found", key);
                    process::exit(1);
                }
            }
        }
        Some(Commands::Print { file }) => {
            if let Err(e) = dotenv::from_filename(file) {
                eprintln!("Error loading .env file: {}", e);
                process::exit(1);
            }

            print_all_env_vars();
        }
        None => {
            let no_overwrite = cli.no_overwrite;
            let env_file = &cli.file;
            let (mut env_vars, original_lines) = match read_env_file(env_file) {
                Ok(result) => result,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        (HashMap::new(), Vec::new())
                    } else {
                        eprintln!("Error reading .env file: {}", e);
                        process::exit(1);
                    }
                }
            };

            let original_env = env_vars.clone();

            let new_vars = if atty::is(Stream::Stdin) {
                parse_args(&cli.vars)
            } else {
                parse_stdin()
            };

            for (key, value) in &new_vars {
                if !env_vars.contains_key(key as &str) || !no_overwrite {
                    env_vars.insert(key.clone(), value.clone());
                }
            }

            if let Err(e) = write_env_file(env_file, &env_vars, &original_lines) {
                eprintln!("Error writing .env file: {}", e);
                process::exit(1);
            }

            print_diff(&original_env, &env_vars);
        }
    }
}

fn print_all_env_vars() {
    print_all_env_vars_to_writer(&mut std::io::stdout());
}

fn print_all_env_vars_to_writer<W: Write>(writer: &mut W) {
    for (key, value) in env::vars() {
        writeln!(writer, "{}={}", key, value).unwrap();
    }
}

fn print_diff(original: &HashMap<String, String>, updated: &HashMap<String, String>) {
    print_diff_to_writer(original, updated, &mut std::io::stdout());
}

fn print_diff_to_writer<W: Write>(
    original: &HashMap<String, String>,
    updated: &HashMap<String, String>,
    writer: &mut W,
) {
    for key in updated.keys() {
        let updated_value = updated.get(key).unwrap();
        match original.get(key) {
            Some(original_value) if original_value != updated_value => {
                writeln!(writer, "{}", format!("-{}={}", key, original_value).red()).unwrap();
                writeln!(writer, "{}", format!("+{}={}", key, updated_value).green()).unwrap();
            }
            None => {
                writeln!(writer, "{}", format!("+{}={}", key, updated_value).green()).unwrap();
            }
            _ => {}
        }
    }

    for key in original.keys() {
        if !updated.contains_key(key) {
            writeln!(
                writer,
                "{}",
                format!("-{}={}", key, original.get(key).unwrap()).red()
            )
            .unwrap();
        }
    }
}

fn read_env_file(
    file_path: &str,
) -> Result<(HashMap<String, String>, Vec<String>), std::io::Error> {
    let path = Path::new(file_path);
    let mut env_vars = HashMap::new();
    let mut original_lines = Vec::new();

    if path.exists() {
        let contents = fs::read_to_string(path)?;
        for line in contents.lines() {
            original_lines.push(line.to_string());
            if let Some((key, value)) = line.split_once('=') {
                if !line.trim_start().starts_with('#') {
                    env_vars.insert(
                        key.trim().to_string(),
                        value.trim().trim_matches('"').trim_matches('\'').to_owned(),
                    );
                }
            }
        }
    }

    Ok((env_vars, original_lines))
}

fn write_env_file(
    file_path: &str,
    env_vars: &HashMap<String, String>,
    original_lines: &[String],
) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)?;

    let mut written_keys = HashSet::new();

    // First pass: write existing lines and update values
    for line in original_lines {
        if let Some((key, _)) = line.split_once('=') {
            let trimmed_key = key.trim();
            if let Some(value) = env_vars.get(trimmed_key) {
                if !written_keys.contains(trimmed_key) {
                    writeln!(file, "{}={}", trimmed_key, value)?;
                    written_keys.insert(trimmed_key.to_string());
                }
            } else {
                writeln!(file, "{}", line)?;
            }
        } else {
            writeln!(file, "{}", line)?;
        }
    }

    // Second pass: write new variables
    for (key, value) in env_vars {
        if !written_keys.contains(key.as_str()) {
            writeln!(file, "{}={}", key, value)?;
        }
    }

    Ok(())
}
fn parse_stdin() -> HashMap<String, String> {
    parse_stdin_with_reader(&mut io::stdin())
}

fn parse_stdin_with_reader<R: Read>(reader: &mut R) -> HashMap<String, String> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    parse_env_content(&buffer)
}

fn parse_args(vars: &[String]) -> HashMap<String, String> {
    vars.iter()
        .filter_map(|var| {
            let mut parts = var.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((
                    key.trim().to_string(),
                    value
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .to_string(),
                )),
                _ => {
                    println!("Invalid argument: {}. Skipping.", var);
                    None
                }
            }
        })
        .collect()
}

fn parse_env_content(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                None
            } else {
                let mut parts = line.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((
                        key.trim().to_string(),
                        value
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"')
                            .to_string(),
                    )),
                    _ => None,
                }
            }
        })
        .collect()
}
