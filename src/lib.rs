use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn read_env_file(
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

pub fn write_env_file(
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

pub fn parse_stdin() -> HashMap<String, String> {
    parse_stdin_with_reader(&mut io::stdin())
}

pub fn parse_stdin_with_reader<R: Read>(reader: &mut R) -> HashMap<String, String> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    parse_env_content(&buffer)
}

pub fn parse_args(vars: &[String]) -> HashMap<String, String> {
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

pub fn parse_env_content(content: &str) -> HashMap<String, String> {
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

pub fn print_all_env_vars(file_path: &str) {
    print_all_env_vars_to_writer(file_path, &mut std::io::stdout());
}

pub fn print_all_env_vars_to_writer<W: Write>(file_path: &str, writer: &mut W) {
    if let Ok((env_vars, _)) = read_env_file(file_path) {
        for (key, value) in env_vars {
            writeln!(writer, "{}={}", key.blue().bold(), value.green()).unwrap();
        }
    } else {
        eprintln!("Error reading .env file");
    }
}

pub fn print_all_keys(file_path: &str) {
    print_all_keys_to_writer(file_path, &mut std::io::stdout());
}

pub fn print_all_keys_to_writer<W: Write>(file_path: &str, writer: &mut W) {
    if let Ok((env_vars, _)) = read_env_file(file_path) {
        for key in env_vars.keys() {
            writeln!(writer, "{}", key).unwrap();
        }
    } else {
        eprintln!("Error reading .env file");
    }
}

pub fn print_diff(original: &HashMap<String, String>, updated: &HashMap<String, String>) {
    print_diff_to_writer(original, updated, &mut std::io::stdout());
}

pub fn print_diff_to_writer<W: Write>(
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

pub fn delete_env_vars(file_path: &str, keys: &[String]) -> std::io::Result<()> {
    let (mut env_vars, original_lines) = read_env_file(file_path)?;

    for key in keys {
        env_vars.remove(key);
    }

    write_env_file(file_path, &env_vars, &original_lines)
}
