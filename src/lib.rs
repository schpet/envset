mod charser;

use chumsky::Parser;
use colored::Colorize;
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

pub fn write_env_file(file_path: &str, env_vars: &HashMap<String, String>) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path).unwrap_or_default();
    let mut lines = charser::parser().parse(&*content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing .env file: {:?}", e),
        )
    })?;

    // Add new variables at the end
    for (key, value) in env_vars {
        if !lines
            .iter()
            .any(|line| matches!(line, charser::Line::KeyValue { key: k, .. } if k == key))
        {
            lines.push(charser::Line::KeyValue {
                key: key.clone(),
                value: value.clone(),
                comment: None,
            });
        }
    }

    let mut buffer = Vec::new();
    write_chumsky_ast_to_writer(&lines, &mut buffer);

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
    // TODO replace this with a chumsky based parser, too?
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
    match charser::parser().parse(content) {
        Ok(lines) => lines
            .into_iter()
            .filter_map(|line| {
                if let charser::Line::KeyValue { key, value, .. } = line {
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

pub fn print_all_env_vars(file_path: &str) {
    print_all_env_vars_to_writer(file_path, &mut std::io::stdout());
}

pub fn print_all_env_vars_to_writer<W: Write>(file_path: &str, writer: &mut W) {
    match fs::read_to_string(file_path) {
        Ok(content) => match charser::parser().parse(content) {
            Ok(lines) => {
                write_chumsky_ast_to_writer(&lines, writer);
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

pub fn write_chumsky_ast_to_writer<W: Write>(lines: &[charser::Line], writer: &mut W) {
    for line in lines {
        match line {
            charser::Line::Comment(comment) => {
                writeln!(writer, "#{}", comment).unwrap();
            }
            charser::Line::KeyValue {
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
    let lines = charser::parser().parse(&*content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing .env file: {:?}", e),
        )
    })?;

    let updated_lines: Vec<charser::Line> = lines
        .into_iter()
        .filter(|line| {
            if let charser::Line::KeyValue { key, .. } = line {
                !keys.contains(key)
            } else {
                true
            }
        })
        .collect();

    let mut buffer = Vec::new();
    write_chumsky_ast_to_writer(&updated_lines, &mut buffer);

    fs::write(file_path, buffer)
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

pub fn parse_chumsky(file_path: &str) -> Result<Vec<charser::Line>, std::io::Error> {
    let content = std::fs::read_to_string(file_path)?;
    charser::parser().parse(content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Error parsing .env file with Chumsky: {:?}", e),
        )
    })
}
