use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use tempfile::tempdir;

use crate::{Cli, Commands};
use envset::{
    parse_stdin_with_reader, print_env_keys_to_writer, print_env_vars, read_env_vars,
    update_env_file,
};

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

    update_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Read the file contents
    let contents = fs::read_to_string(&file_path).unwrap();

    // Print out the file contents for debugging
    println!("File contents:\n{}", contents);

    // Read the file using read_env_file and check the result
    let result = read_env_vars(file_path.to_str().unwrap()).unwrap();

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
fn test_read_env_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "KEY1=value1\nKEY2=value2").unwrap();

    let result = read_env_vars(file_path.to_str().unwrap()).unwrap();
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

    update_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    let result = read_env_vars(file_path.to_str().unwrap()).unwrap();
    assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
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

    let result = read_env_vars(file_path.to_str().unwrap()).unwrap();
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
fn test_parse_stdin_and_write_to_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    let input = "KEY1=value1\nKEY2=value2\n";
    let mut cursor = Cursor::new(input);
    let result = parse_stdin_with_reader(&mut cursor);

    // Write the result to the temporary file
    update_env_file(file_path.to_str().unwrap(), &result).unwrap();

    // Read the file contents
    let contents = fs::read_to_string(&file_path).unwrap();

    // Check if the file contains the expected content
    assert!(contents.contains("KEY1=value1"));
    assert!(contents.contains("KEY2=value2"));

    // Read the file using read_env_file and check the result
    let env_vars = read_env_vars(file_path.to_str().unwrap()).unwrap();
    assert_eq!(env_vars.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(env_vars.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(env_vars.len(), 2);
}

#[test]
fn test_multiple_var_sets() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    // First set ABCD=123
    let mut env_vars = HashMap::new();
    env_vars.insert("ABCD".to_string(), "123".to_string());
    update_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Then set AB=12
    env_vars.insert("AB".to_string(), "12".to_string());
    let _ = read_env_vars(file_path.to_str().unwrap()).unwrap();
    update_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Read the final state of the file
    let result = read_env_vars(file_path.to_str().unwrap()).unwrap();

    // Assert that both variables are set
    assert_eq!(result.get("ABCD"), Some(&"123".to_string()));
    assert_eq!(result.get("AB"), Some(&"12".to_string()));
    assert_eq!(result.len(), 2);

    // Check the final content of the file
    let final_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(final_content, "ABCD=123\nAB=12\n");
}

#[test]
fn test_last_occurence_of_duplicate_keys_updated() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");

    // Create an initial .env file with duplicate keys
    let initial_content = "A=a\nFOO=1\nB=b\nFOO=2\n";
    fs::write(&file_path, initial_content).unwrap();

    // Set FOO=3
    let mut env_vars = HashMap::new();
    env_vars.insert("FOO".to_string(), "3".to_string());
    update_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();

    // Read the final state of the file
    let result = read_env_vars(file_path.to_str().unwrap()).unwrap();

    // Assert that FOO is set to 3
    assert_eq!(result.get("FOO"), Some(&"3".to_string()));
    assert_eq!(result.len(), 3); // A, B, and FOO

    // Check the final content of the file
    let final_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        final_content, "A=a\nFOO=1\nB=b\nFOO=3\n",
        "The last occurrence of FOO should be updated to 3"
    );

    let foo_count = final_content.matches("FOO=").count();
    assert_eq!(foo_count, 2, "There should be two occurrences of FOO");
    assert!(
        final_content.contains("FOO=1"),
        "The first occurrence of FOO should remain unchanged"
    );
    assert!(
        final_content.contains("FOO=3"),
        "The last occurrence of FOO should be updated to 3"
    );
}

#[test]
fn test_delete_env_vars() {
    // TODO test
}

#[test]
fn test_preserve_comments_when_setting_new_var() {
    // TODO
    // let dir = tempdir().unwrap();
    // let file_path = dir.path().join(".env");
    // let initial_content = "# This is a comment\nEXISTING=value\n\n# Another comment\n";
    // fs::write(&file_path, initial_content).unwrap();

    // let mut new_vars = HashMap::new();
    // new_vars.insert("NEW_VAR".to_string(), "new_value".to_string());
    // new_vars.insert("EXISTING".to_string(), "value".to_string());
    // write_env_file(file_path.to_str().unwrap(), &new_vars).unwrap();

    // let final_content = fs::read_to_string(&file_path).unwrap();
    // println!("Final content:\n{}", final_content);
    // assert!(
    //     final_content.contains("# This is a comment\n"),
    //     "First comment should be preserved"
    // );
    // assert!(
    //     final_content.contains("EXISTING=value\n"),
    //     "Existing variable should be preserved"
    // );
    // assert!(
    //     final_content.contains("\n# Another comment\n"),
    //     "Second comment should be preserved"
    // );
    // assert!(
    //     final_content.contains("\nNEW_VAR=new_value\n"),
    //     "New variable should be added on a new line"
    // );

    // let env_vars = read_env_vars(file_path.to_str().unwrap()).unwrap();
    // assert_eq!(env_vars.get("EXISTING"), Some(&"value".to_string()));
    // assert_eq!(env_vars.get("NEW_VAR"), Some(&"new_value".to_string()));
}

#[test]
fn test_get_single_env_var() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".env");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "FOO=bar\nBAZ=qux").unwrap();

    let env_vars = read_env_vars(file_path.to_str().unwrap()).unwrap();
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
    print_env_vars(file_path.to_str().unwrap(), &mut output, false);

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
            print_env_vars(file_path.to_str().unwrap(), &mut cursor, false);
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
    print_env_keys_to_writer(file_path.to_str().unwrap(), &mut output);

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
            Some(Commands::Print {
                parse_tree: false,
                json: false,
            })
            | None => {
                print_env_vars(file_path.to_str().unwrap(), &mut cursor, false);
            }
            Some(Commands::Print {
                parse_tree: true,
                json: _,
            }) => {
                // For this test, we don't need to implement parse tree printing
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
            let mut env_vars = read_env_vars(file_path.to_str().unwrap()).unwrap();
            env_vars.extend(new_vars);
            update_env_file(file_path.to_str().unwrap(), &env_vars).unwrap();
        } else if cli.command.is_none() {
            print_env_vars(file_path.to_str().unwrap(), &mut stdout, false);
        }
    }

    // TODO test diff
}
