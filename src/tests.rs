use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use tempfile::tempdir;

use crate::{Cli, Commands};
use envset::{
    parse_args, parse_env_content, parse_stdin_with_reader, print_all_env_vars_to_writer,
    print_all_keys_to_writer, print_diff_to_writer, read_env_file, write_env_file,
};

#[test]
fn test_parse_stdin() {
    let input = "KEY1=value1\nKEY2=value2\n# Comment\nKEY3='value3'\n";
    let result = parse_env_content(input);
    assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(result.get("KEY3"), Some(&"value3".to_string()));
    assert_eq!(result.len(), 3);
}

#[test]
fn test_write_vars_with_quotes() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    let mut env_vars = HashMap::new();
    env_vars.insert("KEY1".to_string(), r#"value with "quotes""#.to_string());
    env_vars.insert("KEY2".to_string(), r#"value with 'quotes'"#.to_string());
    env_vars.insert(
        "KEY3".to_string(),
        r#"value with both 'single' and "double" quotes"#.to_string(),
    );

    write_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Read the file contents
    let contents = fs::read_to_string(&file_path).unwrap();

    // Print out the file contents for debugging
    println!("File contents:\n{}", contents);

    // Read the file using read_env_file and check the result
    let result = read_env_file(file_path.to_str().unwrap()).unwrap();

    // Print out the environment variables for debugging
    println!("Environment variables:");
    for (key, value) in &result {
        println!("{}: {}", key, value);
    }

    // Print out the environment variables for debugging
    println!("Environment variables:");
    for (key, value) in &result {
        println!("{}: {}", key, value);
    }

    assert_eq!(
        result.get("KEY1"),
        Some(&r#"value with "quotes""#.to_string())
    );
    assert_eq!(
        result.get("KEY2"),
        Some(&r#"value with 'quotes'"#.to_string())
    );
    assert_eq!(
        result.get("KEY3"),
        Some(&r#"value with both 'single' and "double" quotes"#.to_string())
    );

    // Check the file contents directly
    let file_contents = fs::read_to_string(&file_path).unwrap();
    println!("File contents after writing:\n{}", file_contents);
    assert!(file_contents.contains(r#"KEY1="value with \"quotes\"""#));
    assert!(file_contents.contains(r#"KEY2="value with 'quotes'""#));
    assert!(file_contents.contains(r#"KEY3="value with both 'single' and \"double\" quotes""#));
}

#[test]
fn test_parse_args() {
    let args = vec![
        "KEY1=value1".to_string(),
        "KEY2='value2'".to_string(),
        "KEY3=\"value3\"".to_string(),
    ];
    let result = parse_args(&args);
    assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(result.get("KEY3"), Some(&"value3".to_string()));
    assert_eq!(result.len(), 3);
}

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
    assert!(contents.contains("# Comment"));
    assert!(contents.contains("EXISTING=old"));
    assert!(contents.contains("KEY1=value1"));
    assert!(contents.contains("KEY2=value2"));
}

#[test]
fn test_preserve_comments() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(
        file,
        "# example comment\nFOO='bar'\n# another comment\nBAZ=qux"
    )
    .unwrap();

    let result = read_env_file(file_path.to_str().unwrap()).unwrap();
    assert_eq!(result.get("FOO"), Some(&"bar".to_string()));
    assert_eq!(result.get("BAZ"), Some(&"qux".to_string()));

    // Re-read the file to check if comments are preserved
    let contents = fs::read_to_string(file_path).unwrap();
    assert!(contents.contains("# example comment"));
    assert!(contents.contains("# another comment"));
    assert!(contents.contains("FOO='bar'"));
    assert!(contents.contains("BAZ=qux"));
}

#[test]
fn test_parse_stdin_with_pipe() {
    let input = "KEY1=value1\nKEY2=value2\n";
    let mut cursor = Cursor::new(input);
    let result = parse_stdin_with_reader(&mut cursor);
    assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(result.len(), 2);
}

#[test]
fn test_set_quoted_values_through_args() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    let args = vec![
        "KEY1=simple value".to_string(),
        r#"KEY2="quoted value""#.to_string(),
        r#"KEY3='single quoted'"#.to_string(),
        r#"KEY4="nested "quotes" 'here'""#.to_string(),
        r#"FOO="bing "baz" 'bar'""#.to_string(),
    ];
    let result = parse_args(&args);

    write_env_file(file_path.to_str().unwrap(), &result).unwrap();

    let env_vars = read_env_file(file_path.to_str().unwrap()).unwrap();

    assert_eq!(env_vars.get("KEY1"), Some(&"simple value".to_string()));
    assert_eq!(env_vars.get("KEY2"), Some(&"quoted value".to_string()));
    assert_eq!(env_vars.get("KEY3"), Some(&"single quoted".to_string()));
    assert_eq!(
        env_vars.get("KEY4"),
        Some(&r#"nested "quotes" 'here'"#.to_string())
    );
    assert_eq!(
        env_vars.get("FOO"),
        Some(&r#"bing "baz" 'bar'"#.to_string())
    );
}

#[test]
fn test_parse_stdin_and_write_to_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    let input = "KEY1=value1\nKEY2=value2\n";
    let mut cursor = Cursor::new(input);
    let result = parse_stdin_with_reader(&mut cursor);

    // Write the result to the temporary file
    write_env_file(file_path.to_str().unwrap(), &result).unwrap();

    // Read the file contents
    let contents = fs::read_to_string(&file_path).unwrap();

    // Check if the file contains the expected content
    assert!(contents.contains("KEY1=value1"));
    assert!(contents.contains("KEY2=value2"));

    // Read the file using read_env_file and check the result
    let env_vars = read_env_file(file_path.to_str().unwrap()).unwrap();
    assert_eq!(env_vars.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(env_vars.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(env_vars.len(), 2);
}

#[test]
fn test_parse_args_removes_surrounding_quotes() {
    let args = vec![
        "FOO=bing 'baz'".to_string(),
        "BING=bar".to_string(),
        "KEY='val'".to_string(),
    ];
    let result = parse_args(&args);
    assert_eq!(result.get("FOO"), Some(&"bing 'baz'".to_string()));
    assert_eq!(result.get("BING"), Some(&"bar".to_string()));
    assert_eq!(result.get("KEY"), Some(&"val".to_string()));
}

#[test]
fn test_print_diff_multiple_vars() {
    let mut original = HashMap::new();
    original.insert("KEY1".to_string(), "old_value1".to_string());
    original.insert("KEY2".to_string(), "old_value2".to_string());
    original.insert("KEY3".to_string(), "value3".to_string());

    let mut updated = HashMap::new();
    updated.insert("KEY1".to_string(), "new_value1".to_string());
    updated.insert("KEY2".to_string(), "new_value2".to_string());
    updated.insert("KEY3".to_string(), "value3".to_string());
    updated.insert("KEY4".to_string(), "new_value4".to_string());

    let mut output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut output);
        print_diff_to_writer(&original, &updated, &mut cursor);
    }

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("-KEY1=old_value1"));
    assert!(output_str.contains("+KEY1=new_value1"));
    assert!(output_str.contains("-KEY2=old_value2"));
    assert!(output_str.contains("+KEY2=new_value2"));
    assert!(output_str.contains("+KEY4=new_value4"));
    assert!(!output_str.contains("KEY3=value3"));
}

#[test]
fn test_multiple_var_sets() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    // First set ABCD=123
    let mut env_vars = HashMap::new();
    env_vars.insert("ABCD".to_string(), "123".to_string());
    let original_lines = Vec::new();
    write_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Then set AB=12
    env_vars.insert("AB".to_string(), "12".to_string());
    let _ = read_env_file(file_path.to_str().unwrap()).unwrap();
    write_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Read the final state of the file
    let result = read_env_file(file_path.to_str().unwrap()).unwrap();

    // Assert that both variables are set
    assert_eq!(result.get("ABCD"), Some(&"123".to_string()));
    assert_eq!(result.get("AB"), Some(&"12".to_string()));
    assert_eq!(result.len(), 2);

    // Check the final content of the file
    let final_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(final_content, "AB=12\nABCD=123\n");
}

#[test]
fn test_keep_one_occurrence_of_duplicate_keys() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    // Create an initial .env file with duplicate keys
    let initial_content = "A=a\nFOO=1\nB=b\nFOO=2\n";
    fs::write(&file_path, initial_content).unwrap();

    // TODO implement test
}

#[test]
fn test_delete_env_vars() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let initial_content = "FOO=bar\nBAZ=qux\nQUUX=quux\n";
    fs::write(&file_path, initial_content).unwrap();

    let keys_to_delete = vec!["FOO".to_string(), "QUUX".to_string()];
    envset::delete_env_vars(file_path.to_str().unwrap(), &keys_to_delete).unwrap();

    let final_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        final_content, "BAZ=qux\n",
        "Final content should only contain BAZ=qux"
    );

    let result = read_env_file(file_path.to_str().unwrap()).unwrap();
    assert!(!result.contains_key("FOO"), "FOO should be deleted");
    assert!(result.contains_key("BAZ"), "BAZ should still exist");
    assert!(!result.contains_key("QUUX"), "QUUX should be deleted");
    assert_eq!(result.len(), 1, "Only one key should remain");
}

#[test]
fn test_get_single_env_var() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar\nBAZ=qux").unwrap();

    let (env_vars, _) = read_env_file(file_path.to_str().unwrap()).unwrap();
    assert_eq!(env_vars.get("FOO"), Some(&"bar".to_string()));
    assert_eq!(env_vars.get("BAZ"), Some(&"qux".to_string()));
}

#[test]
fn test_print_all_env_vars() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar\nBAZ=qux\nABC=123").unwrap();

    let mut output = Vec::new();
    print_all_env_vars_to_writer(file_path.to_str().unwrap(), &mut output);

    let output_str = String::from_utf8(output).unwrap();

    let plain_bytes = strip_ansi_escapes::strip(&output_str);
    let stripped_output = String::from_utf8_lossy(&plain_bytes);

    assert_eq!(
        stripped_output.trim(),
        "FOO=bar\nBAZ=qux\nABC=123",
        "Output should match the input file content"
    );
}

#[test]
fn test_no_print_when_args_provided() {
    use clap::Parser;
    use std::io::Cursor;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar\nBAZ=qux").unwrap();

    // Simulate command-line arguments with vars
    let args = vec![
        "envset",
        "--file",
        file_path.to_str().unwrap(),
        "NEW_VAR=value",
    ];
    let cli = Cli::parse_from(args);

    // Capture stdout
    let mut output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut output);

        // Run the main logic
        if cli.command.is_none() && !cli.vars.is_empty() {
            // This is where we would normally set the environment variables
            // For this test, we're just ensuring it doesn't print
        } else {
            print_all_env_vars_to_writer(file_path.to_str().unwrap(), &mut cursor);
        }
    }

    let output_str = String::from_utf8(output).unwrap();
    assert!(
        output_str.is_empty(),
        "Output should be empty when args are provided"
    );
}

