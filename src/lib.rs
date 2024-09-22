mod parser;

use chumsky::Parser;
use colored::Colorize;
use serde_json::{self, json};
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

pub fn print_env_file(file_path: &str, env_vars: &HashMap<String, String>) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path).unwrap_or_default();
    let mut lines = parser::parser().parse(&*content).map_err(|e| {
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

    let mut buffer = Vec::new();
    print_lines(&lines, &mut buffer);

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

pub fn parse_args(vars: &[String]) -> HashMap<String, String> {
    vars.iter()
        .filter_map(|arg| {
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
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

pub fn print_env_vars<W: Write>(file_path: &str, writer: &mut W) {
    match fs::read_to_string(file_path) {
        Ok(content) => match parser::parser().parse(content) {
            Ok(lines) => {
                print_lines(&lines, writer);
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

pub fn print_lines<W: Write>(lines: &[parser::Line], writer: &mut W) {
    for line in lines {
        match line {
            parser::Line::Comment(comment) => {
                writeln!(writer, "#{}", comment).unwrap();
            }
            parser::Line::KeyValue {
                key,
                value,
                comment,
            } => {
                let quoted_value = quote_value(value);
                let mut line = format!("{}={}", key, quoted_value);
                if let Some(comment) = comment {
                    line.push_str(&format!(" #{}", comment));
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

pub fn print_diff<W: Write>(
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

pub fn delete_keys(file_path: &str, keys: &[String]) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path)?;
    let lines = parser::parser().parse(&*content).map_err(|e| {
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

    let mut buffer = Vec::new();
    print_lines(&updated_lines, &mut buffer);

    fs::write(file_path, buffer)
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
                '\n' => {
                    quoted.push_str("\\n");
                }
                '\r' => {
                    quoted.push_str("\\r");
                }
                '\t' => {
                    quoted.push_str("\\t");
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
