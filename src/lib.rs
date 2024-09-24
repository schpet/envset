mod parser;

use chumsky::Parser;
use colored::Colorize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn read_env_vars(file_path: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let path = Path::new(file_path);

    if path.exists() {
        let contents = fs::read_to_string(path)?;
        Ok(parse_env_content(&contents))
    } else {
        // Create an empty .env file if it doesn't exist
        fs::write(path, "")?;
        Ok(HashMap::new())
    }
}

pub fn print_parse_tree<W: Write>(file_path: &str, writer: &mut W) {
    match fs::read_to_string(file_path) {
        Ok(content) => match parser::parser().parse(content) {
            Ok(lines) => {
                let json = serde_json::to_string_pretty(&lines).unwrap();
                writeln!(writer, "{}", json).unwrap();
            }
            Err(e) => {
                eprintln!("Error parsing .env file: {:?}", e);
            }
        },
        Err(e) => {
            eprintln!("Error reading .env file: {:?}", e);
        }
    }
}

pub fn print_env_vars_as_json<W: Write>(file_path: &str, writer: &mut W) {
    match read_env_vars(file_path) {
        Ok(env_vars) => {
            let json_output = json!(env_vars);
            writeln!(
                writer,
                "{}",
                serde_json::to_string_pretty(&json_output).unwrap()
            )
            .unwrap();
        }
        Err(e) => {
            eprintln!("Error reading .env file: {:?}", e);
        }
    }
}

pub fn read_env_file_contents(file_path: &str) -> std::io::Result<String> {
    fs::read_to_string(file_path)
}

pub fn add_env_vars(
    content: &str,
    env_vars: &HashMap<String, String>,
) -> Result<Vec<parser::Line>, std::io::Error> {
    let mut lines = parser::parser().parse(content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing .env file: {:?}", e),
        )
    })?;

    // Replace the last instance of each key in place
    for (key, value) in env_vars {
        let mut last_index = None;
        for (index, line) in lines.iter().enumerate().rev() {
            if let parser::Line::KeyValue { key: line_key, .. } = line {
                if line_key == key {
                    last_index = Some(index);
                    break;
                }
            }
        }

        if let Some(index) = last_index {
            lines[index] = parser::Line::KeyValue {
                key: key.clone(),
                value: value.clone(),
                comment: None,
            };
        } else {
            // If the key doesn't exist, add it at the end
            lines.push(parser::Line::KeyValue {
                key: key.clone(),
                value: value.clone(),
                comment: None,
            });
        }
    }

    Ok(lines)
}

pub fn print_env_file_contents<W: Write>(
    lines: &[parser::Line],
    writer: &mut W,
) -> std::io::Result<()> {
    print_lines(lines, writer, false);
    Ok(())
}

pub fn update_env_file(file_path: &str, env_vars: &HashMap<String, String>) -> std::io::Result<()> {
    let content = read_env_file_contents(file_path).unwrap_or_default();
    let lines = add_env_vars(&content, env_vars)?;
    let mut buffer = Vec::new();
    print_env_file_contents(&lines, &mut buffer)?;
    fs::write(file_path, buffer)
}

pub fn parse_stdin() -> HashMap<String, String> {
    parse_stdin_with_reader(&mut io::stdin())
}

pub fn parse_stdin_with_reader<R: Read>(reader: &mut R) -> HashMap<String, String> {
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();
    parse_env_content(&buffer)
}

pub fn parse_args(vars: &[String]) -> Result<HashMap<String, String>, String> {
    vars.iter().try_fold(HashMap::new(), |mut acc, arg| {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            acc.insert(parts[0].to_string(), parts[1].to_string());
            Ok(acc)
        } else {
            Err(format!(
                "Invalid argument format {}. Expected format is {}",
                arg.bold().red(),
                "KEY=value".bold()
            ))
        }
    })
}

pub fn parse_env_content(content: &str) -> HashMap<String, String> {
    match parser::parser().parse(content) {
        Ok(lines) => lines
            .into_iter()
            .filter_map(|line| {
                if let parser::Line::KeyValue { key, value, .. } = line {
                    Some((key, value))
                } else {
                    None
                }
            })
            .collect(),
        Err(e) => {
            eprintln!("Error parsing .env content: {:?}", e);
            HashMap::new()
        }
    }
}

