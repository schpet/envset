pub mod parse;

use crate::parse::{parse, Node};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn read_env_vars(file_path: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let path = Path::new(file_path);
    let mut env_vars = HashMap::new();

    if path.exists() {
        let contents = fs::read_to_string(path)?;
        let ast = parse(&contents);
        for node in ast.iter() {
            if let Node::KeyValue { key, value, .. } = node {
                env_vars.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(env_vars)
}

pub fn write_env_file(file_path: &str, env_vars: &HashMap<String, String>) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)?;

    let mut written_keys = HashSet::new();

    // Read the original content and parse it
    let original_content = fs::read_to_string(file_path).unwrap_or_default();
    let ast = parse(&original_content);

    // Write nodes, updating existing variables in place and keeping the original order
    for node in ast.iter() {
        match node {
            Node::KeyValue {
                key,
                value,
                trailing_comment,
            } => {
                if !key.is_empty() {
                    if let Some(new_value) = env_vars.get(key) {
                        writeln!(file, "{}={}{}", key, quote_value(new_value), trailing_comment.as_ref().map_or(String::new(), |c| format!(" {}", c)))?;
                        written_keys.insert(key.to_string());
                    } else {
                        writeln!(file, "{}={}{}", key, quote_value(value), trailing_comment.as_ref().map_or(String::new(), |c| format!(" {}", c)))?;
                    }
                }
            }
            Node::Comment(comment) => writeln!(file, "{}", comment)?,
            Node::EmptyLine => writeln!(file)?,
        }
    }

    // Write new variables at the end
    let mut new_vars: Vec<_> = env_vars
        .iter()
        .filter(|(key, _)| !written_keys.contains(*key))
        .collect();
    new_vars.sort_by(|a, b| a.0.cmp(b.0));
    for (key, value) in new_vars {
        if !key.is_empty() {
            writeln!(file, "{}={}", key, quote_value(value))?;
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
            let parts: Vec<&str> = var.splitn(2, '=').collect();
            if parts.len() == 2 {
                let value = remove_surrounding_quotes(parts[1]);
                Some((parts[0].to_string(), value))
            } else {
                println!("Invalid argument: {}. Skipping.", var);
                None
            }
        })
        .collect()
}

fn remove_surrounding_quotes(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() >= 2
        && ((chars[0] == '\'' && chars[chars.len() - 1] == '\'')
            || (chars[0] == '"' && chars[chars.len() - 1] == '"'))
    {
        chars[1..chars.len() - 1].iter().collect()
    } else {
        s.to_string()
    }
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
    match fs::read_to_string(file_path) {
        Ok(content) => {
            let ast = parse(&content);
            for node in ast.iter() {
                match node {
                    Node::KeyValue {
                        key,
                        value,
                        trailing_comment,
                    } => {
                        let quoted_value = quote_value(value);
                        let line = format!("{}={}", key, quoted_value);
                        if let Some(comment) = trailing_comment {
                            writeln!(writer, "{} {}", line.blue().bold(), comment.green()).unwrap();
                        } else {
                            writeln!(writer, "{}", line.blue().bold()).unwrap();
                        }
                    }
                    Node::Comment(comment) => {
                        writeln!(writer, "{}", comment.green()).unwrap();
                    }
                    Node::EmptyLine => {
                        writeln!(writer).unwrap();
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Error reading .env file");
        }
    }
}

pub fn print_all_keys(file_path: &str) {
    print_all_keys_to_writer(file_path, &mut std::io::stdout());
}

pub fn print_all_keys_to_writer<W: Write>(file_path: &str, writer: &mut W) {
    if let Ok(env_vars) = read_env_vars(file_path) {
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
    let content = fs::read_to_string(file_path)?;
    let ast = parse::parse(&content);

    let updated_nodes: Vec<parse::Node> = ast
        .iter()
        .filter(|node| {
            if let parse::Node::KeyValue { key, .. } = node {
                !keys.contains(key)
            } else {
                true
            }
        })
        .cloned()
        .collect();

    let updated_content = updated_nodes
        .iter()
        .map(|node| match node {
            parse::Node::KeyValue {
                key,
                value,
                trailing_comment,
            } => {
                let comment = trailing_comment
                    .as_ref()
                    .map_or(String::new(), |c| format!(" {}", c));
                format!("{}={}{}", key, quote_value(value), comment)
            }
            parse::Node::Comment(comment) => comment.clone(),
            parse::Node::EmptyLine => String::new(),
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Ensure there's always a trailing newline
    let final_content = if updated_content.ends_with('\n') {
        updated_content
    } else {
        updated_content + "\n"
    };

    fs::write(file_path, final_content)
}

fn needs_quoting(value: &str) -> bool {
    value.contains(char::is_whitespace)
        || value.contains('\'')
        || value.contains('"')
        || value.contains('\\')
        || value.contains('$')
        || value.contains('#')
        || value.is_empty()
}

fn quote_value(value: &str) -> String {
    if needs_quoting(value) {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}
