use atty::Stream;
use clap::Parser;
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::process;

use envset::{
    add_env_vars, parse_args, parse_stdin, print_env_file_contents, print_env_keys_to_writer,
    print_env_vars, print_env_vars_as_json, print_keys_from_map, print_parse_tree,
    read_env_file_contents, read_env_vars,
};

fn print_diff(old_content: &str, new_content: &str, use_color: bool) {
    let diff = TextDiff::from_lines(old_content, new_content);
    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);

    for change in diff.iter_all_changes() {
        if use_color {
            match change.tag() {
                ChangeTag::Delete => {
                    let line = change.to_string();
                    let padding = " ".repeat(term_width.saturating_sub(line.trim_end().len()));
                    print!(
                        "{}",
                        (line.trim_end().to_string() + &padding).on_bright_red()
                    );
                    println!();
                }
                ChangeTag::Insert => {
                    let line = change.to_string();
                    let padding = " ".repeat(term_width.saturating_sub(line.trim_end().len()));
                    print!(
                        "{}",
                        (line.trim_end().to_string() + &padding).on_bright_green()
                    );
                    println!();
                }
                ChangeTag::Equal => print!("{}", change),
            }
        } else {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            print!("{}{}", sign, change);
        }
    }
}

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// File path for the .env file
    #[arg(short = 'f', long = "file", default_value = ".env", global = true)]
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
        /// Print the JSON representation of the parse tree
        #[arg(short = 'p', long = "parse-tree")]
        parse_tree: bool,
        /// Print the environment variables as a JSON object
        #[arg(short = 'j', long = "json")]
        json: bool,
    },
    /// Print all keys in the .env file
    Keys,
    /// Delete specified environment variables
    #[command(alias = "rm")]
    Delete {
        /// Keys to delete
        #[arg(required = true)]
        keys: Vec<String>,
    },
    /// Format the .env file (sort keys and remove empty lines)
    Fmt {
        /// Remove whole line comments
        #[arg(short = 'p', long = "prune")]
        prune: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let mut should_print = cli.command.is_none() && cli.vars.is_empty();

    match &cli.command {
        Some(Commands::Get { key }) => match read_env_vars(&cli.file) {
            Ok(env_vars) => match env_vars.get(key) {
                Some(value) => println!("{}", value),
                None => {
                    eprintln!("Environment variable '{}' not found", key);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading .env file: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Print { parse_tree, json }) => {
            let use_color = atty::is(Stream::Stdout);
            if *parse_tree {
                print_parse_tree(&cli.file, &mut std::io::stdout());
            } else if *json {
                print_env_vars_as_json(&cli.file, &mut std::io::stdout());
            } else {
                print_env_vars(&cli.file, &mut std::io::stdout(), use_color);
            }
            return; // Exit after printing
        }
        Some(Commands::Keys) => {
            if !atty::is(Stream::Stdin) {
                let env_vars = parse_stdin();
                print_keys_from_map(&env_vars, &mut std::io::stdout());
            } else {
                print_env_keys_to_writer(&cli.file, &mut std::io::stdout());
            }
        }
        Some(Commands::Delete { keys }) => match read_env_file_contents(&cli.file) {
            Ok(old_content) => match envset::delete_env_vars(&old_content, keys) {
                Ok(updated_lines) => {
                    let mut buffer = Vec::new();
                    if let Err(e) = print_env_file_contents(&updated_lines, &mut buffer) {
                        eprintln!("Error writing .env file contents: {}", e);
                        process::exit(1);
                    }
                    let new_content = String::from_utf8_lossy(&buffer);

                    if old_content == new_content {
                        eprintln!(
                            "No environment variables found to delete. Attempted to delete: {}",
                            keys.join(", ")
                        );
                        process::exit(1);
                    }

                    let use_color = atty::is(Stream::Stdout);
                    print_diff(&old_content, &new_content, use_color);

                    if let Err(e) = std::fs::write(&cli.file, buffer) {
                        eprintln!("Error writing .env file: {}", e);
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error deleting environment variables: {}", e);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading .env file: {}", e);
                process::exit(1);
            }
        },
        Some(Commands::Fmt { prune }) => match read_env_file_contents(&cli.file) {
            Ok(old_content) => match envset::format_env_file(&old_content, *prune) {
                Ok(formatted_lines) => {
                    let mut buffer = Vec::new();
                    if let Err(e) = print_env_file_contents(&formatted_lines, &mut buffer) {
                        eprintln!("Error writing formatted .env file contents: {}", e);
                        process::exit(1);
                    }
                    let new_content = String::from_utf8_lossy(&buffer);

                    let use_color = atty::is(Stream::Stdout);
                    print_diff(&old_content, &new_content, use_color);

                    if let Err(e) = std::fs::write(&cli.file, buffer) {
                        eprintln!("Error writing formatted .env file: {}", e);
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error formatting .env file: {}", e);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading .env file: {}", e);
                process::exit(1);
            }
        },
        None => {}
    }

    let new_vars = if !atty::is(Stream::Stdin) || !cli.vars.is_empty() {
        if !atty::is(Stream::Stdin) {
            parse_stdin()
        } else {
            match parse_args(&cli.vars) {
                Ok(vars) => vars,
                Err(e) => {
                    eprintln!("Error parsing arguments: {}", e);
                    process::exit(1);
                }
            }
        }
    } else {
        HashMap::new()
    };

    if !new_vars.is_empty() {
        should_print = false; // Don't print all vars when setting new ones
        let mut env_vars = read_env_vars(&cli.file).unwrap_or_else(|e| {
            eprintln!("Error reading .env file: {}", e);
            process::exit(1);
        });

        env_vars.extend(new_vars);

        match read_env_file_contents(&cli.file) {
            Ok(old_content) => match add_env_vars(&old_content, &env_vars) {
                Ok(updated_lines) => {
                    let mut buffer = Vec::new();
                    if let Err(e) = print_env_file_contents(&updated_lines, &mut buffer) {
                        eprintln!("Error writing .env file contents: {}", e);
                        process::exit(1);
                    }
                    let new_content = String::from_utf8_lossy(&buffer);

                    let use_color = atty::is(Stream::Stdout);
                    print_diff(&old_content, &new_content, use_color);

                    if let Err(e) = std::fs::write(&cli.file, buffer) {
                        eprintln!("Error writing .env file: {}", e);
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error updating .env file contents: {}", e);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading .env file: {}", e);
                process::exit(1);
            }
        }
    }

    if should_print {
        let use_color = atty::is(Stream::Stdout);
        print_env_vars(&cli.file, &mut std::io::stdout(), use_color);
    }
}