pub fn print_env_vars<W: Write>(file_path: &str, writer: &mut W, use_color: bool) {
    match fs::read_to_string(file_path) {
        Ok(content) => match parser::parser().parse(content) {
            Ok(lines) => {
                print_lines(&lines, writer, use_color);
            }
            Err(e) => {
                eprintln!("Error parsing .env file: {:?}", e);
            }
        },
        Err(_) => {
            eprintln!("Error reading .env file");
        }
    }
}

pub fn print_lines<W: Write>(lines: &[parser::Line], writer: &mut W, use_color: bool) {
    for line in lines {
        match line {
            parser::Line::Comment(comment) => {
                let comment_str = if use_color {
                    format!("#{}", comment).bright_black().to_string()
                } else {
                    format!("#{}", comment)
                };
                writeln!(writer, "{}", comment_str).unwrap();
            }
            parser::Line::KeyValue {
                key,
                value,
                comment,
            } => {
                let key_str = if use_color {
                    key.blue().to_string()
                } else {
                    key.to_string()
                };
                let quoted_value = quote_value(value);
                let value_str = if use_color {
                    quoted_value.green().to_string()
                } else {
                    quoted_value
                };
                let mut line = format!("{}={}", key_str, value_str);
                if let Some(comment) = comment {
                    let comment_str = if use_color {
                        format!(" #{}", comment).bright_black().to_string()
                    } else {
                        format!(" #{}", comment)
                    };
                    line.push_str(&comment_str);
                }
                writeln!(writer, "{}", line).unwrap();
            }
        }
    }
}

pub fn print_env_keys_to_writer<W: Write>(file_path: &str, writer: &mut W) {
    if let Ok(env_vars) = read_env_vars(file_path) {
        for key in env_vars.keys() {
            writeln!(writer, "{}", key).unwrap();
        }
    } else {
        eprintln!("Error reading .env file");
    }
}

pub fn delete_env_vars(
    content: &str,
    keys: &[String],
) -> Result<Vec<parser::Line>, std::io::Error> {
    let lines = parser::parser().parse(content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing .env file: {:?}", e),
        )
    })?;

    let updated_lines: Vec<parser::Line> = lines
        .into_iter()
        .filter(|line| {
            if let parser::Line::KeyValue { key, .. } = line {
                !keys.contains(key)
            } else {
                true
            }
        })
        .collect();

    Ok(updated_lines)
}

pub fn format_env_file(content: &str, prune: bool) -> Result<Vec<parser::Line>, std::io::Error> {
    let lines = parser::parser().parse(content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing .env file: {:?}", e),
        )
    })?;

    let mut key_value_lines: Vec<parser::Line> = lines
        .into_iter()
        .filter(|line| match line {
            parser::Line::KeyValue { value, .. } => !value.is_empty(),
            parser::Line::Comment(_) => !prune,
        })
        .collect();

    key_value_lines.sort_by(|a, b| {
        if let (
            parser::Line::KeyValue { key: key_a, .. },
            parser::Line::KeyValue { key: key_b, .. },
        ) = (a, b)
        {
            key_a.cmp(key_b)
        } else {
            std::cmp::Ordering::Equal
        }
    });

    Ok(key_value_lines)
}

fn needs_quoting(value: &str) -> bool {
    value.chars().any(|c| {
        c.is_whitespace()
            || c == '\''
            || c == '"'
            || c == '\\'
            || c == '$'
            || c == '#'
            || c < ' '
            || c as u32 > 127
    }) || value.is_empty()
}

fn quote_value(value: &str) -> String {
    if needs_quoting(value) {
        let mut quoted = String::with_capacity(value.len() + 2);
        quoted.push('"');
        for c in value.chars() {
            match c {
                '"' | '\\' => {
                    quoted.push('\\');
                    quoted.push(c);
                }
                '\n' | '\r' | '\t' => {
                    quoted.push(c);
                }
                _ => {
                    quoted.push(c);
                }
            }
        }
        quoted.push('"');
        quoted
    } else {
        value.to_string()
    }
}
