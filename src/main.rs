use std::fs::{OpenOptions, File};
use std::io::{Write, Read};
use std::path::Path;
use std::collections::HashMap;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Overwrite existing variables
    #[arg(short, long)]
    force: bool,

    /// KEY=value pairs to set
    #[arg(required = true)]
    vars: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let force = cli.force;
    let vars = cli.vars;

    if vars.is_empty() {
        println!("Usage: envset [-f|--force] KEY1='value1' KEY2='value2' ...");
        println!("  -f, --force    Overwrite existing variables");
        return Ok(());
    }

    let env_file = ".env";
    let mut env_vars = read_env_file(env_file)?;
    let mut new_vars = HashMap::new();

    for var in vars {
        let parts: Vec<&str> = var.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim().trim_matches('\'').trim_matches('"');
            new_vars.insert(key.to_string(), value.to_string());
        } else {
            println!("Invalid argument: {}. Skipping.", var);
        }
    }

    for (key, value) in new_vars {
        if !env_vars.contains_key(&key) || force {
            env_vars.insert(key.clone(), value.clone());
            println!("Set {}={}", key, value);
        } else {
            println!("Skipped {} (use -f to overwrite)", key);
        }
    }

    write_env_file(env_file, &env_vars)?;

    Ok(())
}

fn read_env_file(file_path: &str) -> Result<HashMap<String, String>, std::io::Error> {
    let path = Path::new(file_path);
    let mut env_vars = HashMap::new();

    if path.exists() {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                env_vars.insert(key.trim().to_string(), value.trim().to_string());
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
