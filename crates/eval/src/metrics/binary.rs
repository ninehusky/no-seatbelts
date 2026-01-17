use regex::Regex;
use std::path::Path;

use rustc_demangle::demangle;

use crate::{docker::run_in_docker, project::find_elf};

#[derive(Clone, Debug)]
pub struct FunctionAsm {
    /// The name of the function.
    name: String,
    /// The number of assembly instructions for the function.
    num_instructions: usize,
    /// The number of panic-related instructions for the function.
    num_panic_instructions: usize,
    /// The total size in bytes of the function.
    total_bytes: usize,
    /// The size in bytes of panic-related instructions for the function.
    panic_bytes: usize,
}
// result = subprocess.run(["llvm-objdump", "-D", binary],

pub fn analyze_elf(repo_root: &Path, elf_root: &Path) {
    let elf_path = find_elf(elf_root);
    let rel = elf_path
        .strip_prefix(&repo_root)
        .expect("ELF not under repo root");
    let docker_elf_path = format!("/work/{}", rel.display());

    let nm_output = run_in_docker(repo_root, &["llvm-objdump", "-D", &docker_elf_path]);

    let stdout = String::from_utf8_lossy(&nm_output.stdout);
    println!("asm:");
    println!("{}", stdout);
    let fns = extract_functions(&stdout);
    for (fn_name, instrs) in fns {
        println!("name: {}", demangle(&fn_name));
        println!("instrs: {}", instrs.len());
        println!(
            "num panic instrs: {}",
            instrs.iter().filter(|l| is_panic_line(l)).count()
        );
        println!(
            "size: {}",
            instrs
                .into_iter()
                .map(|l| estimate_instruction_size(&l))
                .sum::<usize>()
        );
        println!();
    }
}

fn extract_functions(asm: &str) -> Vec<(String, Vec<String>)> {
    let fn_header = Regex::new(r"^([0-9a-f]+) <(.+)>:$").unwrap();
    let asm_line = Regex::new(r"^\s*[0-9a-f]+:\s+([0-9a-f ]+)\s+(.+)$").unwrap();

    let mut functions = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_asm: Vec<String> = Vec::new();

    for line in asm.lines() {
        if let Some(caps) = fn_header.captures(line) {
            if let Some(name) = current_name.take() {
                functions.push((name, current_asm));
                current_asm = Vec::new();
            }
            current_name = Some(caps[2].to_string());
            continue;
        }

        if asm_line.is_match(line) {
            if current_name.is_some() {
                current_asm.push(line.to_string());
            }
        }
    }

    if let Some(name) = current_name {
        functions.push((name, current_asm));
    }

    functions
}

fn is_panic_line(line: &str) -> bool {
    line.contains("panic") || line.contains("rust_begin_unwind") || line.contains("index_fail")
}

fn estimate_instruction_size(line: &str) -> usize {
    // Split once at ':' to drop the address
    let after_colon = match line.split_once(':') {
        Some((_, rest)) => rest,
        None => return 0, // not an instruction line
    };

    let mut count = 0;

    for tok in after_colon.split_whitespace() {
        // Instruction bytes are exactly two hex digits
        if tok.len() == 2 && tok.chars().all(|c| c.is_ascii_hexdigit()) {
            count += 1;
        } else {
            // First non-byte token = mnemonic; stop
            break;
        }
    }

    count
}
