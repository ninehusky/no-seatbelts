use std::{collections::HashMap, hash::Hash, path::Path};

use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    docker::{TargetArch, run_in_docker, to_container_path},
    workspace::find_eval_root,
};

pub type GlobalFunctionInfo = HashMap<String, FunctionInfo>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSummaryAnalysis {
    pub baseline: GlobalFunctionInfo,
    pub edited: GlobalFunctionInfo,
    pub delta: Vec<FunctionDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub num_instrs: u64,
    pub num_calls: u64,
    pub panic_calls: Vec<PanicCallInfo>,
    pub is_panic_fn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDelta {
    pub name: String,
    pub num_instrs_delta: i64,
    pub num_calls_delta: i64,
    pub panic_calls_delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicCallInfo {
    pub caller: String,
    pub callee: String,
    pub asm: String,
}

// Finds the panic calls in the given GlobalFunctionInfo and returns them as a flat vector.
pub fn find_panics(info: &GlobalFunctionInfo) -> Vec<&PanicCallInfo> {
    let mut panics = vec![];
    for fn_info in info.values() {
        for panic_call in &fn_info.panic_calls {
            panics.push(panic_call);
        }
    }
    panics
}

pub fn get_function_summary(
    target: &TargetArch,
    baseline_elf_path: &Path,
    edited_elf_path: &Path,
) -> anyhow::Result<FunctionSummaryAnalysis> {
    let baseline_info = get_global_fn_info(target, baseline_elf_path)?;
    let edited_info = get_global_fn_info(target, edited_elf_path)?;

    let mut delta = vec![];
    for (name, base_fn) in &baseline_info {
        if let Some(edited_fn) = edited_info.get(name) {
            delta.push(FunctionDelta {
                name: name.clone(),
                num_instrs_delta: edited_fn.num_instrs as i64 - base_fn.num_instrs as i64,
                num_calls_delta: edited_fn.num_calls as i64 - base_fn.num_calls as i64,
                panic_calls_delta: edited_fn.panic_calls.len() as i64
                    - base_fn.panic_calls.len() as i64,
            });
        }
    }

    Ok(FunctionSummaryAnalysis {
        baseline: baseline_info,
        edited: edited_info,
        delta,
    })
}

pub fn get_global_fn_info(
    target: &TargetArch,
    elf_path: &Path,
) -> anyhow::Result<GlobalFunctionInfo> {
    let disasm = run_in_docker(
        &find_eval_root()?,
        &[
            "llvm-objdump",
            "-D",
            &to_container_path(&find_eval_root()?, elf_path),
        ],
    )?;

    let fns = extract_functions(&String::from_utf8_lossy(&disasm.stdout))?;
    let mut fn_infos = HashMap::new();
    for (name, instrs) in fns {
        let info = build_function_info(name.clone(), instrs, target);
        fn_infos.insert(name, info);
    }

    Ok(fn_infos)
}

// Build the FunctionInfo for a single function given its name and instructions.
fn build_function_info(name: String, instrs: Vec<Instr>, target: &TargetArch) -> FunctionInfo {
    let instr_count = instrs.len() as u64;

    let mut call_count = 0;
    let mut panic_calls = vec![];

    for instr in instrs {
        if target.is_call(&instr.text) {
            call_count += 1;
            if let Some(callee) = extract_panic_callee(&instr.text) {
                panic_calls.push(PanicCallInfo {
                    caller: name.clone(),
                    callee,
                    asm: instr.text.clone(),
                });
            }
        }
    }
    FunctionInfo {
        name: name.clone(),
        num_instrs: instr_count,
        num_calls: call_count,
        panic_calls,
        is_panic_fn: is_known_panic_symbol(name.as_str()),
    }
}

// Given the disassembly for a particular binary, return a vector of the (name, body) of each function.
fn extract_functions(asm: &str) -> anyhow::Result<Vec<(String, Vec<Instr>)>> {
    // Function header is an address followed by <symbol>.
    let fn_header = Regex::new(r"^([0-9a-f]+)\s+<(.+)>:$").unwrap();

    // Instruction line is an address followed by ':' and the instruction text.
    let instr_line = Regex::new(r"^\s*([0-9a-f]+):\s+(.*)$").unwrap();

    let mut functions = vec![];
    let mut current_fn = None;
    let mut current_instrs = vec![];

    for line in asm.lines() {
        if let Some(caps) = fn_header.captures(line) {
            // If we were in a function, save it before starting the new one.
            if let Some(name) = current_fn.take() {
                functions.push((name, current_instrs));
                current_instrs = vec![];
            }
            current_fn = Some(caps[2].to_string());
            continue;
        }

        if let Some(caps) = instr_line.captures(line) {
            if current_fn.is_some() {
                let addr = u64::from_str_radix(&caps[1], 16).with_context(|| {
                    format!("failed to parse instruction address in line: {}", line)
                })?;
                let text = caps[2].trim().to_string();
                current_instrs.push(Instr { text, addr });
            } else {
                return Err(anyhow::anyhow!(
                    "found instruction line outside of function: {}",
                    line
                ));
            }
        }
    }

    if let Some(name) = current_fn.take() {
        functions.push((name, current_instrs));
    }

    Ok(functions)
}

fn extract_panic_callee(asm: &str) -> Option<String> {
    let start = asm.find('<')?;
    let end = asm[start + 1..].find('>')?;
    let sym = &asm[start + 1..start + 1 + end];

    if is_known_panic_symbol(sym) {
        Some(sym.to_string())
    } else {
        None
    }
}

// A single assembly instruction.
struct Instr {
    text: String,
    addr: u64,
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
