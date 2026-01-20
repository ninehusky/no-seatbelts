use std::{collections::BTreeMap, path::Path};

use serde::Serialize;

use crate::{
    docker::{run_in_docker, to_container_path},
    project::find_elf,
};

#[derive(Clone, Debug, Serialize)]
pub struct SizeReport {
    pub binary: String,
    pub section_sizes: SectionSizes,
    pub function_sizes: Vec<FunctionSizes>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionSizes {
    pub sections: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionSizes {
    pub name: String,
    pub size_bytes: u64,
}

pub fn get_size_report(repo_root: &Path, crate_root: &Path) -> SizeReport {
    let elf_path = find_elf(crate_root);
    let binary_name = elf_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let section_sizes = get_section_sizes(repo_root, crate_root);
    let function_sizes = get_function_sizes(repo_root, crate_root);

    SizeReport {
        binary: binary_name,
        section_sizes,
        function_sizes,
    }
}

fn get_function_sizes(repo_root: &Path, crate_root: &Path) -> Vec<FunctionSizes> {
    let elf_path = find_elf(crate_root);

    let output = run_in_docker(
        repo_root,
        &[
            "llvm-nm",
            "--print-size",
            "--size-sort",
            "--radix=d",
            to_container_path(repo_root, &elf_path).as_str(),
        ],
    );

    assert!(
        output.status.success(),
        "llvm-nm failed with status {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter_map(|line| {
            // Split into at most 4 fields
            let mut parts = line.split_whitespace();

            let _addr = parts.next()?;
            let size = parts.next()?.parse::<u64>().ok()?;
            let sym_type = parts.next()?;
            let name = parts.next()?.to_string();

            // Only count text symbols
            if sym_type == "t" || sym_type == "T" {
                Some(FunctionSizes {
                    name,
                    size_bytes: size,
                })
            } else {
                None
            }
        })
        .collect()
}

fn get_section_sizes(repo_root: &Path, crate_root: &Path) -> SectionSizes {
    let elf_path = find_elf(crate_root);

    let output = run_in_docker(
        repo_root,
        &[
            "llvm-readelf",
            "-S",
            to_container_path(repo_root, &elf_path).as_str(),
        ],
    );

    // let output = std::process::Command::new("llvm-readelf")
    //     .arg("-S")
    //     .arg(&elf_path)
    //     .output()
    //     .expect("failed to run llvm-readelf");

    assert!(
        output.status.success(),
        "llvm-readelf failed with status {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sections = BTreeMap::new();

    for line in stdout.lines() {
        let line = line.trim();
        if !line.starts_with('[') {
            continue;
        }

        // Tokenize
        let raw: Vec<&str> = line.split_whitespace().collect();
        if raw.len() < 3 {
            continue;
        }

        // Normalize the prefix "[ 6]" or "[10]" into a single numeric token ("6" / "10"),
        // so the rest of the fields shift into consistent positions.
        //
        // Cases:
        //   A) ["[", "6]", ".text", ...]
        //   B) ["[10]", ".got", ...]
        let mut fields: Vec<&str> = Vec::with_capacity(raw.len());

        if raw[0] == "[" {
            // A: "[" "6]" ...
            if raw.len() < 3 {
                continue;
            }
            let nr = raw[1].trim_end_matches(']');
            fields.push(nr);
            fields.extend_from_slice(&raw[2..]);
        } else {
            // B: "[10]" ...
            let nr = raw[0].trim_start_matches('[').trim_end_matches(']');
            fields.push(nr);
            fields.extend_from_slice(&raw[1..]);
        }

        // Now: fields = ["6", ".text", "PROGBITS", "00001000", "001000", "00057b", ...]
        // Indices:         0     1         2         3          4        5
        if fields.len() < 6 {
            continue;
        }

        let name = fields[1].to_string();
        let size_hex = fields[5];

        if let Ok(size) = u64::from_str_radix(size_hex, 16) {
            // Optional: skip the null/empty-name section [0]
            if !name.is_empty() {
                sections.insert(name, size);
            }
        }
    }

    // Strong sanity checks for your use case:
    assert!(
        sections.contains_key(".text"),
        "section table missing .text; parser likely broken"
    );

    SectionSizes { sections }
}
