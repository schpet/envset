use clap::Parser;
use colored::Colorize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process;

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Overwrite existing variables
    #[arg(short, long)]
    force: bool,

    /// Output file path
    #[arg(short, long, default_value = ".env")]
    output: String,

    /// KEY=value pairs to set
    #[arg(required = true)]
    vars: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    let force = cli.force;
    let vars = cli.vars;
    let env_file = &cli.output;
    let (mut env_vars, original_lines) = match read_env_file(env_file) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error reading .env file: {}", e);
            process::exit(1);
        }
    };

    let mut warnings = Vec::new();

    let new_vars: HashMap<String, String> = vars
        .into_iter()
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
        .collect();

    for (key, value) in &new_vars {
        if !env_vars.contains_key(key) || force {
            env_vars.insert(key.clone(), value.clone());
            println!("Set {}={}", key, value);
        } else {
            warnings.push(format!(
                "Warning: Environment variable '{}' is already set. Use --force to overwrite.",
                key
            ));
        }
    }

    if let Err(e) = write_env_file(env_file, &env_vars, &original_lines) {
        eprintln!("Error writing .env file: {}", e);
        process::exit(1);
    }

    // Print warnings after writing the file
    for warning in &warnings {
        eprintln!("{}", warning.yellow());
    }

    // Exit with an error code if there were any warnings
    if !warnings.is_empty() {
        process::exit(1);
    }
}

fn read_env_file(file_path: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let path = Path::new(file_path);
    let mut env_vars = HashMap::new();

    if path.exists() {
        let contents = fs::read_to_string(path)?;
        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                if !line.trim_start().starts_with('#') {
                    env_vars.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    Ok(env_vars)
}

fn write_env_file(file_path: &str, env_vars: &HashMap<String, String>) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)?;

    for (key, value) in env_vars {
        writeln!(file, "{}={}", key, value)?;
    }

    Ok(())
}
