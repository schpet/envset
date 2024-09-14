use atty::Stream;
use clap::Parser;
use std::collections::HashMap;
use std::process;

use envset::parse::{parse, Ast, Node};
use envset::{
    parse_args, parse_stdin, print_all_env_vars, print_all_keys, print_diff, read_env_file,
    write_env_file,
};

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
    Print,
    /// Print all keys in the .env file
    Keys,
    /// Delete specified environment variables
    Delete {
        /// Keys to delete
        #[arg(required = true)]
        keys: Vec<String>,
    },
    /// Print the AST as JSON
    Ast,
}

fn main() {
    let cli = Cli::parse();

    let mut should_print = cli.command.is_none() && cli.vars.is_empty();

    match &cli.command {
        Some(Commands::Get { key }) => match read_env_file(&cli.file) {
            Ok((env_vars, _)) => match env_vars.get(key) {
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
        Some(Commands::Print) => {
            should_print = true;
        }
        Some(Commands::Keys) => {
            print_all_keys(&cli.file);
        }
        Some(Commands::Delete { keys }) => {
            let (env_vars, _) = match read_env_file(&cli.file) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error reading .env file: {}", e);
                    process::exit(1);
                }
            };

            let original_env = env_vars.clone();

            if let Err(e) = envset::delete_env_vars(&cli.file, keys) {
                eprintln!("Error deleting environment variables: {}", e);
                process::exit(1);
            }

            let (updated_env, _) = read_env_file(&cli.file).unwrap();
            print_diff(&original_env, &updated_env);
        }
        Some(Commands::Ast) => match std::fs::read_to_string(&cli.file) {
            Ok(content) => {
                let ast: Ast = parse(&content);
                match serde_json::to_string_pretty(&ast) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("Error serializing AST to JSON: {}", e);
                        process::exit(1);
                    }
                }
            }
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
            parse_args(&cli.vars)
        }
    } else {
        HashMap::new()
    };

    if !new_vars.is_empty() {
        should_print = false; // Don't print all vars when setting new ones
        let no_overwrite = cli.no_overwrite;
        let (mut env_vars, original_lines) = match read_env_file(&cli.file) {
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

        for (key, value) in &new_vars {
            if !env_vars.contains_key(key as &str) || !no_overwrite {
                env_vars.insert(key.clone(), value.clone());
            }
        }

        if let Err(e) = write_env_file(&cli.file, &env_vars, &original_lines) {
            eprintln!("Error writing .env file: {}", e);
            process::exit(1);
        }

        print_diff(&original_env, &env_vars);
    }

    if should_print {
        if atty::is(Stream::Stdout) {
            print_all_env_vars(&cli.file);
        } else {
            // If not outputting to a terminal, use a plain writer without colors
            let mut writer = std::io::stdout();
            envset::print_all_env_vars_to_writer(&cli.file, &mut writer);
        }
    }
}
