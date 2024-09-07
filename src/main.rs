use atty::Stream;
use clap::Parser;
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();

    let no_overwrite = cli.no_overwrite;
    let env_file = &cli.file;
    let (mut env_vars, original_lines) = match read_env_file(env_file) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error reading .env file: {}", e);
            process::exit(1);
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

fn print_diff(original: &HashMap<String, String>, updated: &HashMap<String, String>) {
    let mut original_content = String::new();
    let mut updated_content = String::new();

    for key in original
        .keys()
        .chain(updated.keys())
        .collect::<std::collections::HashSet<_>>()
    {
        let original_value = original.get(key).map(|v| v.as_str()).unwrap_or("");
        let updated_value = updated.get(key).map(|v| v.as_str()).unwrap_or("");

        original_content.push_str(&format!("{}={}\n", key, original_value));
        updated_content.push_str(&format!("{}={}\n", key, updated_value));
    }

    let diff = TextDiff::from_lines(&original_content, &updated_content);

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => print!("{}", format!("-{}", change).red()),
            ChangeTag::Insert => print!("{}", format!("+{}", change).green()),
            ChangeTag::Equal => {}
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

    for line in original_lines {
        if let Some((key, original_value)) = line.split_once('=') {
            let trimmed_key = key.trim();
            if let Some(value) = env_vars.get(trimmed_key) {
                let formatted_value = if original_value.trim().starts_with('"')
                    || original_value.trim().starts_with('\'')
                {
                    format!(
                        "{}{}{}",
                        &original_value[..1],
                        value,
                        &original_value[original_value.len() - 1..]
                    )
                } else {
                    value.to_string()
                };
                writeln!(file, "{}={}", trimmed_key, formatted_value)?;
            } else {
                writeln!(file, "{}", line)?;
            }
        } else {
            writeln!(file, "{}", line)?;
        }
    }

    // Add any new variables that weren't in the original file
    for (key, value) in env_vars {
        if !original_lines.iter().any(|line| line.starts_with(key)) {
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
