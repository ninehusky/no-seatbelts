use regex::Regex;
use std::{collections::HashMap, fmt::Display, path::Path};

use rustc_demangle::demangle;

use crate::{
    docker::run_in_docker,
    metrics::report::{BinaryPanicStats, PanicCallSiteInfo, PanicRootInfo},
    project::find_elf,
};

#[derive(Clone, Debug)]
pub struct ElfAnalysis {
    pub functions: HashMap<String, FunctionAsm>,
    pub summary: BinaryPanicStats,
}

impl Display for ElfAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ELF Analysis Report")?;
        writeln!(f, "===================")?;
        writeln!(f, "{}", self.summary)?;
        Ok(())
    }
}

struct Instr {
    addr: u64,
    text: String,
}

/// Per-function assembly facts used to build the report.
#[derive(Clone, Debug)]
pub struct FunctionAsm {
    pub name: String,

    /// The assembly body of the function.
    pub body: String,

    /// Optional crate attribution.
    pub crate_name: Option<String>,

    /// Total size of the function in bytes.
    pub total_bytes: usize,

    /// Number of assembly instructions.
    pub num_instructions: usize,

    /// Whether this function is a known panic root.
    pub is_panic_root: bool,

    /// Number of call instructions to panic roots.
    pub num_panic_calls: usize,

    /// Total size of call instructions to panic roots.
    pub panic_call_bytes: usize,
}

pub fn analyze_elf(repo_root: &Path, elf_root: &Path) -> ElfAnalysis {
    let elf_path = find_elf(elf_root);
    let rel = elf_path
        .strip_prefix(repo_root)
        .expect("ELF not under repo root");
    let docker_elf_path = format!("/work/{}", rel.display());

    let nm_output = run_in_docker(repo_root, &["llvm-objdump", "-D", &docker_elf_path]);

    let stdout = String::from_utf8_lossy(&nm_output.stdout);
    let fns = extract_functions(&stdout);
    let mut panic_roots: Vec<PanicRootInfo> = Vec::new();
    let mut panic_call_sites = Vec::new();
    let mut function_asms: Vec<FunctionAsm> = Vec::new();
    for (fn_name, instrs) in fns {
        let body = instrs
            .iter()
            .map(|x| x.text.clone())
            .collect::<Vec<String>>()
            .join("\n");
        let demangled = demangle(&fn_name).to_string();
        let mut total_bytes = 0usize;
        let mut num_instructions = 0usize;
        let mut num_panic_calls = 0usize;
        let mut panic_call_bytes = 0usize;

        let func_bytes: usize = instrs
            .iter()
            .enumerate()
            .map(|(i, _)| instruction_size(&instrs, i))
            .sum();

        let is_panic_root = is_panic_root(&demangled);

        if is_panic_root {
            // We don't care about the internals of panic root functions, other than their size.
            panic_roots.push(PanicRootInfo::new(demangled.clone(), func_bytes));
        } else {
            for (i, instr) in instrs.iter().enumerate() {
                let instr_size = instruction_size(&instrs, i);

                total_bytes += instr_size;
                num_instructions += 1;

                if let Some(panic_callee) = is_panic_call(&instr.text) {
                    println!(
                        "PANIC CALL: caller={}, size={}, text={}",
                        demangled, instr_size, instr.text
                    );
                    num_panic_calls += 1;
                    panic_call_bytes += instr_size;

                    panic_call_sites.push(PanicCallSiteInfo {
                        caller: demangled.clone(),
                        callee: panic_callee,
                        call_size_bytes: instr_size,
                    });
                }
            }

            function_asms.push(FunctionAsm {
                name: demangled,
                body,
                crate_name: None,
                total_bytes,
                num_instructions,
                is_panic_root,
                num_panic_calls,
                panic_call_bytes,
            });
        }
    }

    // Now, aggregate the results

    // the number of panic functions.
    let removable_panic_root_bytes: usize = panic_roots.iter().map(|pr| pr.size_bytes).sum();

    let summary = BinaryPanicStats {
        num_functions: function_asms.len(),
        num_panic_functions: panic_roots.len(),
        total_bytes: function_asms.iter().map(|fa| fa.total_bytes).sum(),
        removable_panic_function_bytes: removable_panic_root_bytes,
        total_panic_calls: panic_call_sites.len(),
    };

    let mut functions_map = HashMap::new();
    for fa in function_asms {
        let res = functions_map.insert(fa.name.clone(), fa.clone());
        if res.is_some() {
            panic!("duplicate function name found: {}", fa.name);
        }
    }

    ElfAnalysis {
        functions: functions_map,
        summary,
    }
}

fn extract_functions(asm: &str) -> Vec<(String, Vec<Instr>)> {
    let fn_header = Regex::new(r"^([0-9a-f]+) <(.+)>:$").unwrap();
    let asm_line = Regex::new(r"^\s*([0-9a-f]+):\s+([0-9a-f ]+)\s+(.+)$").unwrap();

    let mut functions = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_instrs: Vec<Instr> = Vec::new();

    for line in asm.lines() {
        // Function header
        if let Some(caps) = fn_header.captures(line) {
            if let Some(name) = current_name.take() {
                functions.push((name, current_instrs));
                current_instrs = Vec::new();
            }

            current_name = Some(caps[2].to_string());
            continue;
        }

        // Instruction line
        if let Some(caps) = asm_line.captures(line) {
            if current_name.is_some() {
                let addr = u64::from_str_radix(&caps[1], 16).expect("invalid instruction address");

                let text = caps[3].trim().to_string();

                current_instrs.push(Instr { addr, text });
            }
        }
    }

    // Final function
    if let Some(name) = current_name {
        functions.push((name, current_instrs));
    }

    functions
}

fn is_panic_root(fn_name: &str) -> bool {
    is_known_panic_symbol(fn_name)
}

fn is_known_panic_symbol(sym: &str) -> bool {
    // Core panic machinery
    sym.contains("core::panicking::panic")
        || sym.contains("panic_fmt")
        || sym.contains("rust_begin_unwind")
        || sym.contains("panic_bounds_check")
        || sym.contains("panic_const")
        || sym.contains("index_fail")
        || sym.contains("slice_end_index_len_fail")
        || sym.contains("assert_failed")
}

fn is_panic_call(line: &str) -> Option<String> {
    // Only care about call instructions
    if !(line.contains("call") || line.contains("bl")) {
        eprintln!("skipping non-call line: {}", line);
        return None;
    }

    // Try to extract the callee symbol, if present
    // This depends on your objdump format; adjust as needed
    if let Some(start) = line.find('<') {
        if let Some(end) = line[start + 1..].find('>') {
            let sym = &line[start + 1..start + 1 + end];

            if is_known_panic_symbol(sym) {
                return Some(sym.to_string());
            }
        }
    }

    None
}

fn instruction_size(instrs: &[Instr], i: usize) -> usize {
    if i + 1 < instrs.len() {
        (instrs[i + 1].addr - instrs[i].addr) as usize
    } else {
        // TODO: fix this.
        eprintln!("approximating the last instruction size to be 5 bytes");
        5
    }
}
