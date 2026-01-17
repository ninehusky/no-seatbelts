use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    process::Command,
};

use rustfix::{CodeFix, Filter, Suggestion};

pub fn run_no_seatbelts(repo_root: &Path, entry: &Path) -> Vec<Suggestion> {
    let output = Command::new("cargo")
        .current_dir(repo_root)
        .args([
            "run",
            "-p",
            "no-seatbelts",
            "--bin",
            "no-seatbelts",
            "--",
            entry.to_str().unwrap(),
            "--error-format=json",
            "--no-std",
        ])
        .output()
        .expect("failed to run no-seatbelts");

    if !output.status.success() {
        println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        println!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("no-seatbelts failed to run");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    let json_only: String = stderr
        .lines()
        .filter(|line| line.trim_start().starts_with('{'))
        .collect::<Vec<_>>()
        .join("\n");

    rustfix::get_suggestions_from_json(
        &json_only,
        &HashSet::<String>::default(),
        Filter::Everything,
    )
    .expect("failed to parse suggestions")
}

pub fn apply_suggestions(suggestions: &Vec<Suggestion>) {
    let mut fixes = HashMap::new();
    for suggestion in suggestions {
        let file_name = &suggestion.snippets[0].file_name;
        fixes
            .entry(file_name)
            .or_insert_with(Vec::new)
            .push(suggestion);
    }

    for (source_file, suggestions) in fixes {
        let source = fs::read_to_string(source_file).expect("Couldn't read source file");
        let mut fix = CodeFix::new(&source);
        for suggestion in suggestions.iter() {
            if let Err(e) = fix.apply(suggestion) {
                panic!("Failed to apply suggestion to {:?}: {}", source_file, e);
            }
        }
        let fixes = fix.finish().expect("Failed to finish applying fixes");
        fs::write(source_file, fixes).expect("Couldn't write fixed source file");
    }

    println!("applied {} fixes", suggestions.len());
}
