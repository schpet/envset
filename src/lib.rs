pub mod parse;

use crate::parse::{parse, Node};
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
        let ast = parse(&contents);
        for node in ast.iter() {
            match node {
                Node::KeyValue { key, value, .. } => {
                    env_vars.insert(key.clone(), value.clone());
                }
                Node::Comment(comment) => {
                    original_lines.push(comment.clone());
                }
                Node::EmptyLine => {
                    original_lines.push(String::new());
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
    let mut new_vars = Vec::new();

    // First pass: write existing lines and update values
    for line in original_lines {
        let ast = parse(line);
        match ast.first() {
            Some(Node::KeyValue { key, .. }) => {
                if let Some(value) = env_vars.get(key) {
                    writeln!(file, "{}={}", key, value)?;
                    written_keys.insert(key.to_string());
                } else {
                    writeln!(file, "{}", line)?;
                }
            }
            _ => writeln!(file, "{}", line)?,
        }
    }

    // Collect new variables
    for (key, value) in env_vars {
        if !written_keys.contains(key.as_str()) {
            new_vars.push((key, value));
        }
    }

    // Sort new variables to ensure consistent order
    new_vars.sort_by(|a, b| a.0.cmp(b.0));

    // Second pass: write new variables
    for (key, value) in new_vars {
        writeln!(file, "{}={}", key, value)?;
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
            let ast = parse(var);
            if let Some(Node::KeyValue { key, value, .. }) = ast.first() {
                Some((key.clone(), value.clone()))
            } else {
                println!("Invalid argument: {}. Skipping.", var);
                None
            }
        })
        .collect()
}

pub fn parse_env_content(content: &str) -> HashMap<String, String> {
    let ast = parse(content);
    ast.iter()
        .filter_map(|node| {
            if let Node::KeyValue { key, value, .. } = node {
                Some((key.clone(), value.clone()))
            } else {
                None
            }
        })
        .collect()
}

pub fn print_all_env_vars(file_path: &str) {
    print_all_env_vars_to_writer(file_path, &mut std::io::stdout());
}

pub fn print_all_env_vars_to_writer<W: Write>(file_path: &str, writer: &mut W) {
    if let Ok((env_vars, _)) = read_env_file(file_path) {
        let mut sorted_keys: Vec<_> = env_vars.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            if let Some(value) = env_vars.get(key) {
                writeln!(writer, "{}={}", key.blue().bold(), value.green()).unwrap();
            }
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
    let (env_vars, original_lines) = read_env_file(file_path)?;

    let updated_env_vars: HashMap<String, String> = env_vars
        .into_iter()
        .filter(|(key, _)| !keys.contains(key))
        .collect();

    write_env_file(file_path, &updated_env_vars, &original_lines)
}