#[test]
fn test_print_all_keys() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar\nBAZ=qux").unwrap();

    let mut output = Vec::new();
    print_all_keys_to_writer(file_path.to_str().unwrap(), &mut output);

    let output_str = String::from_utf8(output).unwrap();
    assert!(output_str.contains("FOO"), "Output does not contain FOO");
    assert!(output_str.contains("BAZ"), "Output does not contain BAZ");
    assert!(!output_str.contains("="), "Output should not contain '='");
}

#[test]
fn test_print_when_no_args() {
    use clap::Parser;
    use std::io::Cursor;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar\nBAZ=qux").unwrap();

    // Simulate command-line arguments
    let args = vec!["envset", "--file", file_path.to_str().unwrap()];
    let cli = Cli::parse_from(args);

    // Capture stdout
    let mut output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut output);

        // Run the main logic
        match &cli.command {
            Some(Commands::Print) | None => {
                print_all_env_vars_to_writer(file_path.to_str().unwrap(), &mut cursor);
            }
            _ => panic!("Unexpected command"),
        }
    }

    let output_str = String::from_utf8(output).unwrap();
    assert!(
        output_str.contains("FOO") && output_str.contains("bar"),
        "Output does not contain FOO and bar"
    );
    assert!(
        output_str.contains("BAZ") && output_str.contains("qux"),
        "Output does not contain BAZ and qux"
    );
}

