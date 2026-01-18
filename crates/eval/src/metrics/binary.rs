use regex::Regex;
use std::{collections::HashMap, fmt::Display, path::Path};

use rustc_demangle::demangle;

use crate::{docker::run_in_docker, project::find_elf};

#[derive(Clone, Debug)]
pub struct ElfAnalysis {
    pub functions: HashMap<String, FunctionAsm>,
    pub summary: PanicDebloatSummary,
}

impl Display for ElfAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ELF Analysis Report")?;
        writeln!(f, "===================")?;
        writeln!(f, "{}", self.summary)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct PanicDebloatSummary {
    /// Counts
    pub num_functions: usize,
    pub num_panic_functions: usize,

    /// Sizes
    pub total_bytes: usize,

    /// Upper bound of potential savings
    pub removable_panic_function_bytes: usize,
    pub removable_panic_call_bytes: usize,
}

impl Display for PanicDebloatSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Panic Debloat Summary")?;
        writeln!(f, "---------------------")?;
        writeln!(f, "Total functions: {}", self.num_functions)?;
        writeln!(f, "Panic functions: {}", self.num_panic_functions)?;
        writeln!(f, "Total size (bytes): {}", self.total_bytes)?;
        writeln!(
            f,
            "Removable panic function bytes: {}",
            self.removable_panic_function_bytes
        )?;
        writeln!(
            f,
            "Removable panic call bytes: {}",
            self.removable_panic_call_bytes
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
/// A function, brought in by the compiler, whose sole purpose is to
/// emit a panic.
pub struct PanicRootInfo {
    /// The name of the root function.
    name: String,
    /// Its size.
    size_bytes: usize,
}

impl PanicRootInfo {
    pub fn new(name: String, size_bytes: usize) -> Self {
        assert!(
            name.starts_with("core"),
            "Panic root functions are part of `core`."
        );
        Self { name, size_bytes }
    }
}
#[derive(Clone, Debug)]
pub struct PanicCallSiteInfo {
    /// Name of the function containing the call
    pub caller: String,

    /// The panic function being called.
    pub callee: String,

    /// The size of the call instruction.
    pub call_size_bytes: usize,
}

/// (Optional, internal-use struct)
/// Per-function assembly facts used to build the report.
/// This does NOT need to be serialized.
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
            panic_roots.push(PanicRootInfo::new(demangled.clone(), func_bytes));
        }

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

    // Now, aggregate the results

    // the number of panic functions.
    let removable_panic_root_bytes: usize = panic_roots.iter().map(|pr| pr.size_bytes).sum();

    // the number of panic call sites.
    let removable_panic_call_bytes: usize =
        panic_call_sites.iter().map(|pcs| pcs.call_size_bytes).sum();

    let summary = PanicDebloatSummary {
        num_functions: function_asms.len(),
        num_panic_functions: panic_roots.len(),
        total_bytes: function_asms.iter().map(|fa| fa.total_bytes).sum(),
        removable_panic_function_bytes: removable_panic_root_bytes,
        removable_panic_call_bytes,
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

struct Instr {
    addr: u64,
    text: String,
}

fn parse_instr_line(line: &str) -> Option<Instr> {
    // Trim leading whitespace
    let line = line.trim_start();

    // Split once at ':'
    let (addr_str, rest) = line.split_once(':')?;

    // Parse hex address
    let addr = u64::from_str_radix(addr_str, 16).ok()?;

    Some(Instr {
        addr,
        text: rest.trim().to_string(),
    })
}

fn byte_len_from_text(text: &str) -> usize {
    // Split on ':' to remove the address part
    let after_colon = match text.split_once(':') {
        Some((_, rest)) => rest,
        None => return 0,
    };

    // Count consecutive hex byte tokens
    after_colon
        .split_whitespace()
        .take_while(|tok| tok.len() == 2 && tok.chars().all(|c| c.is_ascii_hexdigit()))
        .count()
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
