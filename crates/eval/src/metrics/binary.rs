use std::path::Path;

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
    println!("ELF Analysis for binary at {}", elf_root.display());
    println!("----------------------------------------");
    println!("{}", stdout);
}

fn estimate_instruction_size(line: &str) -> usize {
    todo!()
}