#[test]
fn test_no_print_when_vars_set_via_stdin() {
    use clap::Parser;
    use std::io::Cursor;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "EXISTING=value").unwrap();

    // Simulate command-line arguments
    let args = vec!["envset", "--file", file_path.to_str().unwrap()];
    let cli = Cli::parse_from(args);

    // Simulate stdin input
    let stdin_input = "NEW_VAR=new_value\n";
    let mut stdin = Cursor::new(stdin_input);

    // Capture stdout
    let mut output = Vec::new();
    {
        let mut stdout = Cursor::new(&mut output);

        // Run the main logic
        let new_vars = parse_stdin_with_reader(&mut stdin);
        if !new_vars.is_empty() {
            let mut env_vars = read_env_file(file_path.to_str().unwrap()).unwrap();
            let original_env = env_vars.clone();
            env_vars.extend(new_vars);
            write_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();
            print_diff_to_writer(&original_env, &env_vars, &mut stdout);
        } else if cli.command.is_none() {
            print_all_env_vars_to_writer(file_path.to_str().unwrap(), &mut stdout);
        }
    }

    let output_str = String::from_utf8(output).unwrap();
    assert!(
        output_str.contains("+NEW_VAR=new_value"),
        "Diff output should show the new variable"
    );
    assert!(
        !output_str.contains("EXISTING=value"),
        "Full env vars should not be printed"
    );
}
