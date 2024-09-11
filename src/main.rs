use atty::Stream;
use clap::Parser;
use std::collections::HashMap;
use std::env;
use std::process;

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
    #[arg(short, long, default_value = ".env", global = true)]
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
        Some(Commands::Print) => {
            print_all_env_vars(&cli.file);
        }
        Some(Commands::Keys) => {
            print_all_keys(&cli.file);
        }
        None => {
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

            if let Err(e) = write_env_file(&cli.file, &env_vars, &original_lines) {
                eprintln!("Error writing .env file: {}", e);
                process::exit(1);
            }

            print_diff(&original_env, &env_vars);
        }
    }
}
