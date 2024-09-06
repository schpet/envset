use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

use super::*;

#[test]
fn test_read_env_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "KEY1=value1\nKEY2=value2").unwrap();

    let result = read_env_file(file_path.to_str().unwrap()).unwrap();
    assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
}

#[test]
fn test_write_env_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut env_vars = HashMap::new();
    env_vars.insert("KEY1".to_string(), "value1".to_string());
    env_vars.insert("KEY2".to_string(), "value2".to_string());

    write_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    let contents = fs::read_to_string(file_path).unwrap();
    assert!(contents.contains("KEY1=value1"));
    assert!(contents.contains("KEY2=value2"));
}

#[test]
fn test_parse_vars() {
    let vars = vec![
        "KEY1=value1".to_string(),
        "KEY2='value 2'".to_string(),
        "KEY3=\"value 3\"".to_string(),
        "INVALID".to_string(),
    ];

    let result: HashMap<String, String> = vars
        .into_iter()
        .filter_map(|var| {
            let mut parts = var.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((
                    key.trim().to_string(),
                    value.trim().trim_matches('\'').trim_matches('"').to_string(),
                )),
                _ => None,
            }
        })
        .collect();

    assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value 2".to_string()));
    assert_eq!(result.get("KEY3"), Some(&"value 3".to_string()));
    assert_eq!(result.get("INVALID"), None);
}
