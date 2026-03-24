use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("greq")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Grep + Query: A file search tool with BM25 ranking"));

    Ok(())
}

#[test]
fn test_search_with_query() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Hello world\nThis is a test file\nHello again")?;

    let mut cmd = Command::cargo_bin("greq")?;
    cmd.arg("Hello")
        .arg(temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.txt"));

    Ok(())
}

#[test]
fn test_missing_query() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("greq")?;
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));

    Ok(())
}

#[test]
fn test_file_extensions_filter() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let rust_file = temp_dir.path().join("test.rs");
    let text_file = temp_dir.path().join("test.txt");
    
    fs::write(&rust_file, "fn main() { println!(\"Hello\"); }")?;
    fs::write(&text_file, "Hello world")?;

    let mut cmd = Command::cargo_bin("greq")?;
    cmd.arg("Hello")
        .arg("--extensions")
        .arg("rs")
        .arg(temp_dir.path());
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test.rs"));

    Ok(())
}